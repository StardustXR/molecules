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
		self.multi.update_from_map(
			&input,
			hover_condition,
			interact_condition,
			|snap| queue.start_capture(snap),
			|snap| queue.release_capture(snap),
		);
		self.update_actor(change_actor, &input);
	}

	#[cfg(test)]
	pub fn test_update(
		&mut self,
		change_actor: bool,
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
	) {
		self.multi
			.update_from_map(input, hover_condition, interact_condition, |_| {}, |_| {});
		self.update_actor(change_actor, input);
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::input_action::test_helpers::*;

	fn always(_: &InputSnapshot) -> bool {
		true
	}
	fn never(_: &InputSnapshot) -> bool {
		false
	}

	/// Advance through the frames needed for a snap to become a captured actor:
	/// hover frame → interact (uncaptured) → interact (captured, actor set).
	fn reach_actor(
		action: &mut SingleAction,
		device: &pion_binder::PionBinderDevice,
	) -> Arc<InputSnapshot> {
		let s = make_snapshot(device, false);
		// hover frame
		action.test_update(false, &make_input([s.clone()]), always, never);
		// interact started but not captured
		let s_int = snap_with_method(s.method.clone(), false);
		action.test_update(false, &make_input([s_int.clone()]), always, always);
		// captured
		let s_cap = snap_with_method(s.method.clone(), true);
		action.test_update(false, &make_input([s_cap.clone()]), always, always);
		s_cap
	}

	#[tokio::test]
	async fn actor_starts_when_interact_begins() {
		let device = make_device();
		let mut action = SingleAction::default();
		let actor = reach_actor(&mut action, &device);

		assert!(action.actor_started());
		assert!(action.actor_acting());
		assert_eq!(action.actor().unwrap().method, actor.method);
	}

	#[tokio::test]
	async fn actor_acting_on_subsequent_frames() {
		let device = make_device();
		let mut action = SingleAction::default();
		let actor = reach_actor(&mut action, &device);

		let next = snap_with_method(actor.method.clone(), true);
		action.test_update(false, &make_input([next.clone()]), always, always);

		assert!(!action.actor_started());
		assert!(action.actor_acting());
		assert_eq!(action.actor().unwrap().method, actor.method);
	}

	#[tokio::test]
	async fn actor_snap_updated_each_frame() {
		let device = make_device();
		let mut action = SingleAction::default();
		let actor = reach_actor(&mut action, &device);

		let updated = snap_with_method(actor.method.clone(), true);
		action.test_update(false, &make_input([updated.clone()]), always, always);

		assert!(std::sync::Arc::ptr_eq(action.actor().unwrap(), &updated));
	}

	#[tokio::test]
	async fn actor_stops_when_interact_ends() {
		let device = make_device();
		let mut action = SingleAction::default();
		let actor = reach_actor(&mut action, &device);

		let released = snap_with_method(actor.method.clone(), false);
		action.test_update(false, &make_input([released.clone()]), always, never);

		assert!(action.actor_stopped());
		assert!(!action.actor_acting());
		assert!(action.actor().is_none());
	}

	#[tokio::test]
	async fn second_actor_ignored_when_change_actor_false() {
		let device = make_device();
		let mut action = SingleAction::default();
		let first = reach_actor(&mut action, &device);

		// s2 becomes captured and interacting while first is still interacting
		let s2_cap = snap_with_method(make_snapshot(&device, false).method.clone(), true);
		action.test_update(
			false,
			&make_input([snap_with_method(first.method.clone(), true), s2_cap.clone()]),
			always,
			always,
		);

		assert!(!action.actor_changed());
		assert_eq!(action.actor().unwrap().method, first.method);
	}

	#[tokio::test]
	async fn actor_changes_when_change_actor_true() {
		let device = make_device();
		let mut action = SingleAction::default();
		let first = reach_actor(&mut action, &device);

		// s2 becomes captured and interacting while first is still interacting
		let s2_cap = snap_with_method(make_snapshot(&device, false).method.clone(), true);
		action.test_update(
			true,
			&make_input([snap_with_method(first.method.clone(), true), s2_cap.clone()]),
			always,
			always,
		);

		assert!(action.actor_changed());
		assert_eq!(action.actor().unwrap().method, s2_cap.method);
	}
}
