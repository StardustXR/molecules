use std::f32::consts::PI;

use crate::input_action::{InputQueue, InputQueueable, SingleActorAction};
use glam::{vec3, Quat, Vec3};
use mint::Vector3;
use stardust_xr_fusion::{
	client::FrameInfo,
	fields::{Field, FieldAspect},
	input::{InputData, InputDataType, InputHandler},
	node::{NodeError, NodeType},
	spatial::{Spatial, SpatialAspect, Transform},
};
use tokio::sync::mpsc;
use tracing::{debug, trace};

fn swing_direction(direction: Vec3) -> Quat {
	let pitch = direction.y.asin();
	let yaw = direction.z.atan2(direction.x);
	Quat::from_rotation_y(-yaw - PI / 2.0) * Quat::from_rotation_x(pitch)
}

/// How should the grabbable interact with pointers?
#[derive(Debug, Clone, Copy)]
pub enum PointerMode {
	/// Grabbable should act as a child of the pointer, its rotation stays constant relative to the pointer
	Parent,
	/// The grabbable aligns its forward direction with the pointer ray
	Align,
	/// The grabbable never rotates, only moves
	Move,
}

/// Linear drag is in m/s, angular drag is in rad/s.
#[derive(Debug, Clone, Copy)]
pub struct MomentumSettings {
	/// Drag (unity style) for momentum.
	pub drag: f32,
	/// Minimum speed before momentum applies.
	pub threshold: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GrabbableSettings {
	/// Max distance that you can be to start grabbing
	pub max_distance: f32,
	/// None means no linear momentum.
	pub linear_momentum: Option<MomentumSettings>,
	/// None means no angular momentum.
	pub angular_momentum: Option<MomentumSettings>,
	/// Should the grabbable be magnetized to the grab point?
	pub magnet: bool,
	/// How should pointers be handled?
	pub pointer_mode: PointerMode,
	/// Should the object be movable by zones?
	pub zoneable: bool,
}
impl Default for GrabbableSettings {
	fn default() -> Self {
		Self {
			max_distance: 0.05,
			linear_momentum: Some(MomentumSettings {
				drag: 8.0,
				threshold: 0.01,
			}),
			angular_momentum: Some(MomentumSettings {
				drag: 15.0,
				threshold: 0.2,
			}),
			magnet: true,
			pointer_mode: PointerMode::Parent,
			zoneable: true,
		}
	}
}

#[derive(Clone)]
pub struct GrabData {
	pub settings: GrabbableSettings,
}

pub struct Grabbable {
	root: Spatial,
	content_parent: Spatial,
	field: Field,
	input: InputQueue,
	grab_action: SingleActorAction,
	pointer_distance: f32,
	settings: GrabbableSettings,
	frame: u32,
	start_frame: u32,
	start_pose: (Vec3, Quat),
	prev_pose: (Vec3, Quat),
	pose: (Vec3, Quat),

	closest_point_tx: mpsc::Sender<Vec3>,
	closest_point_rx: mpsc::Receiver<Vec3>,

	linear_velocity: Option<Vec3>,
	angular_velocity: Option<(Vec3, f32)>,
}
impl Grabbable {
	pub fn create(
		content_space: &impl SpatialAspect,
		content_transform: Transform,
		field: &impl FieldAspect,
		settings: GrabbableSettings,
	) -> Result<Self, NodeError> {
		let input =
			InputHandler::create(content_space.client()?.get_root(), Transform::none(), field)?
				.queue()?;
		let grab_action = SingleActorAction::default();
		let root = Spatial::create(input.handler(), Transform::none(), false)?;
		let content_parent =
			Spatial::create(input.handler(), Transform::none(), settings.zoneable)?;
		content_parent.set_relative_transform(content_space, content_transform)?;

		let (closest_point_tx, closest_point_rx) = mpsc::channel(1);
		Ok(Grabbable {
			root,
			content_parent,
			input,
			grab_action,
			field: Field::alias_field(field),
			pointer_distance: 0.0,
			settings,
			frame: 0,
			start_frame: 0,
			start_pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),
			prev_pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),
			pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),

			closest_point_tx,
			closest_point_rx,

