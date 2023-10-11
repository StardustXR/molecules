/// An accumulation effect that acts similar to heating up metal or film photography.
///
/// Used for buttons that require either high intensity or high time pressed
/// e.g. close buttons that shouldn't be pressed accidentally
pub struct Exposure {
	/// How exposed this currently is
	pub exposure: f32,
	/// How much per second the exposure decreases
	pub cooling: f32,
	/// Maximum exposure
	pub max: f32,
}
impl Exposure {
	pub fn update(&mut self, delta: f32) {
		self.exposure -= self.cooling * delta;
		self.exposure = self.exposure.clamp(0.0, self.max);
	}
	pub fn expose_flash(&mut self, amount: f32) {
		self.exposure += amount;
	}
	pub fn expose(&mut self, amount: f32, delta: f32) {
		self.exposure += amount * delta;
	}
}
