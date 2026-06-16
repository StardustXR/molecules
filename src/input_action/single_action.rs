use super::{DeltaSet, InputMethod, InputQueue, InputSnapshot, MultiAction};
use rustc_hash::FxHashMap;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct SingleAction {
	multi: MultiAction,

	actor_started: bool,
	actor_changed: bool,
	actor_acting: bool,
	actor_stopped: bool,

	actor: Option<Arc<InputSnapshot>>,
}
impl SingleAction {
	fn update_actor(
		&mut self,
		change_actor: bool,
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
	) {
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
			} else if let Some(new_actor) = input.get(&actor.method).cloned() {
				*actor = new_actor;
			}
		}

		self.actor_acting = self.actor.is_some();
	}

	pub fn update(
		&mut self,
		change_actor: bool,
		queue: &InputQueue,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
	) {
		let input = queue.input();
		self.multi
			.update_from_map(queue, hover_condition, interact_condition);
		self.update_actor(change_actor, &input);
	}

	pub fn hovering(&self) -> &DeltaSet<Arc<InputSnapshot>> {
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
	pub fn actor(&self) -> Option<&Arc<InputSnapshot>> {
		self.actor.as_ref()
	}
}
