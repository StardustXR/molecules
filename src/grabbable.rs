use crate::single_actor_action::SingleActorAction;
use glam::{vec3, Quat, Vec3};
use mint::Vector3;
use stardust_xr_fusion::{
	client::FrameInfo,
	core::values::Transform,
	fields::Field,
	input::{
		action::{BaseInputAction, InputAction, InputActionHandler},
		InputData, InputDataType, InputHandler,
	},
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};
use tracing::{debug, trace};

/// Linear drag is in m/s, angular drag is in rad/s.
#[derive(Debug, Clone, Copy)]
pub struct MomentumSettings {
	/// Drag (unity style) for momentum.
	pub drag: f32,
	/// Minimum speed before momentum applies.
	pub threshold: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GrabData {
	/// Max distance that you can be to start grabbing
	pub max_distance: f32,
	/// None means no linear momentum.
	pub linear_momentum: Option<MomentumSettings>,
	/// None means no angular momentum.
	pub angular_momentum: Option<MomentumSettings>,
}
impl Default for GrabData {
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
		}
	}
}

pub struct Grabbable {
	root: Spatial,
	content_parent: Spatial,
	global_action: BaseInputAction<GrabData>,
	condition_action: BaseInputAction<GrabData>,
	grab_action: SingleActorAction<GrabData>,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<GrabData>>,
	pointer_distance: f32,
	min_distance: f32,
	settings: GrabData,
	prev_pose: (Vec3, Quat),
	pose: (Vec3, Quat),
	linear_velocity: Option<Vec3>,
	angular_velocity: Option<(Vec3, f32)>,
}
impl Grabbable {
	pub fn create<Fi: Field>(
		content_space: &Spatial,
		content_transform: Transform,
		field: &Fi,
		settings: GrabData,
	) -> Result<Self, NodeError> {
		let global_action = BaseInputAction::new(false, |_, _| true);
		let condition_action = BaseInputAction::new(false, |input, data: &GrabData| {
			input.distance < data.max_distance
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
		let input_handler = InputHandler::create(
			content_space.client()?.get_root(),
			Transform::default(),
			field,
		)?
		.wrap(InputActionHandler::new(settings))?;
		let root = Spatial::create(input_handler.node(), Transform::default(), false)?;
		let content_parent = Spatial::create(input_handler.node(), Transform::default(), true)?;
		content_parent.set_transform(Some(content_space), content_transform)?;

		Ok(Grabbable {
			root,
			content_parent,
			global_action,
			condition_action,
			grab_action,
			input_handler,
			min_distance: f32::MAX,
			pointer_distance: 0.0,
			settings,
			prev_pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),
			pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),
			linear_velocity: None,
			angular_velocity: None,
		})
	}
	pub fn update(&mut self, info: &FrameInfo) -> Result<(), NodeError> {
		self.input_handler.lock_wrapped().update_actions([
			self.global_action.type_erase(),
			self.condition_action.type_erase(),
			self.grab_action.type_erase(),
		]);
		self.grab_action.update(&mut self.condition_action);

		if self.grab_action.actor_started() {
			self.content_parent
				.set_spatial_parent_in_place(self.input_handler.node())?;
			let actor = self.grab_action.actor().unwrap();
			if let InputDataType::Pointer(pointer) = &actor.input {
				self.pointer_distance =
					Vec3::from(pointer.origin).distance(pointer.deepest_point.into());
			}
		}

		if let Some(actor) = self.grab_action.actor().cloned() {
			let (position, rotation) = self.input_position_rotation(&actor);
			debug!(?position, ?rotation, uid = actor.uid, "Currently grabbing");

			self.root.set_transform(
				Some(self.input_handler.node()),
				Transform::from_position_rotation(position, rotation),
			)?;

			self.prev_pose = self.pose;
			self.pose = (position, rotation);

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
		}
		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");
			self.content_parent.set_zoneable(true)?;
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

		self.min_distance = self
			.global_action
			.actively_acting
			.iter()
			.map(|data| data.distance)
			.reduce(|a, b| a.min(b))
			.unwrap_or(f32::MAX);

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
					.with_data(|d| d.idx("scroll").as_vector().idx(1).as_f32());
				self.pointer_distance += scroll * 0.01;
				let grab_point =
					Vec3::from(p.origin) + (Vec3::from(p.direction()) * self.pointer_distance);
				(grab_point, p.orientation.into())
			}
			InputDataType::Tip(t) => (t.origin.into(), t.orientation.into()),
		}
	}
	const LINEAR_VELOCITY_STOP_THRESHOLD: f32 = 0.0001;
	fn apply_linear_momentum(&mut self, info: &FrameInfo, settings: MomentumSettings) {
		let Some(velocity) = &mut self.linear_velocity else {return};
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
		let Some((axis, angle)) = &mut self.angular_velocity else {return};
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
	pub fn min_distance(&self) -> f32 {
		self.min_distance
	}

	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input_handler.node().set_enabled(enabled)
	}
}
