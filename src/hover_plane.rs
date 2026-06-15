use crate::{
	DebugSettings, VisualDebug,
	input_action::{DeltaSet, InputQueue, InputSnapshot, PointerExt, SingleAction},
	lines::{self, LineExt},
};
use glam::{FloatExt, Mat4, Vec3, vec3};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	drawable::{Line, LinePoint, Lines, LinesExt},
	fields::{Field, FieldExt, Shape},
	spatial::{Spatial, SpatialExt, SpatialRef, Transform},
	suis::InputDataType,
	types::{Color, rgba_linear},
};
use std::{ops::Range, sync::Arc};

#[derive(Debug, Clone)]
pub struct HoverPlaneSettings {
	pub distance_range: Range<f32>,
	pub line_start_thickness: f32,
	pub line_start_color_hover: Color,
	pub line_start_color_interact: Color,
	pub line_end_thickness: f32,
	pub line_end_color_hover: Color,
	pub line_end_color_interact: Color,
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
	field_spatial: Spatial,
	field: Field,
	input: InputQueue,
	interact: SingleAction,
	size: [f32; 2],
	pub x_range: Range<f32>,
	pub y_range: Range<f32>,
	thickness: f32,
	settings: HoverPlaneSettings,
	lines: Lines,
	debug_lines: Lines,
}
impl HoverPlane {
	#[allow(clippy::too_many_arguments)]
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		parent: &SpatialRef,
		transform: Transform,
		size: [f32; 2],
		thickness: f32,
		x_range: Range<f32>,
		y_range: Range<f32>,
		settings: HoverPlaneSettings,
	) -> Result<Self> {
		let (root, root_ref) = Spatial::new(client, parent, transform).await?;
		let (field_spatial, _) = Spatial::new(
			client,
			&root_ref,
			Transform::from_translation([0.0, 0.0, thickness * -0.5]),
		)
		.await?;
		let (field, _) = Field::new(
			client,
			&field_spatial,
			Shape::Box {
				size: [size[0], size[1], thickness].into(),
			},
		)
		.await?;
		let input = InputQueue::new(client, root.clone(), field.clone(), root_ref.clone()).await?;
		let lines = Lines::new(client, &root, vec![]).await?;
		let debug_lines = Lines::new(client, &root, vec![]).await?;

		Ok(HoverPlane {
			root,
			field_spatial,
			field,
			input,
			interact: SingleAction::default(),
			size,
			x_range,
			y_range,
			thickness,
			settings,
			lines,
			debug_lines,
		})
	}

	fn hover(size: [f32; 2], point: Vec3, front: bool) -> bool {
		point.x.abs() * 2.0 < size[0]
			&& point.y.abs() * 2.0 < size[1]
			&& point.z.is_sign_positive() == front
	}

	pub fn interact_point_local(snap: &InputSnapshot) -> Vec3 {
		match snap.input() {
			InputDataType::Pointer { data } => data.intersect_plane(vec3(0.0, 0.0, 1.0)),
			InputDataType::Hand { data } => {
				(Vec3::from(data.index.tip.pose.position)
					+ Vec3::from(data.thumb.tip.pose.position))
					* 0.5
			}
			InputDataType::Tip { data } => Vec3::from(data.pose.position),
		}
	}

	pub fn interact_point(&self, snap: &InputSnapshot) -> ([f32; 2], f32) {
		let p = Self::interact_point_local(snap);
		let x = p.x.clamp(-self.size[0] / 2.0, self.size[0] / 2.0).remap(
			-self.size[0] / 2.0,
			self.size[0] / 2.0,
			self.x_range.start,
			self.x_range.end,
		);
		let y = p.y.clamp(-self.size[1] / 2.0, self.size[1] / 2.0).remap(
			self.size[1] / 2.0,
			-self.size[1] / 2.0,
			self.y_range.start,
			self.y_range.end,
		);
		([x, y], p.z)
	}

	pub fn input_to_points<'a>(
		&self,
		snaps: impl Iterator<Item = &'a Arc<InputSnapshot>>,
	) -> Vec<[f32; 2]> {
		snaps.map(|s| self.interact_point(s).0).collect()
	}

	pub fn root(&self) -> &Spatial {
		&self.root
	}
	pub fn input_queue(&self) -> &InputQueue {
		&self.input
	}
	pub fn field(&self) -> &Field {
		&self.field
	}

	pub fn set_size(&mut self, size: [f32; 2]) {
		self.size = size;
		let _ = self.field.set_shape(Shape::Box {
			size: [size[0], size[1], self.thickness].into(),
		});
	}
	pub fn set_thickness(&mut self, thickness: f32) {
		self.thickness = thickness;
		let _ = self
			.field_spatial
			.set_local_transform(Transform::from_translation([0.0, 0.0, thickness * -0.5]));
		let _ = self.field.set_shape(Shape::Box {
			size: [self.size[0], self.size[1], thickness].into(),
		});
	}

	pub fn hovering(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		self.interact.hovering()
	}
	pub fn current_hover_points(&self) -> Vec<[f32; 2]> {
		self.input_to_points(self.hovering().current().iter())
	}
	pub fn interact_status(&self) -> &SingleAction {
		&self.interact
	}

	pub fn update(&mut self) {
		self.input.handle_events();
		let size = self.size;
		let distance_range = self.settings.distance_range.clone();
		self.interact.update(
			false,
			&self.input,
			|snap| match snap.input() {
				InputDataType::Pointer { .. } => snap.distance() <= 0.0,
				_ => {
					let p = Self::interact_point_local(snap);
					distance_range.contains(&p.z.abs()) && Self::hover(size, p, true)
				}
			},
			|snap| match snap.input() {
				InputDataType::Hand { .. } => snap.datamap_f32("pinch_strength") > 0.95,
				_ => snap.datamap_f32("select") > 0.9,
			},
		);

		let mut lines: Vec<Line> = self
			.hovering()
			.current()
			.iter()
			.filter_map(|s| self.line_from_snap(s, false))
			.collect();
		if let Some(actor) = self.interact.actor()
			&& let Some(line) = self.line_from_snap(actor, true)
		{
			lines.push(line);
		}
		let _ = self.lines.set_lines(lines);
	}

	fn line_from_snap(&self, snap: &InputSnapshot, interacting: bool) -> Option<Line> {
		if let InputDataType::Pointer { .. } = snap.input() {
			None
		} else {
			Some(self.line_from_point(Self::interact_point_local(snap), interacting))
		}
	}
	fn line_from_point(&self, point: Vec3, interacting: bool) -> Line {
		Line {
			points: vec![
				LinePoint {
					point: [
						point.x.clamp(self.size[0] * -0.5, self.size[0] * 0.5),
						point.y.clamp(self.size[1] * -0.5, self.size[1] * 0.5),
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
		let lines = if let Some(settings) = settings {
			let line_front = lines::rounded_rectangle(
				self.size[0],
				self.size[1],
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
			vec![line_front, line_back]
		} else {
			vec![]
		};
		let _ = self.debug_lines.set_lines(lines);
	}
}
