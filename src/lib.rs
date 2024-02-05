pub mod button;
pub mod data;
mod dummy;
mod exposure;
mod grabbable;
pub mod input_action;
pub mod keyboard;
pub mod lines;
pub mod mouse;
pub mod multi;
pub mod touch_plane;

use color::{color_space::LinearRgb, rgba_linear};
pub use dummy::*;
pub use exposure::*;
pub use grabbable::*;

#[derive(Debug, Clone, Copy)]
pub struct DebugSettings {
	pub line_thickness: f32,
	pub line_color: color::Rgba<f32, LinearRgb>,
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
