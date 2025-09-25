use crate::{
	DebugSettings, UIElement, VisualDebug,
	input_action::{InputQueue, InputQueueable, MultiAction},
	lines::{self, LineExt},
};
use glam::{Mat4, Vec3, vec3};
use map_range::MapRange;
use stardust_xr_fusion::{
	core::values::{Vector2, Vector3, color::rgba_linear},
	drawable::Lines,
	fields::{Field, FieldAspect, Shape},
	input::{InputData, InputDataType, InputHandler},
	node::{NodeError, NodeType},
	spatial::{Spatial, SpatialAspect, SpatialRefAspect, Transform},
};
use std::{ops::Range, sync::Arc};

pub struct TouchPlane {
	size: Vector2<f32>,
	pub x_range: Range<f32>,
	pub y_range: Range<f32>,
	thickness: f32,

	root: Spatial,
	input: InputQueue,
	field: Field,
	action: MultiAction,

	debug_lines: Option<Lines>,
}
impl TouchPlane {
	pub fn create(
		parent: &impl SpatialRefAspect,
		transform: Transform,
		size: impl Into<Vector2<f32>>,
		thickness: f32,
		x_range: Range<f32>,
		y_range: Range<f32>,
	) -> Result<Self, NodeError> {
		let size = size.into();
		let root = Spatial::create(parent, transform, false)?;
		let field = Field::create(
			&root,
			Transform::from_translation([0.0, 0.0, thickness * -0.5]),
			Shape::Box([size.x, size.y, thickness].into()),
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
			action: Default::default(),
			debug_lines: None,
		})
	}

	fn hover(size: Vector2<f32>, point: Vector3<f32>, front: bool) -> bool {
		point.z.is_sign_positive() == front
			&& point.x.abs() * 2.0 < size.x
			&& point.y.abs() * 2.0 < size.y
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
	pub fn field(&self) -> &Field {
		&self.field
	}
	pub fn action(&self) -> &MultiAction {
		&self.action
	}

	pub fn set_size(&mut self, size: impl Into<Vector2<f32>>) -> Result<(), NodeError> {
		let size = size.into();
		self.size = size;
		self.field
			.set_shape(Shape::Box([size.x, size.y, self.thickness].into()))?;
		Ok(())
	}
	pub fn set_thickness(&mut self, thickness: f32) -> Result<(), NodeError> {
		self.thickness = thickness;
		self.field
			.set_local_transform(Transform::from_translation([0.0, 0.0, thickness * -0.5]))?;
		self.field
			.set_shape(Shape::Box([self.size.x, self.size.y, thickness].into()))?;
		Ok(())
	}

	/// Set whether this will receive input or not
	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.handler().set_enabled(enabled)
	}
}
impl UIElement for TouchPlane {
	fn handle_events(&mut self) -> bool {
		if !self.input.handle_events() {
			return false;
		}
		self.action.update(
			&self.input,
			|input| match &input.input {
				InputDataType::Pointer(_) => input.distance < 0.0,
				InputDataType::Hand(h) => Self::hover(self.size, h.index.tip.position, true),
				InputDataType::Tip(t) => Self::hover(self.size, t.origin, true),
			},
			|input| match &input.input {
				InputDataType::Pointer(_) => {
					input.datamap.with_data(|d| d.idx("select").as_f32() > 0.5)
				}
				InputDataType::Hand(h) => Self::hover(self.size, h.index.tip.position, false),
				InputDataType::Tip(t) => Self::hover(self.size, t.origin, false),
			},
		);
		true
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
