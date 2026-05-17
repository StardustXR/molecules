use super::{DeltaSet, InputQueue, InputSnapshot, SimpleAction};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct MultiAction {
	interact_condition: SimpleAction,
	hover: DeltaSet<Arc<InputSnapshot>>,
	interact: DeltaSet<Arc<InputSnapshot>>,
}
impl MultiAction {
	pub fn update(
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

		self.interact_condition.update(queue, &interact_condition);

		// capture when just started interacting and was already hovering (not newly focused)
		for snap in self
			.interact_condition
			.started_acting()
			.iter()
			.filter(|s| self.hover.current.contains(*s))
			.filter(|s| !self.hover.added.contains(*s))
		{
			queue.start_capture(snap);
		}
		// release when stopped interacting
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
				// don't hover when interacting
				.filter(|s| !self.interact_condition.currently_acting().contains(*s))
				// except if we were hovering before and not yet captured
				.chain(
					hover_snaps
						.iter()
						.filter(|s| current_hover.contains(*s) && !s.semantic.captured),
				)
				.cloned(),
		);
	}
	pub fn hover(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		&self.hover
	}
	pub fn interact(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		&self.interact
	}
}
