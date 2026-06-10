use crate::{
	FrameSensitive, UIElement, VisualDebug,
	input_action::{InputQueue, InputSnapshot, SingleAction, grab_pinch_interact},
	lines::{LineExt, axes, bounding_box},
};
use glam::{Affine3A, Quat, Vec3, vec3};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler, FrameInfo},
	drawable::{Lines, LinesExt},
	fields::Field,
	spatial::{Spatial, SpatialExt, SpatialRef, Transform},
	suis::InputDataType,
};
use std::f32::consts::PI;
use tracing::{debug, trace};

fn swing_direction(direction: Vec3) -> Quat {
	let pitch = direction.y.asin();
	let yaw = direction.z.atan2(direction.x);
	Quat::from_rotation_y(-yaw - PI / 2.0) * Quat::from_rotation_x(pitch)
}

#[derive(Debug, Clone, Copy)]
pub enum PointerMode {
	Parent,
	Align,
	Move,
}

#[derive(Debug, Clone, Copy)]
pub struct MomentumSettings {
	pub drag: f32,
	pub threshold: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GrabbableSettings {
	pub max_distance: f32,
	pub linear_momentum: Option<MomentumSettings>,
	pub angular_momentum: Option<MomentumSettings>,
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
			pointer_mode: PointerMode::Parent,
		}
	}
}

pub struct Grabbable {
	parent: SpatialRef,
	content_parent: Spatial,
	field: Field,
	input: InputQueue,
	grab_action: SingleAction,

	content_lines: Lines,
	root_lines: Lines,
	pub settings: GrabbableSettings,

	prev_pose: Affine3A,
	relative_transform: Affine3A,
	pose: Affine3A,

