mod single_action;
use glam::Vec3;
pub use single_action::*;
mod simple_action;
pub use simple_action::*;
mod multi_action;
pub use multi_action::*;

use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	input::{
		InputData, InputDataType, InputHandler, InputHandlerAspect, InputHandlerEvent,
		InputMethodRef, InputMethodRefAspect, Pointer,
	},
	node::NodeResult,
	values::Vector3,
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
		Ok(InputQueue {
			handler: self,
			input: FxHashMap::default(),
		})
	}
}

pub struct InputQueue {
	handler: InputHandler,
	input: FxHashMap<Arc<InputData>, InputMethodRef>,
}
impl InputQueue {
	pub fn handler(&self) -> &InputHandler {
		&self.handler
	}
	pub fn input(&self) -> FxHashMap<Arc<InputData>, &InputMethodRef> {
		FxHashMap::from_iter(self.input.iter().map(|(i, m)| (i.clone(), m)))
	}
	pub fn request_capture(&self, data: &Arc<InputData>) {
		let input = self.input();
		let Some(method) = input.get(data) else {
			return;
		};
		let _ = method.request_capture(self.handler());
	}

	// check this as often as possible, will return true when input has been updated
	pub fn handle_events(&mut self) -> bool {
		let mut updated = false;
		while let Some(InputHandlerEvent::Input { methods, data }) =
			self.handler.recv_input_handler_event()
		{
			updated = true;
			self.input = data.into_iter().map(Arc::new).zip(methods).collect();
		}
		updated
	}
}
impl Debug for InputQueue {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		self.input().keys().fmt(f)
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

pub fn grab_pinch_interact(data: &InputData) -> bool {
	data.datamap.with_data(|datamap| match &data.input {
		InputDataType::Hand(_) => datamap.idx("pinch_strength").as_f32() > 0.90,
		_ => datamap.idx("grab").as_f32() > 0.90,
	})
}
pub fn select_pinch_interact(data: &InputData) -> bool {
	data.datamap.with_data(|datamap| match &data.input {
		InputDataType::Hand(_) => datamap.idx("pinch_strength").as_f32() > 0.90,
		_ => datamap.idx("select").as_f32() > 0.90,
	})
}

pub trait PointerExt {
	fn intersect_plane(&self, normal: Vector3<f32>) -> Vector3<f32>;
}
impl PointerExt for Pointer {
	fn intersect_plane(&self, normal: Vector3<f32>) -> Vector3<f32> {
		let normal: Vec3 = normal.into();
		let denom = normal.dot(self.direction().into());
		let t = -Vec3::from(self.origin).dot(normal) / denom;
		let p = Vec3::from(self.origin) + Vec3::from(self.direction()) * t;
		p.into()
	}
}
