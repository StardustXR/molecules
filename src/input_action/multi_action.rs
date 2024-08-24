use super::{DeltaSet, InputQueue, SimpleAction};
use stardust_xr_fusion::input::InputData;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct MultiAction {
	interact_condition: SimpleAction,
	hover: DeltaSet<Arc<InputData>>,
	interact: DeltaSet<Arc<InputData>>,
}
impl MultiAction {
	pub fn update(
		&mut self,
		queue: &InputQueue,
		hover_condition: impl Fn(&InputData) -> bool,
		interact_condition: impl Fn(&InputData) -> bool,
	) {
		let input = queue.input();
		let hover_action = input.keys().filter(|d| (hover_condition)(d));
		self.interact_condition.update(queue, &interact_condition);

		// initial capture when just started interacting and valid
		for input in self
			.interact_condition
			.started_acting()
			.iter()
			// gotta make sure it only tries to capture it when hovering
			.filter(|i| self.hover.current.contains(*i))
			// but not if it started hovering at the same time (this means it just got "focus")
			.filter(|i| !self.hover.added.contains(*i))
		{
			queue.request_capture(input);
		}
		let interacting_inputs = self
			.interact_condition
			.currently_acting()
			.iter()
			.filter(|k| k.captured)
			.cloned()
			.collect::<Vec<_>>();
		// keep capturing when interacting and already captured
		for input in &interacting_inputs {
			queue.request_capture(input);
		}
		// only something that's been captured can count as interactable to ensure a valid interaction
		self.interact.push_new(interacting_inputs.into_iter());

		// TOOD: make this code not stupid
		let current_hover_state = self.hover.current.clone();
		self.hover.push_new(
			hover_action
				.clone()
				// don't hover when interacting
				.filter(|i| !self.interact_condition.currently_acting().contains(*i))
				// except if we just started interacting and were hovering before and it's not captured
				.chain(hover_action.filter(|i| current_hover_state.contains(*i) && !i.captured))
				.cloned(),
		);
	}
	pub fn hover(&self) -> &DeltaSet<Arc<InputData>> {
		&self.hover
	}
	pub fn interact(&self) -> &DeltaSet<Arc<InputData>> {
		&self.interact
	}
}
