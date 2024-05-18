use crate::{
	input_action::{DeltaSet, InputQueue, InputQueueable, MultiActorAction},
	lines::{self, LineExt},
	DebugSettings, VisualDebug,
};
use glam::{vec3, Mat4, Vec3};
use map_range::MapRange;
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	core::values::{color::rgba_linear, Vector2, Vector3},
	drawable::Lines,
	fields::{BoxField, BoxFieldAspect, Field},
	input::{InputData, InputDataType, InputHandler, InputMethodRefAspect},
	node::{NodeError, NodeType},
	spatial::{Spatial, SpatialAspect, Transform},
};
use std::{ops::Range, sync::Arc};

enum InputState {
	None,
	Hovering,
	Touching { confirmed: bool },
}

pub struct TouchPlane {
	size: Vector2<f32>,
	pub x_range: Range<f32>,
	pub y_range: Range<f32>,
	thickness: f32,

	root: Spatial,
	input: InputQueue,
	field: BoxField,
	hover_action: MultiActorAction,
	touch_action: MultiActorAction,
	input_states: FxHashMap<String, (Arc<InputData>, InputState)>,
	hovering: DeltaSet<Arc<InputData>>,
	touching: DeltaSet<Arc<InputData>>,

	debug_lines: Option<Lines>,
}
impl TouchPlane {
	pub fn create(
		parent: &impl SpatialAspect,
		transform: Transform,
		size: impl Into<Vector2<f32>>,
		thickness: f32,
		x_range: Range<f32>,
		y_range: Range<f32>,
	) -> Result<Self, NodeError> {
		let size = size.into();
		let root = Spatial::create(parent, transform, false)?;
		let field = BoxField::create(
			&root,
			Transform::from_translation([0.0, 0.0, thickness * -0.5]),
			[size.x, size.y, thickness],
		)?;
		let input = InputHandler::create(&root, Transform::none(), &field)?.queue()?;

		Ok(TouchPlane {
			size,
			x_range,
			y_range,
			thickness,

			root,
			input,
			field,
			hover_action: Default::default(),
			touch_action: Default::default(),
			input_states: Default::default(),
			hovering: Default::default(),
			touching: Default::default(),
			debug_lines: None,
		})
	}

