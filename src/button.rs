use crate::{
	UIElement, VisualDebug,
	lines::{LineExt, circle, rounded_rectangle},
	touch_plane::TouchPlane,
};
use glam::{FloatExt, Mat4, vec3};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	drawable::{Lines, LinesExt},
	spatial::{SpatialRef, Transform},
	types::{Color, rgba_linear},
};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct ButtonVisualSettings {
	pub line_thickness: f32,
	pub accent_color: Color,
}
impl Default for ButtonVisualSettings {
	fn default() -> Self {
		Self {
			line_thickness: 0.005,
			accent_color: rgba_linear!(0.0, 1.0, 0.75, 1.0),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonSettings {
	pub max_hover_distance: f32,
	pub visuals: Option<ButtonVisualSettings>,
}
impl Default for ButtonSettings {
	fn default() -> Self {
		Self {
			max_hover_distance: 0.025,
			visuals: Some(ButtonVisualSettings::default()),
		}
	}
}

pub struct Button {
	pub settings: ButtonSettings,
	touch_plane: TouchPlane,
	visuals: Option<ButtonVisuals>,
}
impl Button {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		parent: &SpatialRef,
		transform: Transform,
		size: [f32; 2],
		settings: ButtonSettings,
	) -> Result<Self> {
		let half_size_x = size[0] * 0.5;
		let half_size_y = size[1] * 0.5;
		let touch_plane = TouchPlane::new(
			client,
			parent,
			transform,
			size,
			0.015,
			-half_size_x..half_size_x,
			half_size_y..-half_size_y,
		)
		.await?;

		let visuals = if let Some(v) = settings.visuals {
			Some(ButtonVisuals::new(client, touch_plane.root(), size, v).await?)
		} else {
			None
		};

		Ok(Button {
			settings,
			touch_plane,
			visuals,
		})
	}

	pub fn touch_plane(&self) -> &TouchPlane {
		&self.touch_plane
	}

	pub fn pressed(&self) -> bool {
		!self.touch_plane.action().interact().current().is_empty()
			&& self.touch_plane.action().interact().added().len()
				== self.touch_plane.action().interact().current().len()
	}
	pub fn released(&self) -> bool {
		self.touch_plane.action().interact().current().is_empty()
			&& !self.touch_plane.action().interact().removed().is_empty()
	}
}
impl UIElement for Button {
	fn handle_events(&mut self) -> bool {
		if !self.touch_plane.handle_events() {
			return false;
		}
		if let Some(visuals) = &mut self.visuals {
			visuals.update(&self.touch_plane, &self.settings);
		}
		true
	}
}
impl VisualDebug for Button {
	fn set_debug(&mut self, settings: Option<crate::DebugSettings>) {
		self.touch_plane.set_debug(settings)
	}
}

struct ButtonVisuals {
	size: [f32; 2],
	visual_settings: ButtonVisualSettings,
	segment_count: usize,
	lines: Lines,
}
impl ButtonVisuals {
	async fn new<H: ClientHandler>(
		client: &Client<H>,
		parent: &stardust_xr_fusion::spatial::Spatial,
		size: [f32; 2],
		settings: ButtonVisualSettings,
	) -> Result<Self> {
		let min_size = size[0].min(size[1]);
		let segment_count = if min_size < 0.1 {
			32
		} else if min_size < 1.0 {
			64
		} else {
			128
		};
		let outline = Lines::create(client, parent, vec![]).await?;

		Ok(ButtonVisuals {
			size,
			visual_settings: settings,
			segment_count,
			lines: outline,
		})
	}

	pub fn update(&self, touch_plane: &TouchPlane, settings: &ButtonSettings) {
		let closest_interaction = touch_plane
			.action()
			.hover()
			.current()
			.iter()
			.chain(touch_plane.action().interact().current())
			.map(|s| touch_plane.interact_point(s))
			.reduce(|(a_pos, a_dist), (b_pos, b_dist)| {
				if a_dist < b_dist {
					(a_pos, a_dist)
				} else {
					(b_pos, b_dist)
				}
			});

		let rounded_rect = rounded_rectangle(
			self.size[0],
			self.size[1],
			self.visual_settings.line_thickness * 0.5,
			self.segment_count / 4 - 1,
		)
		.thickness(self.visual_settings.line_thickness);

		let _ = if let Some((interact_point, interact_distance)) = closest_interaction {
			if !touch_plane.action().interact().current().is_empty() {
				self.lines
					.set_lines(vec![rounded_rect.color(self.visual_settings.accent_color)])
			} else {
				let blend = interact_distance
					.remap(settings.max_hover_distance, 0.0, 0.0, 1.0)
					.clamp(0.0, 1.0);
				let mut c = circle(self.segment_count, PI * 0.5, 0.0)
					.thickness(0.0025)
					.transform(Mat4::from_translation(vec3(
						interact_point[0],
						interact_point[1],
						0.0,
					)));
				c.points.reverse();
				let lines = c
					.lerp(&rounded_rect, blend)
					.map(|l| vec![l])
					.unwrap_or_default();
				self.lines.set_lines(lines)
			}
		} else {
			self.lines.set_lines(vec![])
		};
	}
}
