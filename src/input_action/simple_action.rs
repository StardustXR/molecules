use super::{DeltaSet, InputQueue, InputSnapshot};
use rustc_hash::FxHashSet;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct SimpleAction(DeltaSet<Arc<InputSnapshot>>);
impl SimpleAction {
	pub fn update(&mut self, queue: &InputQueue, active_condition: &impl Fn(&InputSnapshot) -> bool) {
		self.0.push_new(
			queue
				.input()
				.into_values()
				.filter(|snap| (active_condition)(snap))
		);
	}
	pub fn started_acting(&self) -> &FxHashSet<Arc<InputSnapshot>> {
		self.0.added()
	}
	pub fn currently_acting(&self) -> &FxHashSet<Arc<InputSnapshot>> {
		self.0.current()
	}
	pub fn stopped_acting(&self) -> &FxHashSet<Arc<InputSnapshot>> {
		self.0.removed()
	}
}
