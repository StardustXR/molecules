mod single_actor_action;
pub use single_actor_action::*;
mod multi_actor_action;
pub use multi_actor_action::*;

use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	input::{
		InputData, InputHandler, InputHandlerAspect, InputHandlerHandler, InputMethod,
		InputMethodAspect,
	},
	node::{NodeResult, NodeType},
	HandlerWrapper,
};
use std::{
	fmt::{Debug, Formatter, Result},
	hash::Hash,
	sync::Arc,
};

pub trait InputQueueable: Sized {
	fn queue(self) -> NodeResult<InputQueue>;
}
impl InputQueueable for InputHandler {
	fn queue(self) -> NodeResult<InputQueue> {
		Ok(InputQueue(self.wrap(InputQueueInternal {
			flush_queue: false,
			queued_input: Default::default(),
		})?))
	}
}

pub struct InputQueue(HandlerWrapper<InputHandler, InputQueueInternal>);
impl InputQueue {
	pub fn handler(&self) -> &InputHandler {
		self.0.node()
	}
	pub fn input(&self) -> FxHashMap<Arc<InputData>, InputMethod> {
		let mut locked = self.0.lock_wrapped();
		FxHashMap::from_iter(
			locked
				.get_queued()
				.iter()
				.map(|(i, m)| (i.clone(), m.alias())),
		)
	}
	pub fn flush_queue(&self) {
		let mut lock = self.0.lock_wrapped();
		lock.queued_input.clear();
		lock.flush_queue = false;
	}
	pub fn request_capture(&self, data: &Arc<InputData>) {
		let input = self.input();
		let Some(method) = input.get(data) else {
			return;
		};
		let _ = method.request_capture(self.handler());
	}
}
impl Debug for InputQueue {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		self.0.lock_wrapped().queued_input.keys().fmt(f)
	}
}

#[derive(Default, Debug)]
pub struct InputQueueInternal {
	flush_queue: bool,
	queued_input: FxHashMap<Arc<InputData>, InputMethod>,
}
impl InputQueueInternal {
	fn get_queued<'a>(&mut self) -> &FxHashMap<Arc<InputData>, InputMethod> {
		// make it so we can do any amount of update_action and not clear anything until we get next input
		self.flush_queue = true;
		&self.queued_input
	}
}
impl InputHandlerHandler for InputQueueInternal {
	fn input(&mut self, input: InputMethod, data: InputData) {
		if self.flush_queue {
			self.queued_input.clear();
			self.flush_queue = false;
		}
		self.queued_input.insert(Arc::new(data), input);
	}
}

pub struct DeltaSet<T: Clone + Hash + Eq> {
	added: FxHashSet<T>,
	current: FxHashSet<T>,
	removed: FxHashSet<T>,
}
impl<T: Clone + Hash + Eq> Default for DeltaSet<T> {
	fn default() -> Self {
		DeltaSet {
			added: Default::default(),
			current: Default::default(),
			removed: Default::default(),
		}
	}
}
impl<T: Clone + Hash + Eq + Debug> Debug for DeltaSet<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DeltaSet")
			.field("added", &self.added)
			.field("current", &self.current)
			.field("removed", &self.removed)
			.finish()
	}
}
impl<T: Clone + Hash + Eq> DeltaSet<T> {
	pub fn push_new(&mut self, new: impl Iterator<Item = T>) {
		let new = FxHashSet::from_iter(new);
		self.added = FxHashSet::from_iter(new.difference(&self.current).cloned());
		self.removed = FxHashSet::from_iter(self.current.difference(&new).cloned());
		self.current = new;
	}
	pub fn added(&self) -> &FxHashSet<T> {
		&self.added
	}
	pub fn current(&self) -> &FxHashSet<T> {
		&self.current
	}
	pub fn removed(&self) -> &FxHashSet<T> {
		&self.removed
	}
}
