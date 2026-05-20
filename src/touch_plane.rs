use crate::{
	DebugSettings, UIElement, VisualDebug,
	input_action::{InputQueue, InputSnapshot, MultiAction},
	lines::{self, LineExt},
};
use glam::{FloatExt, Mat4, Vec3, vec3};
use gluon::Object;
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	drawable::{Lines, LinesExt},
	error::ServerError,
	fields::{Field, FieldExt, Shape},
	spatial::{Spatial, SpatialExt, SpatialRef, Transform},
	suis::InputDataType,
	types::rgba_linear,
};
use std::{ops::Range, sync::Arc};

pub struct TouchPlane {
	size: [f32; 2],
	pub x_range: Range<f32>,
	pub y_range: Range<f32>,
	thickness: f32,

	root: Spatial,
	field_spatial: Spatial,
	field: Field,
	input: Object<InputQueue>,
	action: MultiAction,

	debug_lines: Lines,
}
impl TouchPlane {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		parent: &SpatialRef,
		transform: Transform,
		size: [f32; 2],
		thickness: f32,
		x_range: Range<f32>,
		y_range: Range<f32>,
	) -> Result<Self, ServerError> {
		let root = Spatial::create(client, parent, transform).await?;
		let root_ref = root.spatial_ref().await?;
		let field_spatial = Spatial::create(
			client,
			&root_ref,
			Transform::from_translation([0.0, 0.0, thickness * -0.5]),
		)
		.await?;
		let field = Field::create(
			client,
			&field_spatial,
			Shape::Box {
				size: [size[0], size[1], thickness].into(),
			},
		)
		.await?;
		let input = InputQueue::new(client, root.clone(), field.clone(), root_ref.clone()).await?;
		let debug_lines = Lines::create(client, &root, vec![]).await?;

		Ok(TouchPlane {
			size,
			x_range,
			y_range,
			thickness,
			root,
			field_spatial,
			field,
			input,
			action: Default::default(),
			debug_lines,
		})
	}

	fn hover(size: [f32; 2], point: Vec3, front: bool) -> bool {
		point.z.is_sign_positive() == front
			&& point.x.abs() * 2.0 < size[0]
			&& point.y.abs() * 2.0 < size[1]
	}

	pub fn interact_point(&self, snap: &InputSnapshot) -> ([f32; 2], f32) {
		let p = match snap.input() {
			InputDataType::Pointer { data } => {
				let normal = vec3(0.0, 0.0, 1.0);
				let origin = Vec3::from(data.pose.position);
				let dir = Vec3::from(data.direction());
				let t = -origin.dot(normal) / normal.dot(dir);
				origin + dir * t
			}
			InputDataType::Hand { data } => Vec3::from(data.index.tip.pose.position),
			InputDataType::Tip { data } => Vec3::from(data.pose.position),
		};

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
	pub fn field(&self) -> &Field {
		&self.field
	}
	pub fn action(&self) -> &MultiAction {
		&self.action
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
}
impl UIElement for TouchPlane {
	fn handle_events(&mut self) -> bool {
		if !self.input.handle_events() {
			return false;
		}
		let size = self.size;
		self.action.update(
			&self.input,
			|snap| match snap.input() {
				InputDataType::Pointer { .. } => snap.distance() < 0.0,
				InputDataType::Hand { data } => {
					Self::hover(size, Vec3::from(data.index.tip.pose.position), true)
				}
				InputDataType::Tip { data } => {
					Self::hover(size, Vec3::from(data.pose.position), true)
				}
			},
			|snap| match snap.input() {
				InputDataType::Pointer { .. } => snap.datamap_f32("select") > 0.5,
				InputDataType::Hand { data } => {
					Self::hover(size, Vec3::from(data.index.tip.pose.position), false)
				}
				InputDataType::Tip { data } => {
					Self::hover(size, Vec3::from(data.pose.position), false)
				}
			},
		);
		true
	}
}
impl VisualDebug for TouchPlane {
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
