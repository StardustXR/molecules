use super::{DeltaSet, InputQueue};
use rustc_hash::FxHashSet;
use stardust_xr_fusion::input::{InputData, InputMethodRefAspect};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct MultiActorAction(DeltaSet<Arc<InputData>>);
impl MultiActorAction {
	pub fn update(
		&mut self,
		capture_on_trigger: bool,
		queue: &InputQueue,
		active_condition: impl Fn(&InputData) -> bool,
	) {
		self.0.push_new(
			queue
				.input()
				.iter()
				// filter out every input method that doesn't meet the active condition
				.filter(|(d, _)| (active_condition)(d))
				// now capture everything that is interacting if we capture on trigger
				.inspect(|(_, m)| {
					if capture_on_trigger {
						let _ = m.request_capture(queue.handler());
					}
				})
				// and filter out any input methods that haven't yet been captured when we want them to be
				.filter(|(m, _)| !capture_on_trigger || m.captured)
				// we don't need the input method anymore
				.map(|(d, _)| d)
				.cloned(),
		);
	}
	pub fn started_acting(&self) -> &FxHashSet<Arc<InputData>> {
		self.0.added()
	}
	pub fn currently_acting(&self) -> &FxHashSet<Arc<InputData>> {
		self.0.current()
	}
	pub fn stopped_acting(&self) -> &FxHashSet<Arc<InputData>> {
		self.0.removed()
	}
}
