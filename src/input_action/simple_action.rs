use super::{DeltaSet, InputQueue};
use rustc_hash::FxHashSet;
use stardust_xr_fusion::input::InputData;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct SimpleAction(DeltaSet<Arc<InputData>>);
impl SimpleAction {
	pub fn update(&mut self, queue: &InputQueue, active_condition: &impl Fn(&InputData) -> bool) {
		self.0.push_new(
			queue
				.input()
				.iter()
				// filter out every input method that doesn't meet the active condition
				.filter(|(d, _)| (active_condition)(d))
				// we don't need the input method anymore
				.map(|(d, _)| d)
				.cloned(),
		);
	}
	pub fn started_acting(&self) -> &FxHashSet<Arc<InputData>> {
		self.0.added()
	}
	pub fn currently_acting(&self) -> &FxHashSet<Arc<InputData>> {
		self.0.current()
	}
	pub fn stopped_acting(&self) -> &FxHashSet<Arc<InputData>> {
		self.0.removed()
	}
}
