pub mod button;
pub mod data;
mod exposure;
mod grabbable;
pub mod hover_plane;
pub mod input_action;
pub mod keyboard;
pub mod lines;
pub mod mouse;
pub mod multi;
pub mod state_machine;
pub mod touch_plane;

pub use exposure::*;
pub use grabbable::*;

use stardust_xr_fusion::{
	core::values::color::{color_space::LinearRgb, rgba_linear, Rgba},
	root::FrameInfo,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugSettings {
	pub line_thickness: f32,
	pub line_color: Rgba<f32, LinearRgb>,
}
impl Default for DebugSettings {
	fn default() -> Self {
		Self {
			line_thickness: 0.002,
			line_color: rgba_linear!(0.14, 0.62, 1.0, 1.0),
		}
	}
}

/// Trait to enable visual debugging of invisible widgets
pub trait VisualDebug {
	/// Enable or disable the visual debugging using the provided settings.
	fn set_debug(&mut self, settings: Option<DebugSettings>);
}

pub trait UIElement {
	/// Handle events, returns true if events were handled (e.g. when input has been updated)
	fn handle_events(&mut self) -> bool;
}
pub trait FrameSensitive {
	fn frame(&mut self, info: &FrameInfo);
}
