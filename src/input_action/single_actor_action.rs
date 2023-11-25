use rustc_hash::FxHashSet;
use stardust_xr_fusion::input::InputData;
use std::sync::Arc;

use crate::input_action::{ActiveCondition, BaseInputAction, InputAction, InputActionState};

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
	pub fn update<O: InputActionState>(
		&mut self,
		condition_action: Option<&mut impl InputAction<O>>,
	) {
		let old_actor = self.actor.clone();

		if let Some(actor) = &self.actor {
			if self.base_action.stopped_acting.contains(actor) {
				self.actor = None;
			}
		}
		let started_acting;
		if let Some(condition_action) = condition_action {
			let condition_acting = condition_action
				.base()
				.currently_acting
				.difference(&condition_action.base().started_acting)
				.cloned()
				.collect::<FxHashSet<_>>();
			started_acting = self
				.base_action
				.started_acting
				.intersection(&condition_acting)
				.next()
				.cloned();
			self.base_action.capture_on_trigger =
				self.capture_on_trigger && !condition_acting.is_empty();
		} else {
			started_acting = self.base_action.started_acting.iter().next().cloned();
			self.base_action.capture_on_trigger = self.capture_on_trigger;
		}
		if let Some(started_acting) = started_acting {
			self.actor = Some(started_acting.clone());
		} else if let Some(actor) = &self.actor {
			if let Some(actor) = self.base_action.currently_acting.get(actor) {
				self.actor = Some(actor.clone());
			}
		}

		self.actor_started = old_actor.is_none() && self.actor.is_some();
		self.actor_changed = old_actor.is_some() && self.actor.is_some() && old_actor != self.actor;
		self.actor_acting = self.actor.is_some();
		self.actor_stopped = old_actor.is_some() && self.actor.is_none();
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
impl<S: InputActionState> InputAction<S> for SingleActorAction<S> {
	fn base(&self) -> &BaseInputAction<S> {
		&self.base_action
	}
	fn base_mut(&mut self) -> &mut BaseInputAction<S> {
		&mut self.base_action
	}
}