			linear_velocity: None,
			angular_velocity: None,
		})
	}
	pub fn update(&mut self, info: &FrameInfo) -> Result<(), NodeError> {
		self.grab_action.update(
			true,
			&self.input,
			|input| {
				let max_distance = self.settings.max_distance;
				match &input.input {
					InputDataType::Hand(h) => {
						h.thumb.tip.distance < max_distance && h.index.tip.distance < max_distance
					}
					_ => input.distance < max_distance,
				}
			},
			|data| {
				data.datamap.with_data(|datamap| match &data.input {
					InputDataType::Hand(_) => datamap.idx("pinch_strength").as_f32() > 0.90,
					_ => datamap.idx("grab").as_f32() > 0.90,
				})
			},
		);

		if self.grab_action.actor_started() {
			// Make sure we can directly apply the grab data to the content parent
			self.content_parent
				.set_spatial_parent_in_place(self.input.handler())?;
			let actor = self.grab_action.actor().unwrap();
			if let InputDataType::Pointer(pointer) = &actor.input {
				// store the pointer distance so we can keep it at the correct point
				self.pointer_distance =
					Vec3::from(pointer.origin).distance(pointer.deepest_point.into());
			}
			self.start_frame = self.frame;
		}

		if let Some(actor) = self.grab_action.actor().cloned() {
			let (mut position, rotation) = self.input_position_rotation(&actor);
			debug!(?position, ?rotation, uid = actor.uid, "Currently grabbing");

			if self.settings.magnet {
				if let Ok(closest_point) = self.closest_point_rx.try_recv() {
					position -= rotation * closest_point;
					let _ = self.closest_point_tx.try_send(closest_point);
				}
			}
			let transform_spatial = match (self.settings.pointer_mode, &actor.input) {
				(PointerMode::Align, InputDataType::Pointer(_)) => self.content_parent(),
				_ => &self.root,
			};
			transform_spatial.set_relative_transform(
				self.input.handler(),
				Transform::from_translation_rotation(position, rotation),
			)?;

			self.prev_pose = self.pose;
			self.pose = (position, rotation);
			if self.grab_action.actor_started() {
				self.start_pose = self.pose;
			}

			let delta = info.delta as f32;
			if let Some(momentum_settings) = &self.settings.linear_momentum {
				let linear_velocity = self.pose.0 - self.prev_pose.0;
				let above_threshold =
					linear_velocity.length_squared() > momentum_settings.threshold.powf(2.0);
				self.linear_velocity = above_threshold.then(|| linear_velocity / delta);
			}
			if let Some(momentum_settings) = &self.settings.angular_momentum {
				let (axis, angle) = (self.pose.1 * self.prev_pose.1.inverse()).to_axis_angle();
				let above_threshold = angle > momentum_settings.threshold;
				self.angular_velocity = above_threshold.then(|| (axis, angle / delta));
			}
		}

		if self.grab_action.actor_started() {
			debug!(
				uid = self.grab_action.actor().as_ref().unwrap().uid,
				"Started grabbing"
			);
			self.content_parent.set_zoneable(false)?;
			self.content_parent
				.set_spatial_parent_in_place(&self.root)?;

			'magnet: {
				if self.settings.magnet {
					// if we have magnet strength, store the closest point so we can lerp that to the grab point
					let grab_data = self.grab_action.actor().unwrap().clone();
					// pointers are just too unstable to magnet
					if let InputDataType::Pointer(_) = &grab_data.input {
						break 'magnet;
					}
					let field = self.field.alias();
					let root = self.root.alias();
					let closest_point_tx = self.closest_point_tx.clone();
					tokio::task::spawn(async move {
						let result = field.closest_point(&root, [0.0; 3]).await.unwrap();
						// if let Ok(result) = result {
						let _ = closest_point_tx.send(result.into()).await;
						// }
					});
				}
			}
		}
		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");
			self.content_parent.set_zoneable(self.settings.zoneable)?;

			// drain the closest point queue
			let _ = self.closest_point_rx.try_recv();
		}

		if !self.grab_action.actor_acting() {
			if let Some(settings) = self.settings.linear_momentum {
				self.apply_linear_momentum(info, settings);
			}
			if let Some(settings) = self.settings.angular_momentum {
				self.apply_angular_momentum(info, settings);
			}

			if self.linear_velocity.is_some() || self.angular_velocity.is_some() {
				self.root.set_relative_transform(
					self.input.handler(),
					Transform::from_translation_rotation(self.pose.0, self.pose.1),
				)?;
			}
		}

		self.frame += 1;
		self.input.flush_queue();
		Ok(())
	}
	fn input_position_rotation(&mut self, input: &InputData) -> (Vec3, Quat) {
		match &input.input {
			InputDataType::Hand(h) => (
				Vec3::from(h.thumb.tip.position).lerp(Vec3::from(h.index.tip.position), 0.5),
				h.palm.rotation.into(),
			),
			InputDataType::Pointer(p) => {
				let scroll = input
					.datamap
					.with_data(|d| d.idx("scroll_continuous").as_vector().idx(1).as_f32());
				self.pointer_distance += scroll * 0.01;
				let grab_point =
					Vec3::from(p.origin) + (Vec3::from(p.direction()) * self.pointer_distance);
				match self.settings.pointer_mode {
					PointerMode::Parent => (p.origin.into(), p.orientation.into()),
					PointerMode::Align => (grab_point, swing_direction(p.direction().into())),
					PointerMode::Move => (grab_point, Quat::IDENTITY),
				}
			}
			InputDataType::Tip(t) => (t.origin.into(), t.orientation.into()),
		}
	}
	const LINEAR_VELOCITY_STOP_THRESHOLD: f32 = 0.0001;
	fn apply_linear_momentum(&mut self, info: &FrameInfo, settings: MomentumSettings) {
		let Some(velocity) = &mut self.linear_velocity else {
			return;
		};
		let delta = info.delta as f32;
		if velocity.length_squared() < Self::LINEAR_VELOCITY_STOP_THRESHOLD {
			self.linear_velocity.take();
		} else {
			*velocity *= (1.0 - settings.drag * delta).clamp(0.0, 1.0);
			self.pose.0 += *velocity * delta;
			trace!(?velocity, "linear momentum");
		}
	}
	const ANGULAR_VELOCITY_STOP_THRESHOLD: f32 = 0.001;
	fn apply_angular_momentum(&mut self, info: &FrameInfo, settings: MomentumSettings) {
		let Some((axis, angle)) = &mut self.angular_velocity else {
			return;
		};
		let delta = info.delta as f32;
		if *angle < Self::ANGULAR_VELOCITY_STOP_THRESHOLD {
			self.angular_velocity.take();
		} else {
			*angle *= (1.0 - settings.drag * delta).clamp(0.0, 1.0);
			self.pose.1 *= Quat::from_axis_angle(*axis, *angle * delta);
			trace!(?axis, angle, "angular momentum");
		}
	}

	pub fn linear_velocity(&self) -> Option<Vector3<f32>> {
		self.linear_velocity.map(|v| v.into())
	}
	pub fn linear_speed(&self) -> Option<f32> {
		self.linear_velocity.map(|v| v.length())
	}
	pub fn cancel_linear_velocity(&mut self) {
		self.linear_velocity.take();
	}
	pub fn just_stopped_moving(&self) -> bool {
		!self.grab_action.actor_acting()
			&& self.linear_velocity.is_some()
			&& self.linear_velocity.unwrap().length_squared() < Self::LINEAR_VELOCITY_STOP_THRESHOLD
	}
	pub fn angular_velocity(&self) -> Option<(Vector3<f32>, f32)> {
		self.angular_velocity.map(|(a, v)| (a.into(), v))
	}
	pub fn cancel_angular_velocity(&mut self) {
		self.angular_velocity.take();
	}
	pub fn just_stopped_rotating(&self) -> bool {
		!self.grab_action.actor_acting()
			&& self.angular_velocity.is_some()
			&& self.angular_velocity.unwrap().1 < Self::ANGULAR_VELOCITY_STOP_THRESHOLD
	}

	pub fn grab_action(&self) -> &SingleActorAction {
		&self.grab_action
	}
	pub fn content_parent(&self) -> &Spatial {
		&self.content_parent
	}

	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.handler().set_enabled(enabled)
	}
}
