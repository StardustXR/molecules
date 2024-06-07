mod single_actor_action;
pub use single_actor_action::*;
mod multi_actor_action;
pub use multi_actor_action::*;

use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	input::{
		InputData, InputHandler, InputHandlerAspect, InputHandlerHandler, InputMethodRef,
		InputMethodRefAspect,
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
		Ok(InputQueue(self.wrap(InputQueueInternal::default())?))
	}
}

pub struct InputQueue(HandlerWrapper<InputHandler, InputQueueInternal>);
impl InputQueue {
	pub fn handler(&self) -> &InputHandler {
		self.0.node()
	}
	pub fn input(&self) -> FxHashMap<Arc<InputData>, InputMethodRef> {
		let mut locked = self.0.lock_wrapped();
		FxHashMap::from_iter(
			locked
				.get_queued()
				.iter()
				.map(|(i, m)| (i.clone(), m.alias())),
		)
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
		self.0.lock_wrapped().0.keys().fmt(f)
	}
}

#[derive(Default, Debug)]
pub struct InputQueueInternal(FxHashMap<Arc<InputData>, InputMethodRef>);
impl InputQueueInternal {
	fn get_queued<'a>(&mut self) -> &FxHashMap<Arc<InputData>, InputMethodRef> {
		&self.0
	}
}
impl InputHandlerHandler for InputQueueInternal {
	// TODO: put all input handling and reaction in here
	fn input(&mut self, input: Vec<InputMethodRef>, data: Vec<InputData>) {
		self.0 = data
			.into_iter()
			.map(Arc::new)
			.zip(input.into_iter())
			.collect();
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