	fn hover(size: Vector2<f32>, point: Vector3<f32>, front: bool) -> bool {
		point.z.is_sign_positive() == front
			&& point.x.abs() * 2.0 < size.x
			&& point.y.abs() * 2.0 < size.y
	}
	/// Update the state of this touch plane. Run once every frame.
	pub fn update(&mut self) {
		self.hover_action
			.update(false, &self.input, |input| match &input.input {
				InputDataType::Pointer(_) => input.distance < 0.0,
				InputDataType::Hand(h) => Self::hover(self.size, h.index.tip.position, true),
				InputDataType::Tip(t) => Self::hover(self.size, t.origin, true),
			});
		self.touch_action
			.update(false, &self.input, |input| match &input.input {
				InputDataType::Pointer(_) => {
					input.datamap.with_data(|d| d.idx("select").as_f32() > 0.5)
				}
				InputDataType::Hand(h) => Self::hover(self.size, h.index.tip.position, false),
				InputDataType::Tip(t) => Self::hover(self.size, t.origin, false),
			});

		// add all the newly hovered stuff to the input states
		for (data, _) in self.input.input() {
			if !self.input_states.contains_key(&data.uid) {
				self.input_states
					.insert(data.uid.clone(), (data, InputState::None));
			}
		}

		// update all the input data
		let input = self.input.input();
		for (_, (data, state)) in &mut self.input_states {
			let Some((new_data, method)) = input.get_key_value(data) else {
				continue;
			};
			*data = new_data.clone();
			match state {
				InputState::None => {
					if self.hover_action.started_acting().contains(data) {
						*state = InputState::Hovering;
					}
				}
				InputState::Hovering => {
					if self.touch_action.started_acting().contains(data) {
						let _ = method.request_capture(self.input.handler());
						*state = InputState::Touching { confirmed: false };
					} else if self.hover_action.stopped_acting().contains(data) {
						*state = InputState::None;
					}
				}
				InputState::Touching { confirmed } => {
					let _ = method.request_capture(self.input.handler());
					*confirmed = data.captured;
					// transition state back
					if self.touch_action.stopped_acting().contains(data) {
						if self.hover_action.currently_acting().contains(data) {
							*state = InputState::Hovering;
						} else {
							*state = InputState::None;
						}
					}
				}
			}
		}

		self.hovering.push_new(
			self.input_states
				.values()
				.filter(|(_, state)| match state {
					InputState::Hovering => true,
					_ => false,
				})
				.map(|(data, _)| data.clone()),
		);
		self.touching.push_new(
			self.input_states
				.values()
				.filter(|(_, state)| match state {
					InputState::Touching { confirmed } => *confirmed,
					_ => false,
				})
				.map(|(data, _)| data.clone()),
		);
	}
	pub fn interact_point(&self, input: &InputData) -> (Vector2<f32>, f32) {
		let interact_point = match &input.input {
			InputDataType::Pointer(p) => {
				let normal = vec3(0.0, 0.0, 1.0);
				let denom = normal.dot(p.direction().into());
				let t = -Vec3::from(p.origin).dot(normal) / denom;
				(Vec3::from(p.origin) + Vec3::from(p.direction()) * t).into()
			}
			InputDataType::Hand(h) => h.index.tip.position,
			InputDataType::Tip(t) => t.origin,
		};

		let x = interact_point
			.x
			.clamp(-self.size.x / 2.0, self.size.x / 2.0)
			.map_range(-self.size.x / 2.0..self.size.x / 2.0, self.x_range.clone());
		let y = interact_point
			.y
			.clamp(-self.size.y / 2.0, self.size.y / 2.0)
			.map_range(self.size.y / 2.0..-self.size.y / 2.0, self.y_range.clone());

		([x, y].into(), interact_point.z)
	}
	pub fn input_to_points<'a>(
		&self,
		inputs: impl Iterator<Item = &'a Arc<InputData>>,
	) -> Vec<Vector2<f32>> {
		inputs.map(|i| self.interact_point(i).0).collect()
	}

	pub fn root(&self) -> &Spatial {
		&self.root
	}
	pub fn field(&self) -> Field {
		Field::alias_field(&self.field)
	}

	pub fn set_size(&mut self, size: impl Into<Vector2<f32>>) -> Result<(), NodeError> {
		let size = size.into();
		self.size = size;
		self.field.set_size([size.x, size.y, self.thickness])?;
		Ok(())
	}
	pub fn set_thickness(&mut self, thickness: f32) -> Result<(), NodeError> {
		self.thickness = thickness;
		self.field
			.set_local_transform(Transform::from_translation([0.0, 0.0, thickness * -0.5]))?;
		self.field.set_size([self.size.x, self.size.y, thickness])?;
		Ok(())
	}

	pub fn hovering(&self) -> &DeltaSet<Arc<InputData>> {
		&self.hovering
	}
	pub fn touching(&self) -> &DeltaSet<Arc<InputData>> {
		&self.touching
	}

	/// Set whether this will receive input or not
	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.handler().set_enabled(enabled)
	}
}
impl VisualDebug for TouchPlane {
	fn set_debug(&mut self, settings: Option<DebugSettings>) {
		self.debug_lines = settings.and_then(|settings| {
			let line_front = lines::rounded_rectangle(
				self.size.x,
				self.size.y,
				settings.line_thickness * 0.5,
				4,
			)
			.thickness(settings.line_thickness)
			.color(settings.line_color);
			let line_back = line_front
				.clone()
				.color(rgba_linear!(
					settings.line_color.c.r,
					settings.line_color.c.g,
					settings.line_color.c.b,
					settings.line_color.a * 0.5
				))
				.transform(Mat4::from_translation(vec3(0.0, 0.0, -self.thickness)));

			let lines = Lines::create(
				&self.root,
				Transform::from_translation([0.0, 0.0, 0.0]),
				&[line_front, line_back],
			)
			.ok()?;
			Some(lines)
		})
	}
}
