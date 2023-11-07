use crate::input_action::{BaseInputAction, InputAction, InputActionHandler, SingleActorAction};
use glam::{vec3, EulerRot, Quat, Vec3};
use mint::Vector3;
use stardust_xr_fusion::{
	client::FrameInfo,
	core::values::Transform,
	fields::{Field, UnknownField},
	input::{InputData, InputDataType, InputHandler},
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};
use tokio::sync::mpsc;
use tracing::{debug, trace};

pub fn swing_twist_decomposition(rotation: Quat, direction: Vec3) -> (Quat, Quat) {
	let ra = Vec3::new(rotation.x, rotation.y, rotation.z); // rotation axis
	let p = ra.project_onto(direction); // projection of ra onto direction
	let twist = Quat::from_xyzw(p.x, p.y, p.z, rotation.w).normalize();
	let swing = rotation * twist.conjugate();
	(swing, twist)
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
	condition_action: BaseInputAction<GrabData>,
	grab_action: SingleActorAction<GrabData>,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<GrabData>>,
	field: UnknownField,
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
	pub fn create<Fi: Field>(
		content_space: &Spatial,
		content_transform: Transform,
		field: &Fi,
		settings: GrabbableSettings,
	) -> Result<Self, NodeError> {
		let condition_action = BaseInputAction::new(false, |input, data: &GrabData| {
			let max_distance = data.settings.max_distance;
			match &input.input {
				InputDataType::Hand(h) => h.thumb.tip.distance < 0.0 && h.index.tip.distance < 0.0,
				_ => input.distance < max_distance,
			}
		});
		let grab_action = SingleActorAction::new(
			true,
			|data, _| {
				data.datamap.with_data(|datamap| match &data.input {
					InputDataType::Hand(_) => datamap.idx("pinch_strength").as_f32() > 0.90,
					_ => datamap.idx("grab").as_f32() > 0.90,
				})
			},
			false,
		);
		let input_handler =
			InputHandler::create(content_space.client()?.get_root(), Transform::none(), field)?
				.wrap(InputActionHandler::new(GrabData { settings }))?;
		let root = Spatial::create(input_handler.node(), Transform::none(), false)?;
		let content_parent = Spatial::create(input_handler.node(), Transform::none(), true)?;
		content_parent.set_transform(Some(content_space), content_transform)?;

		let (closest_point_tx, closest_point_rx) = mpsc::channel(1);
		Ok(Grabbable {
			root,
			content_parent,
			condition_action,
			grab_action,
			input_handler,
			field: field.alias_unknown_field(),
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
		// update input actions
		self.input_handler.lock_wrapped().update_actions([
			self.condition_action.type_erase(),
			self.grab_action.type_erase(),
		]);
		self.grab_action.update(&mut self.condition_action);

		if self.grab_action.actor_started() {
			// Make sure we can directly apply the grab data to the content parent
			self.content_parent
				.set_spatial_parent_in_place(self.input_handler.node())?;
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

			self.root.set_transform(
				Some(self.input_handler.node()),
				Transform::from_position_rotation(position, rotation),
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
						let result = async { field.closest_point(&root, [0.0; 3])?.await }
							.await
							.unwrap();
						// if let Ok(result) = result {
						let _ = closest_point_tx.send(result.into()).await;
						// }
					});
				}
			}
		}
		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");
			self.content_parent.set_zoneable(true)?;

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
				self.root.set_transform(
					Some(self.input_handler.node()),
					Transform::from_position_rotation(self.pose.0, self.pose.1),
				)?;
			}
		}

		self.frame += 1;
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
					PointerMode::Parent => (grab_point, p.orientation.into()),
					PointerMode::Align => (grab_point, {
						swing_twist_decomposition(Quat::from(p.orientation), p.direction().into()).0
					}),
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

	pub fn grab_action(&self) -> &SingleActorAction<GrabData> {
		&self.grab_action
	}
	pub fn content_parent(&self) -> &Spatial {
		&self.content_parent
	}

	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input_handler.node().set_enabled(enabled)
	}
}
