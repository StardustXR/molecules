use super::{DeltaSet, InputQueue, InputSnapshot, SimpleAction};

use std::sync::Arc;

#[derive(Default, Debug)]
pub struct MultiAction {
	interact_condition: SimpleAction,
	hover: DeltaSet<Arc<InputSnapshot>>,
	interact: DeltaSet<Arc<InputSnapshot>>,
}
impl MultiAction {
	pub(super) fn update_from_map(
		&mut self,
		queue: &InputQueue,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
	) {
		let input = queue.input();
		let hover_snaps: Vec<Arc<InputSnapshot>> = input
			.values()
			.filter(|snap| (hover_condition)(snap))
			.cloned()
			.collect();

		self.interact_condition
			.update_from_map(&input, &interact_condition);

		for snap in self
			.interact_condition
			.started_acting()
			.iter()
			.filter(|s| self.hover.current.contains(*s))
			.filter(|s| !self.hover.added.contains(*s))
		{
			queue.start_capture(snap);
		}
		for snap in self.interact_condition.stopped_acting() {
			queue.release_capture(snap);
		}

		let interacting: Vec<Arc<InputSnapshot>> = self
			.interact_condition
			.currently_acting()
			.iter()
			.filter(|s| s.semantic.captured)
			.cloned()
			.collect();
		self.interact.push_new(interacting.into_iter());

		let current_hover = self.hover.current.clone();
		self.hover.push_new(
			hover_snaps
				.iter()
				.filter(|s| !self.interact_condition.currently_acting().contains(*s))
				.chain(
					hover_snaps
						.iter()
						.filter(|s| current_hover.contains(*s) && !s.semantic.captured),
				)
				.cloned(),
		);
	}

	pub fn update(
		&mut self,
		queue: &InputQueue,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
	) {
		self.update_from_map(queue, hover_condition, interact_condition);
	}

	pub fn hover(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		&self.hover
	}
	pub fn interact(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		&self.interact
	}
}
