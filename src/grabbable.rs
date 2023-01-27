use crate::single_actor_action::SingleActorAction;
use glam::{vec3, Quat, Vec3};
use stardust_xr_fusion::{
	client::FrameInfo,
	core::values::Transform,
	fields::Field,
	input::{
		action::{BaseInputAction, InputAction, InputActionHandler},
		InputDataType, InputHandler,
	},
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};
use tracing::debug;

#[derive(Debug, Clone, Copy)]
pub struct GrabData {
	/// Max distance that you can be to start grabbing
	pub max_distance: f32,
	/// Should the object drift a bit after being let go?
	pub momentum: bool,
	/// Linear drag in m/s^2 for momentum
	pub linear_drag: f32,
	/// Angular drag in rad/s^2 for momentum
	pub angular_drag: f32,
}
impl Default for GrabData {
	fn default() -> Self {
		Self {
			max_distance: 0.05,
			momentum: true,
			linear_drag: 8.0,
			angular_drag: 15.0,
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
	min_distance: f32,
	settings: GrabData,
	prev_pose: (Vec3, Quat),
	pose: (Vec3, Quat),
	linear_velocity: Option<Vec3>,
	angular_velocity: Option<(Vec3, f32)>,
}
impl Grabbable {
	pub fn new<Fi: Field>(
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
			settings,
			prev_pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),
			pose: (vec3(0.0, 0.0, 0.0), Quat::IDENTITY),
			linear_velocity: None,
			angular_velocity: None,
		})
	}
	pub fn update(&mut self, info: &FrameInfo) {
		self.input_handler.lock_wrapped().update_actions([
			self.global_action.type_erase(),
			self.condition_action.type_erase(),
			self.grab_action.type_erase(),
		]);
		self.grab_action.update(&mut self.condition_action);

		if self.grab_action.actor_started() {
			self.content_parent
				.set_spatial_parent_in_place(self.input_handler.node())
				.unwrap();
		}

		if let Some(actor) = self.grab_action.actor() {
			let (position, rotation) = input_position_rotation(&actor.input);
			debug!(?position, ?rotation, uid = actor.uid, "Currently grabbing");

			self.root
				.set_transform(
					Some(self.input_handler.node()),
					Transform::from_position_rotation(position, rotation),
				)
				.unwrap();

			if self.settings.momentum {
				self.prev_pose = self.pose;
				self.pose = (position, rotation);

				let delta = info.delta as f32;
				let linear_velocity = self.pose.0 - self.prev_pose.0;
				self.linear_velocity.replace(linear_velocity / delta);

				let (axis, angle) = (self.pose.1 * self.prev_pose.1.inverse()).to_axis_angle();
				self.angular_velocity = Some((axis, angle / delta));
			}
		}

		if self.grab_action.actor_started() {
			debug!(
				uid = self.grab_action.actor().as_ref().unwrap().uid,
				"Started grabbing"
			);
			self.content_parent.set_zoneable(false).unwrap();
			self.content_parent
				.set_spatial_parent_in_place(&self.root)
				.unwrap();
		}
		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");
			self.content_parent.set_zoneable(true).unwrap();
		}

		if !self.grab_action.actor_acting() {
			self.apply_linear_momentum(info);
			self.apply_angular_momentum(info);

			if self.linear_velocity.is_some() || self.angular_velocity.is_some() {
				self.root
					.set_transform(
						Some(self.input_handler.node()),
						Transform::from_position_rotation(self.pose.0, self.pose.1),
					)
					.unwrap();
				println!();
			}
		}

		self.min_distance = self
			.global_action
			.actively_acting
			.iter()
			.map(|data| data.distance)
			.reduce(|a, b| a.min(b))
			.unwrap_or(f32::MAX);
	}
	fn apply_linear_momentum(&mut self, info: &FrameInfo) {
		let Some(velocity) = &mut self.linear_velocity else {return};
		let delta = info.delta as f32;
		let linear_drag = self.settings.linear_drag;
		if velocity.length_squared() < 0.0001 {
			self.linear_velocity.take();
		} else {
			*velocity *= (1.0 - linear_drag * delta).clamp(0.0, 1.0);
			self.pose.0 += *velocity * delta;
			println!("linear speed: {:.3}", velocity.length());
		}
	}
	fn apply_angular_momentum(&mut self, info: &FrameInfo) {
		let Some((axis, angle)) = &mut self.angular_velocity else {return};
		let delta = info.delta as f32;
		let angular_drag = self.settings.angular_drag;
		if *angle < 0.001 {
			self.angular_velocity.take();
		} else {
			*angle *= (1.0 - angular_drag * delta).clamp(0.0, 1.0);
			self.pose.1 *= Quat::from_axis_angle(*axis, *angle * delta);
			println!("angular speed: {:.3}", angle);
		}
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
}

fn input_position_rotation(input: &InputDataType) -> (Vec3, Quat) {
	match input {
		InputDataType::Hand(h) => (
			Vec3::from(h.thumb.tip.position).lerp(Vec3::from(h.index.tip.position), 0.5),
			h.palm.rotation.into(),
		),
		InputDataType::Pointer(p) => (p.origin.into(), p.orientation.into()),
		InputDataType::Tip(t) => (t.origin.into(), t.orientation.into()),
	}
}
