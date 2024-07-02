use crate::{
	input_action::{DeltaSet, InputQueue, InputQueueable, SingleAction},
	lines::{self, LineExt},
	DebugSettings, VisualDebug,
};
use glam::{vec3, Mat4, Vec3};
use map_range::MapRange;
use stardust_xr_fusion::{
	core::values::{
		color::{color_space::LinearRgb, rgba_linear, Rgba},
		Vector2, Vector3,
	},
	drawable::{Line, LinePoint, Lines, LinesAspect},
	fields::{Field, FieldAspect, Shape},
	input::{InputData, InputDataType, InputHandler},
	node::{NodeError, NodeType},
	spatial::{Spatial, SpatialAspect, SpatialRefAspect, Transform},
};
use std::{ops::Range, sync::Arc};

#[derive(Debug, Clone)]
pub struct HoverPlaneSettings {
	pub distance_range: Range<f32>,
	pub line_start_thickness: f32,
	pub line_start_color_hover: Rgba<f32, LinearRgb>,
	pub line_start_color_interact: Rgba<f32, LinearRgb>,
	pub line_end_thickness: f32,
	pub line_end_color_hover: Rgba<f32, LinearRgb>,
	pub line_end_color_interact: Rgba<f32, LinearRgb>,
}
impl Default for HoverPlaneSettings {
	fn default() -> Self {
		HoverPlaneSettings {
			distance_range: 0.025..f32::MAX,
			line_start_thickness: 0.0,
			line_start_color_hover: rgba_linear!(1.0, 1.0, 1.0, 1.0),
			line_start_color_interact: rgba_linear!(0.0, 1.0, 0.75, 1.0),
			line_end_thickness: 0.005,
			line_end_color_hover: rgba_linear!(1.0, 1.0, 1.0, 0.0),
			line_end_color_interact: rgba_linear!(0.0, 1.0, 0.75, 0.0),
		}
	}
}

pub struct HoverPlane {
	root: Spatial,
	input: InputQueue,
	field: Field,
	interact: SingleAction,
	size: Vector2<f32>,
	pub x_range: Range<f32>,
	pub y_range: Range<f32>,
	thickness: f32,
	settings: HoverPlaneSettings,
	lines: Lines,
	debug_lines: Option<Lines>,
}
impl HoverPlane {
	pub fn create(
		parent: &impl SpatialRefAspect,
		transform: Transform,
		size: impl Into<Vector2<f32>>,
		thickness: f32,
		x_range: Range<f32>,
		y_range: Range<f32>,
		settings: HoverPlaneSettings,
	) -> Result<Self, NodeError> {
		let size = size.into();
		let root = Spatial::create(parent, transform, false)?;
		let field = Field::create(
			&root,
			Transform::from_translation([0.0, 0.0, thickness * -0.5]),
			Shape::Box([size.x, size.y, thickness].into()),
		)?;
		let input = InputHandler::create(&root, Transform::none(), &field)?.queue()?;

		let interact_action = SingleAction::default();

		let lines = Lines::create(&root, Transform::identity(), &[])?;
		Ok(HoverPlane {
			root,
			input,
			field,
			interact: interact_action,
			size,
			x_range,
			y_range,
			thickness,
			settings,
			lines,
			debug_lines: None,
		})
	}

	fn hover(size: Vector2<f32>, point: Vector3<f32>, front: bool) -> bool {
		point.x.abs() * 2.0 < size.x
			&& point.y.abs() * 2.0 < size.y
			&& point.z.is_sign_positive() == front
	}
	pub fn interact_point_local(input: &InputData) -> Vec3 {
		match &input.input {
			InputDataType::Pointer(p) => {
				let normal = vec3(0.0, 0.0, 1.0);
				let denom = normal.dot(p.direction().into());
				let t = -Vec3::from(p.origin).dot(normal) / denom;
				Vec3::from(p.origin) + Vec3::from(p.direction()) * t
			}
			InputDataType::Hand(h) => {
				(Vec3::from(h.index.tip.position) + Vec3::from(h.thumb.tip.position)) * 0.5
			}
			InputDataType::Tip(t) => t.origin.into(),
		}
	}
	pub fn interact_point(&self, input: &InputData) -> (Vector2<f32>, f32) {
		let interact_point = Self::interact_point_local(input);

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
	pub fn input_handler(&self) -> &InputHandler {
		self.input.handler()
	}
	pub fn field(&self) -> &Field {
		&self.field
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

	/// Get all the raw inputs that are hovering
	pub fn hovering(&self) -> &DeltaSet<Arc<InputData>> {
		self.interact.hovering()
	}
	/// Get all the points hovering over the surface, in x_range and y_range
	pub fn current_hover_points(&self) -> Vec<Vector2<f32>> {
		self.input_to_points(self.hovering().current().iter())
	}

	/// Get the input that's interacting
	pub fn interact_status(&self) -> &SingleAction {
		&self.interact
	}

	/// Set whether this will receive input or not
	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.handler().set_enabled(enabled)
	}

	/// Update the state of this touch plane. Run once every frame.
	pub fn update(&mut self) {
		self.interact.update(
			false,
			&self.input,
			|input| match &input.input {
				InputDataType::Pointer(_) => input.distance <= 0.0,
				_ => {
					let interact_point = Self::interact_point_local(input);
					self.settings
						.distance_range
						.contains(&interact_point.z.abs())
						&& Self::hover(self.size, interact_point.into(), true)
				}
			},
			|input| match &input.input {
				InputDataType::Hand(_) => input
					.datamap
					.with_data(|d| d.idx("pinch_strength").as_f32() > 0.95),
				_ => input.datamap.with_data(|d| d.idx("select").as_f32() > 0.9),
			},
		);

		let mut hovered_lines = self
			.hovering()
			.current()
			.iter()
			.filter_map(|i| self.line_from_input(i, false))
			.collect::<Vec<_>>();
		if let Some(input) = self.interact.actor().cloned() {
			if let Some(line) = self.line_from_input(&input, true) {
				hovered_lines.push(line);
			}
		}
		self.lines.set_lines(&hovered_lines).unwrap();
	}

	fn line_from_input(&self, input: &InputData, interacting: bool) -> Option<Line> {
		if let InputDataType::Pointer(_) = &input.input {
			None
		} else {
			Some(self.line_from_point(Self::interact_point_local(input), interacting))
		}
	}
	fn line_from_point(&self, point: Vec3, interacting: bool) -> Line {
		Line {
			points: vec![
				LinePoint {
					point: [
						point.x.clamp(self.size.x * -0.5, self.size.x * 0.5),
						point.y.clamp(self.size.y * -0.5, self.size.y * 0.5),
						0.0,
					]
					.into(),
					thickness: self.settings.line_start_thickness,
					color: if interacting {
						self.settings.line_start_color_interact
					} else {
						self.settings.line_start_color_hover
					},
				},
				LinePoint {
					point: point.into(),
					thickness: self.settings.line_end_thickness,
					color: if interacting {
						self.settings.line_end_color_interact
					} else {
						self.settings.line_end_color_hover
					},
				},
			],
			cyclic: false,
		}
	}
}
impl VisualDebug for HoverPlane {
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
