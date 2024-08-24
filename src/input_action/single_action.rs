use super::{DeltaSet, InputQueue, MultiAction};
use stardust_xr_fusion::input::InputData;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct SingleAction {
	multi: MultiAction,

	actor_started: bool,
	actor_changed: bool,
	actor_acting: bool,
	actor_stopped: bool,

	actor: Option<Arc<InputData>>,
}
impl SingleAction {
	pub fn update(
		&mut self,
		change_actor: bool,
		queue: &InputQueue,
		hover_condition: impl Fn(&InputData) -> bool,
		interact_condition: impl Fn(&InputData) -> bool,
	) {
		self.multi
			.update(queue, hover_condition, interact_condition);

		self.actor_started = false;
		self.actor_changed = false;
		self.actor_stopped = false;
		if let Some(started) = self.multi.interact().added().iter().next() {
			if self.actor.is_none() {
				self.actor_started = true;
				self.actor.replace(started.clone());
			} else if change_actor {
				self.actor_changed = true;
				self.actor.replace(started.clone());
			}
		}

		if let Some(actor) = &mut self.actor {
			if self.multi.interact().removed().contains(actor) {
				self.actor_stopped = true;
				self.actor.take();
			} else if let Some((new_actor, _)) = queue.input().get_key_value(actor) {
				*actor = new_actor.clone();
			}
		}

		self.actor_acting = self.actor.is_some();
	}

	pub fn hovering(&self) -> &DeltaSet<Arc<InputData>> {
		self.multi.hover()
	}
	pub fn actor_started(&self) -> bool {
		self.actor_started
	}
	pub fn actor_changed(&self) -> bool {
		self.actor_changed
	}
	pub fn actor_acting(&self) -> bool {
		self.actor_acting
	}
	pub fn actor_stopped(&self) -> bool {
		self.actor_stopped
	}
	pub fn actor(&self) -> Option<&Arc<InputData>> {
		self.actor.as_ref()
	}
}
