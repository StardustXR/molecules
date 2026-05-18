use super::{DeltaSet, InputMethod, InputQueue, InputSnapshot, SimpleAction};
use rustc_hash::FxHashMap;
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
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
		mut start_capture: impl FnMut(&InputSnapshot),
		mut release_capture: impl FnMut(&InputSnapshot),
	) {
		let hover_snaps: Vec<Arc<InputSnapshot>> = input
			.values()
			.filter(|snap| (hover_condition)(snap))
			.cloned()
			.collect();

		self.interact_condition
			.update_from_map(input, &interact_condition);

		for snap in self
			.interact_condition
			.started_acting()
			.iter()
			.filter(|s| self.hover.current.contains(*s))
			.filter(|s| !self.hover.added.contains(*s))
		{
			start_capture(snap);
		}
		for snap in self.interact_condition.stopped_acting() {
			release_capture(snap);
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
		let input = queue.input();
		self.update_from_map(
			&input,
			hover_condition,
			interact_condition,
			|snap| queue.start_capture(snap),
			|snap| queue.release_capture(snap),
		);
	}

	#[cfg(test)]
	pub fn test_update(
		&mut self,
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
	) {
		self.update_from_map(input, hover_condition, interact_condition, |_| {}, |_| {});
	}

	/// Like `test_update` but returns `(captured, released)` method lists so tests can
	/// assert on capture/release call timing.
	#[cfg(test)]
	pub fn test_update_tracked(
		&mut self,
		input: &FxHashMap<InputMethod, Arc<InputSnapshot>>,
		hover_condition: impl Fn(&InputSnapshot) -> bool,
		interact_condition: impl Fn(&InputSnapshot) -> bool,
	) -> (Vec<InputMethod>, Vec<InputMethod>) {
		let mut captured: Vec<InputMethod> = Vec::new();
		let mut released: Vec<InputMethod> = Vec::new();
		self.update_from_map(
			input,
			hover_condition,
			interact_condition,
			|snap| captured.push(snap.method.clone()),
			|snap| released.push(snap.method.clone()),
		);
		(captured, released)
	}

	pub fn hover(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		&self.hover
	}
	pub fn interact(&self) -> &DeltaSet<Arc<InputSnapshot>> {
		&self.interact
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

	#[tokio::test]
	async fn hover_added_on_first_frame() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = MultiAction::default();

		action.test_update(&make_input([s1.clone()]), always, never);

		assert!(action.hover().added().contains(&s1));
		assert!(action.hover().current().contains(&s1));
		assert!(action.interact().current().is_empty());
	}

	#[tokio::test]
	async fn hover_stays_on_second_frame() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = MultiAction::default();
		let input = make_input([s1.clone()]);

		action.test_update(&input, always, never);
		action.test_update(&input, always, never);

		assert!(action.hover().added().is_empty());
		assert!(action.hover().current().contains(&s1));
	}

	#[tokio::test]
	async fn hover_removed_when_snap_leaves() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = MultiAction::default();

		action.test_update(&make_input([s1.clone()]), always, never);
		action.test_update(&make_input([]), always, never);

		assert!(action.hover().removed().contains(&s1));
		assert!(action.hover().current().is_empty());
	}

	#[tokio::test]
	async fn interact_requires_captured_flag() {
		let device = make_device();
		// Frame 1: snap hovers (not newly entered)
		let s1 = make_snapshot(&device, false);
		let mut action = MultiAction::default();
		action.test_update(&make_input([s1.clone()]), always, never);

		// Frame 2: interact condition met but not captured yet
		let s1_interact = snap_with_method(s1.method.clone(), false);
		action.test_update(&make_input([s1_interact.clone()]), always, always);
		assert!(action.interact().current().is_empty());

		// Frame 3: now captured — snap is in interact
		let s1_captured = snap_with_method(s1.method.clone(), true);
		action.test_update(&make_input([s1_captured.clone()]), always, always);
		assert!(action.interact().current().contains(&s1_captured));
	}

	#[tokio::test]
	async fn hover_cleared_while_interacting() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = MultiAction::default();
		action.test_update(&make_input([s1.clone()]), always, never);

		// Snap becomes captured+interacting
		let s1_captured = snap_with_method(s1.method.clone(), true);
		action.test_update(&make_input([s1_captured.clone()]), always, always);
		action.test_update(&make_input([s1_captured.clone()]), always, always);

		assert!(!action.hover().current().contains(&s1_captured));
		assert!(action.interact().current().contains(&s1_captured));
	}

	// --- capture timing ---

	/// A snap that enters and immediately satisfies interact_condition must never be
	/// captured on that same frame, regardless of how many other snaps are already present.
	#[tokio::test]
	async fn no_capture_on_entry_frame() {
		for n_bystanders in 0usize..=3 {
			let device = make_device();
			let mut action = MultiAction::default();

			// Pre-warm some snaps that are already hovering
			let bystanders: Vec<_> = (0..n_bystanders)
				.map(|_| make_snapshot(&device, false))
				.collect();
			if !bystanders.is_empty() {
				action.test_update(&make_input(bystanders.clone()), always, never);
				action.test_update(&make_input(bystanders.clone()), always, never);
			}

			// New snap enters with interact_condition already true on its first frame
			let s = make_snapshot(&device, false);
			let input = make_input(bystanders.iter().cloned().chain([s.clone()]));
			let (captured, _) = action.test_update_tracked(&input, always, always);

			assert!(
				!captured.iter().any(|m| *m == s.method),
				"snap captured on entry frame (n_bystanders={n_bystanders})"
			);
		}
	}

	/// After hovering for at least two frames (one full hover frame clears hover.added),
	/// triggering interact_condition must always produce a capture.
	#[tokio::test]
	async fn capture_after_any_number_of_hover_frames() {
		for hover_frames in 2usize..=5 {
			let device = make_device();
			let mut action = MultiAction::default();
			let s = make_snapshot(&device, false);
			let input = make_input([s.clone()]);

			for _ in 0..hover_frames {
				action.test_update_tracked(&input, always, never);
			}

			let (captured, _) = action.test_update_tracked(&input, always, always);
			assert!(
				captured.iter().any(|m| *m == s.method),
				"snap not captured after {hover_frames} hover frame(s)"
			);
		}
	}

	/// Interact_condition becoming true while snap is already interacting must not
	/// trigger a second capture (no duplicate captures).
	#[tokio::test]
	async fn no_duplicate_capture_while_interacting() {
		let device = make_device();
		let mut action = MultiAction::default();
		let s = make_snapshot(&device, false);
		let input = make_input([s.clone()]);

		action.test_update_tracked(&input, always, never); // hover frame 1
		action.test_update_tracked(&input, always, never); // hover frame 2 — clears hover.added
		let (first, _) = action.test_update_tracked(&input, always, always); // capture
		assert_eq!(first.len(), 1, "expected exactly one capture");

		for _ in 0..3 {
			let (again, _) = action.test_update_tracked(&input, always, always);
			assert!(again.is_empty(), "duplicate capture on sustained interact");
		}
	}

	/// Stopping interact_condition must produce a release for every snap that was
	/// captured, across various hover-frame counts.
	#[tokio::test]
	async fn release_always_follows_capture() {
		for hover_frames in 2usize..=5 {
			let device = make_device();
			let mut action = MultiAction::default();
			let s = make_snapshot(&device, false);
			let input = make_input([s.clone()]);

			for _ in 0..hover_frames {
				action.test_update_tracked(&input, always, never);
			}
			action.test_update_tracked(&input, always, always); // capture

			let (_, released) = action.test_update_tracked(&input, always, never);
			assert!(
				released.iter().any(|m| *m == s.method),
				"snap not released after interact stops (hover_frames={hover_frames})"
			);
		}
	}

	/// With multiple snaps at different hover depths, capture fires only for the snap
	/// that has cleared hover.added (been hovering ≥2 frames), not for one that just entered.
	#[tokio::test]
	async fn independent_capture_timing_per_snap() {
		let device = make_device();
		let mut action = MultiAction::default();

		let early = make_snapshot(&device, false); // gets 2 hover frames before interact fires
		let late = make_snapshot(&device, false); // enters on frame 2, only 1 hover frame before interact

		// Frame 1: only early hovers
		action.test_update_tracked(&make_input([early.clone()]), always, never);

		// Frame 2: late enters; early now has 2 hover frames (hover.added cleared after frame 1)
		action.test_update_tracked(&make_input([early.clone(), late.clone()]), always, never);

		// Frame 3: interact fires for both; early should capture (2 hover frames),
		// late should NOT (hover.added still contains late from frame 2)
		let (cap3, _) =
			action.test_update_tracked(&make_input([early.clone(), late.clone()]), always, always);
		assert!(
			cap3.iter().any(|m| *m == early.method),
			"early snap not captured after 2 hover frames"
		);
		assert!(
			!cap3.iter().any(|m| *m == late.method),
			"late snap incorrectly captured on its first hover frame"
		);
	}

	#[tokio::test]
	async fn interact_removed_when_condition_stops() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let mut action = MultiAction::default();
		action.test_update(&make_input([s1.clone()]), always, never);

		let s1_cap = snap_with_method(s1.method.clone(), true);
		action.test_update(&make_input([s1_cap.clone()]), always, always);
		action.test_update(&make_input([s1_cap.clone()]), always, always);
		assert!(action.interact().current().contains(&s1_cap));

		// Interact condition no longer met
		let s1_rel = snap_with_method(s1.method.clone(), false);
		action.test_update(&make_input([s1_rel.clone()]), always, never);
		assert!(action.interact().removed().contains(&s1_cap));
		assert!(action.interact().current().is_empty());
	}
}