	linear_velocity: Option<Vec3>,
	angular_velocity: Option<(Vec3, f32)>,
}
impl Grabbable {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		parent: SpatialRef,
		content_transform: Transform,
		field: Field,
		settings: GrabbableSettings,
	) -> Result<Self> {
		let (content_parent, _) = Spatial::create(client, &parent, content_transform).await?;
		let input = InputQueue::new(
			client,
			content_parent.clone(),
			field.clone(),
			parent.clone(),
		)
		.await?;
		let content_lines = Lines::create(client, &content_parent, vec![]).await?;
		let root_lines = Lines::create(client, &content_parent, vec![]).await?;

		Ok(Grabbable {
			parent,
			content_parent,
			field,
			input,
			grab_action: SingleAction::default(),
			content_lines,
			root_lines,
			settings,
			prev_pose: Affine3A::IDENTITY,
			relative_transform: Affine3A::IDENTITY,
			pose: Affine3A::IDENTITY,
			linear_velocity: None,
			angular_velocity: None,
		})
	}

	const LINEAR_VELOCITY_STOP_THRESHOLD: f32 = 0.001;
	fn apply_linear_momentum(&mut self, info: &FrameInfo, settings: MomentumSettings) {
		let Some(velocity) = &mut self.linear_velocity else {
			return;
		};
		let delta = info.delta;
		if velocity.length_squared() < Self::LINEAR_VELOCITY_STOP_THRESHOLD {
			self.linear_velocity.take();
		} else {
			*velocity *= (1.0 - settings.drag * delta).clamp(0.0, 1.0);
			self.pose = Affine3A::from_translation(*velocity * delta) * self.pose;
			trace!(?velocity, "linear momentum");
		}
	}
	const ANGULAR_VELOCITY_STOP_THRESHOLD: f32 = 0.001;
	fn apply_angular_momentum(&mut self, info: &FrameInfo, settings: MomentumSettings) {
		let Some((axis, angle)) = &mut self.angular_velocity else {
			return;
		};
		let delta = info.delta;
		if *angle < Self::ANGULAR_VELOCITY_STOP_THRESHOLD {
			self.angular_velocity.take();
		} else {
			*angle *= (1.0 - settings.drag * delta).clamp(0.0, 1.0);
			self.pose = Affine3A::from_rotation_translation(
				Quat::from_axis_angle(*axis, *angle * delta),
				Vec3::ZERO,
			) * self.pose;
			trace!(?axis, angle, "angular momentum");
		}
	}

	pub fn linear_velocity(&self) -> Option<Vec3> {
		self.linear_velocity
	}
	pub fn linear_speed(&self) -> Option<f32> {
		self.linear_velocity.map(|v| v.length())
	}
	pub fn cancel_linear_velocity(&mut self) {
		self.linear_velocity.take();
	}
	pub fn just_stopped_moving(&self) -> bool {
		!self.grab_action.actor_acting()
			&& self
				.linear_velocity
				.is_some_and(|v| v.length_squared() < Self::LINEAR_VELOCITY_STOP_THRESHOLD)
	}
	pub fn angular_velocity(&self) -> Option<(Vec3, f32)> {
		self.angular_velocity
	}
	pub fn cancel_angular_velocity(&mut self) {
		self.angular_velocity.take();
	}
	pub fn just_stopped_rotating(&self) -> bool {
		!self.grab_action.actor_acting()
			&& self
				.angular_velocity
				.is_some_and(|(_, a)| a < Self::ANGULAR_VELOCITY_STOP_THRESHOLD)
	}

	pub fn field(&self) -> &Field {
		&self.field
	}
	pub fn grab_action(&self) -> &SingleAction {
		&self.grab_action
	}
	pub fn content_parent(&self) -> &Spatial {
		&self.content_parent
	}

	pub fn pose(&self) -> (Vec3, Quat) {
		let (_, rot, pos) = self.pose.to_scale_rotation_translation();
		(pos, rot)
	}
	pub fn set_pose(&mut self, pos: Vec3, rot: Quat) {
		self.pose = Affine3A::from_rotation_translation(rot, pos);
		let _ = self.content_parent.set_relative_transform(
			self.parent.clone(),
			Transform::from_translation_rotation(pos, rot),
		);
	}
}
impl UIElement for Grabbable {
	fn handle_events(&mut self) -> bool {
		if !self.input.handle_events() {
			return false;
		}
		let max_distance = self.settings.max_distance;
		self.grab_action.update(
			true,
			&self.input,
			|snap| match snap.input() {
				InputDataType::Hand { data } => {
					data.thumb.tip.distance < max_distance && data.index.tip.distance < max_distance
				}
				_ => snap.distance() < max_distance,
			},
			grab_pinch_interact,
		);

		let start_grabbing = self.grab_action.actor_started();
		if start_grabbing || self.grab_action.actor_changed() {
			let actor = self.grab_action.actor().unwrap();
			let grab_position = snap_grab_position(actor);
			let grab_rotation = snap_grab_rotation(actor);
			let grab_pose = Affine3A::from_rotation_translation(grab_rotation, grab_position);
			self.relative_transform = grab_pose.inverse() * self.pose;
			self.prev_pose = self.pose;
		}

		if let Some(actor) = self.grab_action.actor().cloned() {
			if matches!(actor.input(), InputDataType::Pointer { .. }) {
				let scroll_amount = actor.datamap_vec2("scroll_continuous").y * 0.01
					+ actor.datamap_vec2("scroll_discrete").y * 0.01;
				let offset = Affine3A::from_translation(vec3(0.0, 0.0, scroll_amount));
				self.relative_transform = offset * self.relative_transform;
			}

			let grab_position = snap_grab_position(&actor);
			let grab_rotation = snap_grab_rotation(&actor);
			let current_grab_pose =
				Affine3A::from_rotation_translation(grab_rotation, grab_position);

			self.pose = match (actor.input(), self.settings.pointer_mode) {
				(InputDataType::Pointer { data }, PointerMode::Align) => {
					let parent_pose = current_grab_pose * self.relative_transform;
					let (_, _, parent_translation) = parent_pose.to_scale_rotation_translation();
					let swing_rotation = swing_direction(Vec3::from(data.direction()));
					Affine3A::from_rotation_translation(swing_rotation, parent_translation)
				}
				(InputDataType::Pointer { .. }, PointerMode::Move) => {
					let parent_pose = current_grab_pose * self.relative_transform;
					let offset_rotation = parent_pose.to_scale_rotation_translation().1
						* self.pose.to_scale_rotation_translation().1.inverse();
					parent_pose * Affine3A::from_quat(offset_rotation.inverse())
				}
				_ => current_grab_pose * self.relative_transform,
			};

			let (_, new_rotation, new_position) = self.pose.to_scale_rotation_translation();
			let _ = self.content_parent.set_relative_transform(
				self.parent.clone(),
				Transform::from_translation_rotation(new_position, new_rotation),
			);
		}

		if start_grabbing {
			debug!("Started grabbing");
		}

		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");
			self.relative_transform = Affine3A::IDENTITY;
		}
		true
	}
}
impl FrameSensitive for Grabbable {
	fn frame(&mut self, info: &FrameInfo) {
		if self.grab_action.actor_acting() {
			let delta = info.delta;
			let velocity = self.pose * self.prev_pose.inverse();
			let (_, angular_velocity, linear_velocity) = velocity.to_scale_rotation_translation();
			if let Some(momentum_settings) = &self.settings.linear_momentum {
				let above_threshold =
					linear_velocity.length_squared() > momentum_settings.threshold.powf(2.0);
				self.linear_velocity = above_threshold.then(|| linear_velocity / delta);
			}
			if let Some(momentum_settings) = &self.settings.angular_momentum {
				let (axis, angle) = angular_velocity.to_axis_angle();
				let above_threshold = angle > momentum_settings.threshold;
				self.angular_velocity = above_threshold.then(|| (axis, angle / delta));
			}
			self.prev_pose = self.pose;
		}
		if !self.grab_action.actor_acting() {
			if let Some(settings) = self.settings.linear_momentum {
				self.apply_linear_momentum(info, settings);
			}
			if let Some(settings) = self.settings.angular_momentum {
				self.apply_angular_momentum(info, settings);
			}

			if self.linear_velocity.is_some() || self.angular_velocity.is_some() {
				self.prev_pose = self.pose;
				let (_, rotation, translation) = self.pose.to_scale_rotation_translation();
				let _ = self.content_parent.set_relative_transform(
					self.parent.clone(),
					Transform::from_translation_rotation(translation, rotation),
				);
			}
		}
	}
}
impl VisualDebug for Grabbable {
	fn set_debug(&mut self, settings: Option<crate::DebugSettings>) {
		if let Some(settings) = settings {
			let _ = self
				.root_lines
				.set_lines(axes(0.01, settings.line_thickness));
			let content_lines = self.content_lines.clone();
			let content_parent = self.content_parent.clone();
			tokio::task::spawn(async move {
				if let Ok(bounds) = content_parent.get_local_bounding_box().await {
					let _ = content_lines.set_lines(
						bounding_box(bounds)
							.into_iter()
							.map(|l| {
								l.color(settings.line_color)
									.thickness(settings.line_thickness)
							})
							.collect::<Vec<_>>(),
					);
				}
			});
		} else {
			let _ = self.content_lines.set_lines(vec![]);
			let _ = self.root_lines.set_lines(vec![]);
		}
	}
}

fn snap_grab_position(snap: &InputSnapshot) -> Vec3 {
	match snap.input() {
		InputDataType::Pointer { data } => Vec3::from(data.pose.position),
		InputDataType::Hand { data } => Vec3::from(data.palm.pose.position),
		InputDataType::Tip { data } => Vec3::from(data.pose.position),
	}
}

fn snap_grab_rotation(snap: &InputSnapshot) -> Quat {
	match snap.input() {
		InputDataType::Pointer { data } => Quat::from(data.pose.orientation),
		InputDataType::Hand { data } => Quat::from(data.palm.pose.orientation),
		InputDataType::Tip { data } => Quat::from(data.pose.orientation),
	}
}
