use super::{DeltaSet, InputMethod, InputQueue, InputSnapshot};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct SimpleAction(DeltaSet<Arc<InputSnapshot>>);
impl SimpleAction {
	pub(super) fn update_from_map(
		&mut self,
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
		active_condition: &impl Fn(&InputSnapshot) -> bool,
	) {
		self.0.push_new(
			input
				.values()
				.filter(|snap| (active_condition)(snap))
				.cloned(),
		);
	}

	pub fn update(
		&mut self,
		queue: &InputQueue,
		active_condition: &impl Fn(&InputSnapshot) -> bool,
	) {
		let input = queue.input();
		self.update_from_map(&input, active_condition);
	}

	#[cfg(test)]
	pub fn test_update(
		&mut self,
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
		active_condition: &impl Fn(&InputSnapshot) -> bool,
	) {
		self.update_from_map(input, active_condition);
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
