#![allow(clippy::mutable_key_type)]

pub mod accent_color;
pub mod button;
pub mod derezzable;
pub mod exposure;
pub mod grabbable;
pub mod hover_plane;
pub mod input_action;
pub mod keyboard_handler;
pub mod lines;
pub mod mouse_handler;
pub mod multi;
pub mod touch_plane;

pub use derezzable::Derezzable;
pub use exposure::Exposure;

use stardust_xr_fusion::{
	client::FrameInfo,
	types::{Color, rgba_linear},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugSettings {
	pub line_thickness: f32,
	pub line_color: Color,
}
impl Default for DebugSettings {
	fn default() -> Self {
		Self {
			line_thickness: 0.002,
			line_color: rgba_linear!(0.14, 0.62, 1.0, 1.0),
		}
	}
}

pub trait VisualDebug {
	fn set_debug(&mut self, settings: Option<DebugSettings>);
}
pub trait UIElement {
	fn handle_events(&mut self) -> bool;
}
pub trait FrameSensitive {
	fn frame(&mut self, info: &FrameInfo);
}
