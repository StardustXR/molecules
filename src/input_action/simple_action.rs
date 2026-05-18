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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::input_action::test_helpers::*;

	#[tokio::test]
	async fn started_acting_on_first_frame() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = SimpleAction::default();

		action.test_update(&make_input([s1.clone()]), &|_| true);

		assert!(action.started_acting().contains(&s1));
		assert!(action.currently_acting().contains(&s1));
		assert!(action.stopped_acting().is_empty());
	}

	#[tokio::test]
	async fn stays_acting_on_second_frame() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = SimpleAction::default();
		let input = make_input([s1.clone()]);

		action.test_update(&input, &|_| true);
		action.test_update(&input, &|_| true);

		assert!(action.started_acting().is_empty());
		assert!(action.currently_acting().contains(&s1));
		assert!(action.stopped_acting().is_empty());
	}

	#[tokio::test]
	async fn stopped_acting_when_snap_removed() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = SimpleAction::default();

		action.test_update(&make_input([s1.clone()]), &|_| true);
		action.test_update(&make_input([]), &|_| true);

		assert!(action.started_acting().is_empty());
		assert!(action.currently_acting().is_empty());
		assert!(action.stopped_acting().contains(&s1));
	}

	#[tokio::test]
	async fn condition_filters_snaps() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let s2 = make_snapshot(&device, false);
		let mut action = SimpleAction::default();
		let s1_method = s1.method.clone();

		action.test_update(&make_input([s1.clone(), s2.clone()]), &|snap| {
			snap.method == s1_method
		});

		assert!(action.currently_acting().contains(&s1));
		assert!(!action.currently_acting().contains(&s2));
	}

	#[tokio::test]
	async fn multiple_snaps_simultaneously() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let s2 = make_snapshot(&device, false);
		let mut action = SimpleAction::default();

		action.test_update(&make_input([s1.clone(), s2.clone()]), &|_| true);

		assert_eq!(action.started_acting().len(), 2);
		assert_eq!(action.currently_acting().len(), 2);

		action.test_update(&make_input([s1.clone()]), &|_| true);

		assert!(action.stopped_acting().contains(&s2));
		assert!(action.currently_acting().contains(&s1));
	}
}
