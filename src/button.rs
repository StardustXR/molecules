use std::{array, f32::consts::PI};

use crate::{
	lines::{circle, lerp, make_line_points, rounded_rectangle},
	touch_plane::TouchPlane,
	VisualDebug,
};
use color::{rgba, AlphaColor, Rgba};
use map_range::MapRange;
use mint::Vector2;
use stardust_xr_fusion::{
	core::values::Transform,
	drawable::{LinePoint, Lines},
	node::NodeError,
	spatial::Spatial,
};

#[derive(Debug, Clone, Copy)]
pub struct ButtonSettings {
	pub max_hover_distance: f32,
	pub line_thickness: f32,
	pub accent_color: Rgba<f32>,
}
impl Default for ButtonSettings {
	fn default() -> Self {
		Self {
			max_hover_distance: 0.025,
			line_thickness: 0.005,
			accent_color: rgba!(0.0, 1.0, 0.75, 1.0),
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
		parent: &Spatial,
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
			0.01,
			-half_size_x..half_size_x,
			half_size_y..-half_size_y,
		)?;

		Ok(Button {
			visuals: ButtonVisuals::create(touch_plane.root(), size, &settings)?,
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
}
impl VisualDebug for Button {
	fn set_debug(&mut self, settings: Option<crate::DebugSettings>) {
		self.touch_plane.set_debug(settings)
	}
}

struct ButtonVisuals {
	circle_points: Vec<LinePoint>,
	rounded_rectangle_points: Vec<LinePoint>,
	outline: Lines,
	_corner_lines: [UnboundedVolumeSignifier; 4],
}
impl ButtonVisuals {
	fn create(
		parent: &Spatial,
		size: Vector2<f32>,
		settings: &ButtonSettings,
	) -> Result<Self, NodeError> {
		let half_size_x = size.x * 0.5;
		let half_size_y = size.y * 0.5;
		let segment_count = (size.x.min(size.y) * 1280.0) as usize / 4 * 4;
		let mut circle_points = make_line_points(
			&circle(segment_count, PI * 0.5, half_size_x.min(half_size_y)),
			0.0025,
			rgba!(1.0, 1.0, 1.0, 1.0),
		);
		circle_points.reverse();
		let rounded_rectangle_points = make_line_points(
			&rounded_rectangle(
				size.x,
				size.y,
				settings.line_thickness * 0.5,
				segment_count / 4 - 1,
			),
			settings.line_thickness,
			rgba!(1.0, 1.0, 1.0, 1.0),
		);
		let outline = Lines::create(
			parent,
			Transform::from_scale([1.0, 1.0, 0.0]),
			&circle_points,
			true,
		)?;
		let corner_lines = array::from_fn(|n| {
			let (corner_sin, corner_cos) = (settings.line_thickness * 0.5).sin_cos();
			let corner_sin = (1.0 - corner_sin) * settings.line_thickness * 0.5;
			let corner_cos = (1.0 - corner_cos) * settings.line_thickness * 0.5;

			let position = match n {
				0 => [-half_size_x + corner_sin, half_size_y - corner_cos],
				1 => [half_size_x - corner_sin, half_size_y - corner_cos],
				2 => [half_size_x - corner_sin, -half_size_y + corner_cos],
				3 => [-half_size_x + corner_sin, -half_size_y + corner_cos],
				_ => unimplemented!(),
			};
			UnboundedVolumeSignifier::create(
				&outline,
				position,
				settings.line_thickness,
				settings.accent_color,
			)
			.unwrap()
		});

		Ok(ButtonVisuals {
			circle_points,
			rounded_rectangle_points,
			outline,
			_corner_lines: corner_lines,
		})
	}

	pub fn update(&self, touch_plane: &TouchPlane, settings: &ButtonSettings) {
		if touch_plane.hovering_inputs().is_empty() && !touch_plane.touching() {
			let _ = self.outline.set_scale(None, [0.0; 3]);
		}
		if let Some((hover_point, hover_distance)) = touch_plane
			.hovering_inputs()
			.into_iter()
			.map(|p| touch_plane.interact_point(p))
			.nth(0)
		{
			let scale = hover_distance
				.map_range(settings.max_hover_distance..0.0, 0.0..1.0)
				.clamp(0.0, 1.0);

			let scale_morph = scale.map_range(0.5..1.0, 0.0..1.0);
			let _ = self.outline.update_points(
				lerp(
					&self.circle_points,
					&self.rounded_rectangle_points,
					scale_morph,
				)
				.as_ref()
				.unwrap(),
			);
			let _ = self.outline.set_transform(
				None,
				Transform::from_position_scale(
					[
						hover_point.x * (1.0 - scale),
						hover_point.y * (1.0 - scale),
						0.0,
					],
					[scale, scale, 0.000],
				),
			);
		}
		if touch_plane.touch_started() {
			let points = self
				.rounded_rectangle_points
				.iter()
				.map(|p| {
					let mut point = p.clone();
					point.color = settings.accent_color;
					point
				})
				.collect::<Vec<_>>();
			self.outline.update_points(&points).unwrap();
		}
		if touch_plane.touching() {
			let distance = touch_plane
				.touching_inputs()
				.into_iter()
				.map(|i| touch_plane.interact_point(i).1)
				.reduce(|a, b| a.abs().max(b.abs()))
				.unwrap()
				.abs();

			let _ = self.outline.set_scale(None, [1.0, 1.0, distance]);
		}
		// if touch_plane.touch_stopped() {
		// 	self.outline.update_points(&self.circle_points).unwrap();
		// }
	}
}

struct UnboundedVolumeSignifier(Lines);
impl UnboundedVolumeSignifier {
	pub fn create(
		parent: &Spatial,
		position: impl Into<Vector2<f32>>,
		thickness: f32,
		color: Rgba<f32>,
	) -> Result<Self, NodeError> {
		let position = position.into();
		let start_point = LinePoint {
			point: [0.0; 3].into(),
			thickness,
			color,
		};
		let end_point = LinePoint {
			point: [0.0, 0.0, -1.0].into(),
			thickness,
			color: AlphaColor::new(color.rgb(), 0.0),
		};
		Ok(UnboundedVolumeSignifier(Lines::create(
			parent,
			Transform::from_position([position.x, position.y, 0.0]),
			&[start_point, end_point],
			false,
		)?))
	}
}