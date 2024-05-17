use crate::input_action::MultiActorAction;
use rustc_hash::FxHashSet;
use stardust_xr_fusion::input::InputData;
use std::sync::Arc;

use super::InputQueue;

// so this code is hella buggy, it needs better logic to fulfill the requirements:
// when no condition action is present:
//     - first actor to fulfill the condition is the single actor
//     - if change_actor and another actor fulfills the condition, make it the single actor
// when a condition action is present:
//     - first actor that fulfills the active condition after fulfilling the condition action (so, can't just have started fulfilling the condition action's active condition) is the single actor
//     - if change_actor, then the next actor that fulfills the active condition after fulfilling the condition action (so, can't just have started fulfilling the condition action's active condition) is the single actor
//     - if the single actor stops acting (with the condition action not being met at the same time) then it must have lost tracking or similar, so if that actor starts acting again (even if the condition action was started being met the same frame) then make it the single actor unless there is another

#[derive(Default, Debug)]
pub struct SingleActorAction {
	condition: MultiActorAction,
	interact: MultiActorAction,

	actor_started: bool,
	actor_changed: bool,
	actor_acting: bool,
	actor_stopped: bool,

	actor: Option<Arc<InputData>>,
}
impl SingleActorAction {
	pub fn update(
		&mut self,
		change_actor: bool,
		queue: &InputQueue,
		condition: impl Fn(&InputData) -> bool,
		interact: impl Fn(&InputData) -> bool,
	) {
		let old_actor = self.actor.clone();

		self.condition.update(false, queue, condition);
		self.interact.update(false, queue, interact);

		// check if there's any input that could be the new actor
		let condition_met = FxHashSet::from_iter(
			self.condition
				.currently_acting()
				// if the condition was met at the same time the interaction happens then they must have both come into range
				.difference(&self.condition.started_acting())
				.cloned(),
		);
		let starting_actor = condition_met
			.intersection(self.interact.started_acting())
			.next()
			.cloned();
		'condition: {
			match &mut self.actor {
				None => {
					if let Some(actor) = starting_actor {
						queue.request_capture(&actor);
						self.actor.replace(actor);
					}
				}
				Some(actor) => {
					if change_actor {
						if let Some(new_actor) = starting_actor {
							*actor = new_actor;
						}
					}
					if let Some((new_actor, _)) = queue.input().get_key_value(actor) {
						if self.interact.currently_acting().contains(new_actor) {
							*actor = new_actor.clone();
						} else if self.condition.currently_acting().contains(actor) {
							self.actor.take();
							break 'condition;
						}
					}

					queue.request_capture(actor);
				}
			};
		};

		self.actor_started = old_actor.is_none() && self.actor.is_some();
		self.actor_changed = old_actor.is_some() && self.actor.is_some() && old_actor != self.actor;
		self.actor_acting = self.actor.is_some();
		self.actor_stopped = old_actor.is_some() && self.actor.is_none();
	}

	pub fn condition(&self) -> &MultiActorAction {
		&self.condition
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
