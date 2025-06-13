use crate::{
	input_action::{grab_pinch_interact, InputQueue, InputQueueable, SingleAction},
	lines::{axes, bounding_box, LineExt},
	FrameSensitive, UIElement, VisualDebug,
};
use glam::{vec3, Affine3A, Quat, Vec3};
use stardust_xr_fusion::{
	core::values::Vector3,
	drawable::{Lines, LinesAspect},
	fields::{Field, FieldRefAspect},
	input::{InputDataType, InputHandler},
	node::{NodeError, NodeType},
	root::FrameInfo,
	spatial::{Spatial, SpatialAspect, SpatialRefAspect, Transform},
};
use std::f32::consts::PI;
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

pub struct Grabbable {
	content_parent: Spatial,
	field: Field,
	input: InputQueue,
	grab_action: SingleAction,

	content_lines: Lines,
	root_lines: Lines,
	pub settings: GrabbableSettings,

	prev_pose: Affine3A,
	relative_transform: Affine3A, // Relative transform matrix during grab
	pub pose: Affine3A,

	closest_point_tx: mpsc::Sender<Vec3>,
	closest_point_rx: mpsc::Receiver<Vec3>,

	linear_velocity: Option<Vec3>,
	angular_velocity: Option<(Vec3, f32)>,
}
impl Grabbable {
	pub fn create(
		content_space: &impl SpatialRefAspect,
		content_transform: Transform,
		field: &Field,
		settings: GrabbableSettings,
	) -> Result<Self, NodeError> {
		let input = InputHandler::create(content_space, Transform::none(), field)?.queue()?;
		let content_parent =
			Spatial::create(input.handler(), content_transform, settings.zoneable)?;

		let content_lines = Lines::create(&content_parent, Transform::identity(), &[])?;
		let root_lines = Lines::create(content_space, Transform::identity(), &[])?;

		let (closest_point_tx, closest_point_rx) = mpsc::channel(1);
		Ok(Grabbable {
			content_parent,
			input,
			grab_action: SingleAction::default(),
			field: field.clone(),

			content_lines,
			root_lines,
			settings,

			prev_pose: Affine3A::IDENTITY,
			relative_transform: Affine3A::IDENTITY,
			pose: Affine3A::IDENTITY,

			closest_point_tx,
			closest_point_rx,

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

			// lets us slide the grabbable into a zone seamlessly
			self.content_parent
				.set_zoneable(self.settings.zoneable)
				.unwrap();
		} else {
			*velocity *= (1.0 - settings.drag * delta).clamp(0.0, 1.0);
			self.pose *= Affine3A::from_translation(*velocity * delta);
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
			self.pose *= Affine3A::from_rotation_translation(
				Quat::from_axis_angle(*axis, *angle * delta),
				Vec3::ZERO,
			);
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

	pub fn field(&self) -> &Field {
		&self.field
	}

	pub fn grab_action(&self) -> &SingleAction {
		&self.grab_action
	}
	pub fn content_parent(&self) -> &Spatial {
		&self.content_parent
	}

	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.handler().set_enabled(enabled)
	}
}
impl UIElement for Grabbable {
	fn handle_events(&mut self) -> bool {
		if !self.input.handle_events() {
			return false;
		}
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
			grab_pinch_interact,
		);

		if self.grab_action.actor_started() {
			// Calculate and store the relative transform matrix
			let actor = self.grab_action.actor().unwrap();
			let grab_position = match &actor.input {
				InputDataType::Pointer(p) => p.origin.into(),
				InputDataType::Hand(h) => h.palm.position.into(),
				InputDataType::Tip(t) => t.origin.into(),
			};
			let grab_rotation = match &actor.input {
				InputDataType::Pointer(p) => p.orientation.into(),
				InputDataType::Hand(h) => h.palm.rotation.into(),
				InputDataType::Tip(t) => t.orientation.into(),
			};
			let grab_pose_matrix =
				Affine3A::from_rotation_translation(grab_rotation, grab_position);

			self.relative_transform = grab_pose_matrix.inverse() * self.pose;
			self.prev_pose = self.pose;
		}

		if let Some(actor) = self.grab_action.actor().cloned() {
			if matches!(&actor.input, InputDataType::Pointer(_)) {
				let scroll_sensitivity = 0.01;
				let scroll_amount = actor.datamap.with_data(|datamap| {
					datamap.idx("scroll_continuous").as_vector().idx(1).as_f32() // Use the Y-axis for forward/backward scrolling
				});
				let offset =
					Affine3A::from_translation(vec3(0.0, 0.0, scroll_amount * -scroll_sensitivity));
				self.relative_transform = offset * self.relative_transform;
			}

			let grab_position = match &actor.input {
				InputDataType::Pointer(p) => p.origin.into(),
				InputDataType::Hand(h) => h.palm.position.into(),
				InputDataType::Tip(t) => t.origin.into(),
			};
			let grab_rotation = match &actor.input {
				InputDataType::Pointer(p) => p.orientation.into(),
				InputDataType::Hand(h) => h.palm.rotation.into(),
				InputDataType::Tip(t) => t.orientation.into(),
			};
			let current_grab_pose =
				Affine3A::from_rotation_translation(grab_rotation, grab_position);

			self.pose = match (&actor.input, self.settings.pointer_mode) {
				(InputDataType::Pointer(p), PointerMode::Align) => {
					let parent_pose = current_grab_pose * self.relative_transform;
					let (_, _, parent_translation) = parent_pose.to_scale_rotation_translation();
					let swing_rotation = swing_direction(p.direction().into());
					Affine3A::from_rotation_translation(swing_rotation, parent_translation)
				}
				(InputDataType::Pointer(_), PointerMode::Move) => {
					let parent_pose = current_grab_pose * self.relative_transform;
					let offset_rotation = parent_pose.to_scale_rotation_translation().1
						* self.pose.to_scale_rotation_translation().1.inverse();
					parent_pose * Affine3A::from_quat(offset_rotation.inverse())
				}
				(_, _) => current_grab_pose * self.relative_transform,
			};

			let (_, new_rotation, new_position) = self.pose.to_scale_rotation_translation();
			self.content_parent
				.set_local_transform(Transform::from_translation_rotation(
					new_position,
					new_rotation,
				))
				.unwrap();
		}

		if self.grab_action.actor_started() {
			debug!(
				id = self.grab_action.actor().as_ref().unwrap().id,
				"Started grabbing"
			);
			self.content_parent.set_zoneable(false).unwrap();

			'magnet: {
				if self.settings.magnet {
					// if we have magnet strength, store the closest point so we can lerp that to the grab point
					let grab_data = self.grab_action.actor().unwrap().clone();
					// pointers are just too unstable to magnet
					if let InputDataType::Pointer(_) = &grab_data.input {
						break 'magnet;
					}
					let field = self.field.clone();
					let input = self.input.handler().clone();
					let closest_point_tx = self.closest_point_tx.clone();
					tokio::task::spawn(async move {
						let result = field.closest_point(&input, [0.0; 3]).await.unwrap();
						// if let Ok(result) = result {
						let _ = closest_point_tx.send(result.into()).await;
						// }
					});
				}
			}
		}

		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");

			let _ = self.closest_point_rx.try_recv();
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
				self.content_parent
					.set_local_transform(Transform::from_translation_rotation(
						translation,
						rotation,
					))
					.unwrap();
			}
		}
	}
}
impl VisualDebug for Grabbable {
	fn set_debug(&mut self, settings: Option<crate::DebugSettings>) {
		if let Some(settings) = settings {
			let _ = self
				.root_lines
				.set_lines(&axes(0.01, settings.line_thickness));
			let content_lines = self.content_lines.clone();
			let content_parent = self.content_parent.clone();
			tokio::task::spawn(async move {
				if let Ok(bounds) = content_parent.get_local_bounding_box().await {
					let _ = content_lines.set_lines(
						&bounding_box(bounds)
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
			let _ = self.content_lines.set_lines(&[]);
			let _ = self.root_lines.set_lines(&[]);
		}
	}
}
