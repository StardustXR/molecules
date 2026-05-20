use stardust_xr_fusion::types::{Color, rgba_linear};
use tokio::sync::watch;

/// Accent color watcher. Currently returns a static default color.
/// Desktop portal integration will be added in a future update.
pub struct AccentColor {
	pub color: watch::Receiver<Color>,
}
impl AccentColor {
	pub fn new() -> Self {
		let (_, color) = watch::channel(rgba_linear!(0.14, 0.62, 1.0, 1.0));
		Self { color }
	}

	pub fn color(&self) -> Color {
		*self.color.borrow()
	}
}
impl Default for AccentColor {
	fn default() -> Self {
		Self::new()
	}
}
