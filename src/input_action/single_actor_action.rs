use crate::input_action::{ActiveCondition, BaseInputAction, InputActionState};
use rustc_hash::FxHashSet;
use stardust_xr_fusion::input::InputData;
use std::sync::Arc;

// so this code is hella buggy, it needs better logic to fulfill the requirements:
// when no condition action is present:
//     - first actor to fulfill the condition is the single actor
//     - if change_actor and another actor fulfills the condition, make it the single actor
// when a condition action is present:
//     - first actor that fulfills the active condition after fulfilling the condition action (so, can't just have started fulfilling the condition action's active condition) is the single actor
//     - if change_actor, then the next actor that fulfills the active condition after fulfilling the condition action (so, can't just have started fulfilling the condition action's active condition) is the single actor
//     - if the single actor stops acting (with the condition action not being met at the same time) then it must have lost tracking or similar, so if that actor starts acting again (even if the condition action was started being met the same frame) then make it the single actor unless there is another

#[derive(Debug)]
pub struct SingleActorAction<S: InputActionState> {
	pub base_action: BaseInputAction<S>,
	pub capture_on_trigger: bool,
	pub change_actor: bool,

	actor_started: bool,
	actor_changed: bool,
	actor_acting: bool,
	actor_stopped: bool,

	actor: Option<Arc<InputData>>,
}
impl<S: InputActionState> SingleActorAction<S> {
	pub fn new(
		capture_on_trigger: bool,
		active_condition: ActiveCondition<S>,
		change_actor: bool,
	) -> Self {
		Self {
			base_action: BaseInputAction::new(false, active_condition),
			capture_on_trigger,
			change_actor,

			actor_started: false,
			actor_changed: false,
			actor_acting: false,
			actor_stopped: false,

			actor: None,
		}
	}
	pub fn update(&mut self, condition_action: Option<&mut BaseInputAction<S>>) {
		let old_actor = self.actor.clone();

		match &mut self.actor {
			// Action not active (e.g. object is not being grabbed)
			None => {
				self.actor = self.check_actor_starting(condition_action);
			}
			// Action active (e.g. object is being grabbed)
			Some(actor) => {
				// If we stopped acting (e.g. stopped pinching)
				if self.base_action.stopped_acting.contains(actor) {
					// If the condition is still happening (e.g. your hand just unpinched but is still nearby)
					if condition_action.is_some() {
						self.actor.take();
					} else {
						// Action is still active here but hand stopped being tracked or some other disruption, so don't do anything
					}
				} else {
					if self.change_actor {
						if let Some(new_actor) = self.check_actor_starting(condition_action) {
							self.actor.replace(new_actor);
						}
					} else {
						if let Some(new_actor) =
							self.base_action.currently_acting.get(actor).cloned()
						{
							self.actor.replace(new_actor);
						}
					}
				}
			}
		}

		self.actor_started = old_actor.is_none() && self.actor.is_some();
		self.actor_changed = old_actor.is_some() && self.actor.is_some() && old_actor != self.actor;
		self.actor_acting = self.actor.is_some();
		self.actor_stopped = old_actor.is_some() && self.actor.is_none();
	}
	fn check_actor_starting(
		&self,
		condition_action: Option<&mut BaseInputAction<S>>,
	) -> Option<Arc<InputData>> {
		// If there's a condition (e.g. your hand has to be close to grab something)
		if let Some(condition_action) = condition_action {
			// All the input data of input methods that are currently fulfilling the condition action but have not just started
			let condition_acting = condition_action
				.currently_acting
				.difference(&condition_action.started_acting)
				.cloned()
				.collect::<FxHashSet<_>>();
			// Pick the first one that just started (e.g. grab condition action is being close enough, single actor action pinching, you want to be close to something BEFORE you pinch it)
			self.base_action
				.started_acting
				.intersection(&condition_acting)
				.next()
				.cloned()
		} else {
			// Pick the first thing that started acting
			self.base_action.started_acting.iter().next().cloned()
		}
	}

	pub fn base(&self) -> &BaseInputAction<S> {
		&self.base_action
	}
	pub fn base_mut(&mut self) -> &mut BaseInputAction<S> {
		&mut self.base_action
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
