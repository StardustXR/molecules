use crate::{
	lines::{circle, rounded_rectangle, LineExt},
	touch_plane::TouchPlane,
	VisualDebug,
};
use color::{color_space::LinearRgb, rgba_linear, Rgba};
use glam::{vec3, Mat4};
use map_range::MapRange;
use mint::Vector2;
use stardust_xr_fusion::{
	drawable::{Lines, LinesAspect},
	node::NodeError,
	spatial::{SpatialAspect, Transform},
};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct ButtonSettings {
	pub max_hover_distance: f32,
	pub line_thickness: f32,
	pub accent_color: Rgba<f32, LinearRgb>,
}
impl Default for ButtonSettings {
	fn default() -> Self {
		Self {
			max_hover_distance: 0.025,
			line_thickness: 0.005,
			accent_color: rgba_linear!(0.0, 1.0, 0.75, 1.0),
		}
	}
}

pub struct Button {
	settings: ButtonSettings,
	touch_plane: TouchPlane,
	visuals: ButtonVisuals,
}
impl Button {
	pub fn create(
		parent: &impl SpatialAspect,
		transform: Transform,
		size: impl Into<Vector2<f32>>,
		settings: ButtonSettings,
	) -> Result<Self, NodeError> {
		let size = size.into();
		let half_size_x = size.x * 0.5;
		let half_size_y = size.y * 0.5;
		let touch_plane = TouchPlane::create(
			parent,
			transform,
			size,
			0.015,
			-half_size_x..half_size_x,
			half_size_y..-half_size_y,
		)?;

		Ok(Button {
			visuals: ButtonVisuals::create(touch_plane.root(), size)?,
			settings,
			touch_plane,
		})
	}

	pub fn update(&mut self) {
		self.touch_plane.update();
		self.visuals.update(&self.touch_plane, &self.settings);
	}

	pub fn touch_plane(&self) -> &TouchPlane {
		&self.touch_plane
	}

	pub fn pressed(&self) -> bool {
		!self.touch_plane.touching().current().is_empty()
			&& self.touch_plane.touching().added().len()
				== self.touch_plane.touching().current().len()
	}
	pub fn released(&self) -> bool {
		self.touch_plane.touching().current().is_empty()
			&& !self.touch_plane.touching().removed().is_empty()
	}
}
impl VisualDebug for Button {
	fn set_debug(&mut self, settings: Option<crate::DebugSettings>) {
		self.touch_plane.set_debug(settings)
	}
}

struct ButtonVisuals {
	size: Vector2<f32>,
	segment_count: usize,
	lines: Lines,
}
impl ButtonVisuals {
	fn create(parent: &impl SpatialAspect, size: Vector2<f32>) -> Result<Self, NodeError> {
		let segment_count = (size.x.min(size.y) * 1280.0) as usize / 4 * 4;
		let outline = Lines::create(parent, Transform::from_scale([1.0, 1.0, 0.0]), &[])?;

		Ok(ButtonVisuals {
			size,
			segment_count,
			lines: outline,
		})
	}

	pub fn update(&self, touch_plane: &TouchPlane, settings: &ButtonSettings) {
		let closest_interaction = touch_plane
			.hovering()
			.current()
			.into_iter()
			.chain(touch_plane.touching().current().into_iter())
			.map(|p| touch_plane.interact_point(&p))
			.reduce(|(a_pos, a_distance), (b_pos, b_distance)| {
				if a_distance < b_distance {
					(a_pos, a_distance)
				} else {
					(b_pos, b_distance)
				}
			});

		let rounded_rectangle = rounded_rectangle(
			self.size.x,
			self.size.y,
			settings.line_thickness * 0.5,
			self.segment_count / 4 - 1,
		)
		.thickness(settings.line_thickness);
		let _ = if let Some((interact_point, interact_distance)) = closest_interaction {
			// if we're touching the plane
			if !touch_plane.touching().current().is_empty() {
				// then fill the rectangle
				let lines = vec![rounded_rectangle.color(settings.accent_color)];
				// create_unbounded_volume_signifiers(
				// 	self.size,
				// 	interact_distance,
				// 	settings,
				// 	&mut lines,
				// );
				self.lines.set_lines(&lines)
			} else {
				// if hovering
				let blend = interact_distance
					.map_range(settings.max_hover_distance..0.0, 0.0..1.0)
					.clamp(0.0, 1.0);
				let mut circle = circle(self.segment_count, PI * 0.5, 0.0)
					.thickness(0.0025)
					.transform(Mat4::from_translation(vec3(
						interact_point.x,
						interact_point.y,
						0.0,
					)));
				circle.points.reverse();

				self.lines.set_lines(&[circle
					.clone()
					.lerp(&rounded_rectangle, blend)
					.unwrap_or_default()])
			}
		} else {
			// then nothing is in range
			self.lines.set_lines(&[])
		};
	}
}

// fn create_unbounded_volume_signifiers(
// 	size: Vector2<f32>,
// 	depth: f32,
// 	settings: &ButtonSettings,
// 	lines: &mut Vec<Line>,
// ) {
// 	let half_size_x = size.x * 0.5;
// 	let half_size_y = size.y * 0.5;
// 	let (corner_sin, corner_cos) = (settings.line_thickness * 0.5).sin_cos();
// 	let corner_sin = (1.0 - corner_sin) * settings.line_thickness * 0.5;
// 	let corner_cos = (1.0 - corner_cos) * settings.line_thickness * 0.5;
// 	for n in 0..4 {
// 		let position = match n {
// 			0 => [-half_size_x + corner_sin, half_size_y - corner_cos],
// 			1 => [half_size_x - corner_sin, half_size_y - corner_cos],
// 			2 => [half_size_x - corner_sin, -half_size_y + corner_cos],
// 			3 => [-half_size_x + corner_sin, -half_size_y + corner_cos],
// 			_ => unimplemented!(),
// 		};
// 		lines.push(create_unbounded_volume_signifier(
// 			position.into(),
// 			settings.line_thickness,
// 			depth,
// 			settings.accent_color,
// 		))
// 	}
// }
// fn create_unbounded_volume_signifier(
// 	position: Vector2<f32>,
// 	thickness: f32,
// 	depth: f32,
// 	color: Rgba<f32, LinearRgb>,
// ) -> Line {
// 	let start_point = LinePoint {
// 		point: [0.0; 3].into(),
// 		thickness,
// 		color,
// 	};
// 	let end_point = LinePoint {
// 		point: [0.0, 0.0, -depth.abs()].into(),
// 		thickness,
// 		color: AlphaColor::new(color.rgb(), 0.0),
// 	};
// 	Line {
// 		points: vec![start_point, end_point],
// 		cyclic: false,
// 	}
// 	.transform(Mat4::from_translation(vec3(
// 		position.x, position.y, -thickness,
// 	)))
// }
