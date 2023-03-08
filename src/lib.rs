mod grabbable;
pub mod keyboard;
pub mod lines;
pub mod mouse;
pub mod resources;
mod single_actor_action;
pub mod touch_plane;

use color::rgba;
pub use grabbable::*;
pub use single_actor_action::SingleActorAction;

#[derive(Debug, Clone, Copy)]
pub struct DebugSettings {
	pub thickness: f32,
	pub color: color::Rgba<f32>,
}
impl Default for DebugSettings {
	fn default() -> Self {
		Self {
			thickness: 0.002,
			color: rgba!(0.14, 0.62, 1.0, 1.0),
		}
	}
}

pub trait VisualDebug {
	fn set_debug(&mut self, settings: Option<DebugSettings>);
}
