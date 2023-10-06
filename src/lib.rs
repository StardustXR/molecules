pub mod button;
pub mod data;
pub mod datamap;
mod dummy;
mod grabbable;
pub mod keyboard;
pub mod lines;
pub mod mouse;
pub mod multi;
mod single_actor_action;
pub mod touch_plane;

use color::rgba;
pub use dummy::*;
pub use grabbable::*;
pub use single_actor_action::SingleActorAction;

#[derive(Debug, Clone, Copy)]
pub struct DebugSettings {
	pub line_thickness: f32,
	pub line_color: color::Rgba<f32>,
}
impl Default for DebugSettings {
	fn default() -> Self {
		Self {
			line_thickness: 0.002,
			line_color: rgba!(0.14, 0.62, 1.0, 1.0),
		}
	}
}

/// Trait to enable visual debugging of invisible widgets
pub trait VisualDebug {
	/// Enable or disable the visual debugging using the provided settings.
	fn set_debug(&mut self, settings: Option<DebugSettings>);
}
