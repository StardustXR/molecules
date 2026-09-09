use glam::{Mat4, Quat, Vec3};
use stardust_xr_fusion::{
	fields::{FieldInterface, FieldRef},
	spatial::{SpatialInterface, SpatialRef},
	suis::{Finger, Hand, InputDataType, Joint, Pointer, SpatialData, Thumb, Tip},
	types::Posef,
};

/// Move input data from the space it was made in into an input handler's space, filling in
/// every distance the handler expects along the way.
///
/// All the field sampling this needs runs at once, so a hand costs one round trip rather
/// than one per joint.
pub async fn localize(
	spatials: &SpatialInterface,
	fields: &FieldInterface,
	from: &SpatialRef,
	handler_spatial: &SpatialRef,
	handler_field: &FieldRef,
	input: &InputDataType,
) -> Option<SpatialData> {
	let transform = spatials
		.get_relative_transform(handler_spatial.clone(), from.clone())
		.await
		.ok()?
		.ok()?;
	let to_handler = Mat4::from_scale_rotation_translation(
		transform.scale.into(),
		transform.rotation.into(),
		transform.translation.into(),
	);
	let rotation = Quat::from(transform.rotation);

	let ctx = Localize {
		fields,
		from,
		handler_field,
		to_handler,
		rotation,
	};

	Some(match input {
		InputDataType::Pointer { data } => ctx.pointer(data).await?,
		InputDataType::Hand { data } => ctx.hand(data).await,
		InputDataType::Tip { data } => ctx.tip(data).await?,
	})
}

struct Localize<'a> {
	fields: &'a FieldInterface,
	from: &'a SpatialRef,
	handler_field: &'a FieldRef,
	to_handler: Mat4,
	rotation: Quat,
}
impl Localize<'_> {
	fn pose(&self, pose: Posef) -> Posef {
		Posef {
			position: self
				.to_handler
				.transform_point3(pose.position.into())
				.into(),
			orientation: (self.rotation * Quat::from(pose.orientation)).into(),
		}
	}

	/// distance is sampled at the untransformed position in the method's own space, which is
	/// the same point in the world and doesn't lean on the transform being right
	async fn distance(&self, position: impl Into<Vec3>) -> f32 {
		let position: Vec3 = position.into();
		self.fields
			.sample(
				self.handler_field.clone(),
				self.from.clone(),
				position.into(),
			)
			.await
			.map(|sample| sample.distance)
			.unwrap_or(f32::INFINITY)
	}

	async fn joint(&self, joint: &Joint) -> Joint {
		Joint {
			pose: self.pose(joint.pose),
			radius: joint.radius,
			distance: self.distance(joint.pose.position).await,
		}
	}

	async fn finger(&self, finger: &Finger) -> Finger {
		let (tip, distal, intermediate, proximal, metacarpal) = futures::join!(
			self.joint(&finger.tip),
			self.joint(&finger.distal),
			self.joint(&finger.intermediate),
			self.joint(&finger.proximal),
			self.joint(&finger.metacarpal),
		);
		Finger {
			tip,
			distal,
			intermediate,
			proximal,
			metacarpal,
		}
	}

	async fn thumb(&self, thumb: &Thumb) -> Thumb {
		let (tip, distal, proximal, metacarpal) = futures::join!(
			self.joint(&thumb.tip),
			self.joint(&thumb.distal),
			self.joint(&thumb.proximal),
			self.joint(&thumb.metacarpal),
		);
		Thumb {
			tip,
			distal,
			proximal,
			metacarpal,
		}
	}

	async fn localized_hand(&self, hand: &Hand) -> Hand {
		let (thumb, index, middle, ring, little, palm, wrist, elbow) = futures::join!(
			self.thumb(&hand.thumb),
			self.finger(&hand.index),
			self.finger(&hand.middle),
			self.finger(&hand.ring),
			self.finger(&hand.little),
			self.joint(&hand.palm),
			self.joint(&hand.wrist),
			async {
				match &hand.elbow {
					Some(elbow) => Some(self.joint(elbow).await),
					None => None,
				}
			},
		);
		Hand {
			chirality: hand.chirality,
			thumb,
			index,
			middle,
			ring,
			little,
			palm,
			wrist,
			elbow,
		}
	}

	async fn hand(&self, hand: &Hand) -> SpatialData {
		let hand = self.localized_hand(hand).await;
		SpatialData {
			distance: hand_distance(&hand),
			input: InputDataType::Hand { data: hand },
		}
	}

	async fn tip(&self, tip: &Tip) -> Option<SpatialData> {
		let (distance, simulated_hand) = futures::join!(self.distance(tip.pose.position), async {
			match &tip.simulated_hand {
				Some(hand) => Some(self.localized_hand(hand).await),
				None => None,
			}
		});
		Some(SpatialData {
			input: InputDataType::Tip {
				data: Tip {
					pose: self.pose(tip.pose),
					chirality: tip.chirality,
					grip_pose: tip.grip_pose.map(|p| self.pose(p)),
					grip_surface_pose: tip.grip_surface_pose.map(|p| self.pose(p)),
					simulated_hand,
				},
			},
			distance,
		})
	}

	async fn pointer(&self, pointer: &Pointer) -> Option<SpatialData> {
		let ray = self
			.fields
			.ray_march(
				self.handler_field.clone(),
				self.from.clone(),
				pointer.pose.position,
				pointer.direction(),
			)
			.await
			.ok()??;
		Some(SpatialData {
			input: InputDataType::Pointer {
				data: Pointer {
					pose: self.pose(pointer.pose),
					deepest_point: ray.deepest_point_distance,
				},
			},
			distance: ray.min_distance,
		})
	}
}

/// closest any part of the hand gets to the field
///
/// the server weights four fingertips instead, but that's a heuristic for *ordering*
/// handlers, which is [`super::InputMethodHelper::order_handlers_and_captures`]'s job here
fn hand_distance(hand: &Hand) -> f32 {
	let finger = |f: &Finger| [f.tip, f.distal, f.intermediate, f.proximal, f.metacarpal];
	[
		hand.thumb.tip,
		hand.thumb.distal,
		hand.thumb.proximal,
		hand.thumb.metacarpal,
		hand.palm,
		hand.wrist,
	]
	.into_iter()
	.chain(hand.elbow)
	.chain(finger(&hand.index))
	.chain(finger(&hand.middle))
	.chain(finger(&hand.ring))
	.chain(finger(&hand.little))
	.map(|joint| joint.distance - joint.radius)
	.fold(f32::INFINITY, f32::min)
}
