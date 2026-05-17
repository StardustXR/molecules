mod single_action;
pub use single_action::*;
mod simple_action;
pub use simple_action::*;
mod multi_action;
pub use multi_action::*;

use glam::Vec3;
use gluon::{Context, Handler, Object};
use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	error::ServerError,
	fields::{Field, FieldRef},
	query::{QueryExt, QueryableObject},
	spatial::{Spatial, SpatialRef},
	suis::{
		DatamapData, InputDataType, InputHandler as InputHandlerProxy, InputHandlerHandler,
		InputMethod, SemanticData, SpatialData,
	},
	types::Timestamp,
};
use std::{
	any::Any,
	fmt::{Debug, Formatter, Result},
	hash::Hash,
	sync::{Arc, Mutex, OnceLock},
};

/// Snapshot of a single input method's state at the time of a callback.
#[derive(Debug)]
pub struct InputSnapshot {
	pub method: InputMethod,
	pub spatial: SpatialData,
	pub semantic: SemanticData,
	pub time: Timestamp,
}
impl PartialEq for InputSnapshot {
	fn eq(&self, other: &Self) -> bool {
		self.method == other.method
	}
}
impl Eq for InputSnapshot {}
impl Hash for InputSnapshot {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.method.hash(state);
	}
}

struct InputQueueState {
	current: FxHashMap<InputMethod, Arc<InputSnapshot>>,
	dirty: bool,
}

#[derive(Handler)]
pub struct InputQueue {
	field: FieldRef,
	reference_space: SpatialRef,
	handler_proxy: OnceLock<InputHandlerProxy>,
	_interface_guard: OnceLock<Box<dyn Any + Send + Sync>>,
	inner: Mutex<InputQueueState>,
}
impl Debug for InputQueue {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.debug_struct("InputQueue")
			.field(
				"current",
				&self
					.inner
					.lock()
					.unwrap()
					.current
					.keys()
					.collect::<Vec<_>>(),
			)
			.finish()
	}
}

impl InputQueue {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		query_spatial: Spatial,
		field: Field,
		reference_space: SpatialRef,
	) -> std::result::Result<Object<InputQueue>, ServerError> {
		let queue = InputQueue {
			field: field.field_ref().await?,
			reference_space,
			handler_proxy: OnceLock::new(),
			_interface_guard: OnceLock::new(),
			inner: Mutex::new(InputQueueState {
				current: FxHashMap::default(),
				dirty: false,
			}),
		};
		let queue_obj = client.pion_device().register_object(queue);
		let proxy = InputHandlerProxy::from_handler(&queue_obj);
		let _ = queue_obj.handler_proxy.set(proxy);

		let queryable = QueryableObject::new(client, query_spatial, field).await?;
		let guard = queryable
			.unwrap()
			.add_interface(&queue_obj, InputHandlerProxy::QUERY_INTERFACE)
			.await?;
		let _ = queue_obj
			._interface_guard
			.set(Box::new(guard) as Box<dyn Any + Send + Sync>);

		Ok(queue_obj)
	}

	/// Returns `true` if any input arrived or left since the last call. Resets the dirty flag.
	pub fn handle_events(&self) -> bool {
		let mut s = self.inner.lock().unwrap();
		let dirty = s.dirty;
		s.dirty = false;
		dirty
	}

	/// Current snapshot of all active input methods.
	pub fn input(&self) -> FxHashMap<InputMethod, Arc<InputSnapshot>> {
		self.inner.lock().unwrap().current.clone()
	}

	pub fn start_capture(&self, snap: &InputSnapshot) {
		let Some(proxy) = self.handler_proxy.get() else {
			return;
		};
		let _ = snap.method.request_capture(proxy.clone());
	}

	pub fn release_capture(&self, snap: &InputSnapshot) {
		let Some(proxy) = self.handler_proxy.get() else {
			return;
		};
		let _ = snap.method.release_capture(proxy.clone());
	}
}

impl InputHandlerHandler for InputQueue {
	async fn get_spatial(&self, _ctx: Context) -> SpatialRef {
		self.reference_space.clone()
	}
	async fn get_field(&self, _ctx: Context) -> FieldRef {
		self.field.clone()
	}
	async fn suggested_bindings(
		&self,
		_ctx: Context,
	) -> std::collections::HashMap<String, Vec<String>> {
		Default::default()
	}
	async fn handler_groups(&self, _ctx: Context) -> Vec<String> {
		vec![]
	}
	async fn input_gained(
		&self,
		_ctx: Context,
		method: InputMethod,
		time: Timestamp,
		spatial: SpatialData,
		semantic: SemanticData,
	) {
		let snap = Arc::new(InputSnapshot {
			method: method.clone(),
			spatial,
			semantic,
			time,
		});
		let mut s = self.inner.lock().unwrap();
		s.current.insert(method, snap);
		s.dirty = true;
	}
	async fn input_updated(
		&self,
		_ctx: Context,
		method: InputMethod,
		time: Timestamp,
		spatial: SpatialData,
		semantic: SemanticData,
	) {
		let snap = Arc::new(InputSnapshot {
			method: method.clone(),
			spatial,
			semantic,
			time,
		});
		let mut s = self.inner.lock().unwrap();
		s.current.insert(method, snap);
		s.dirty = true;
	}
	async fn input_left(&self, _ctx: Context, method: InputMethod, _time: Timestamp) {
		let mut s = self.inner.lock().unwrap();
		s.current.remove(&method);
		s.dirty = true;
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

pub fn grab_pinch_interact(snap: &InputSnapshot) -> bool {
	let f32_val = |key: &str| matches!(snap.semantic.datamap.get(key), Some(DatamapData::Float { value }) if *value > 0.90);
	match &snap.spatial.input {
		InputDataType::Hand { .. } => f32_val("pinch_strength"),
		_ => f32_val("grab"),
	}
}

pub fn select_pinch_interact(snap: &InputSnapshot) -> bool {
	let f32_val = |key: &str| matches!(snap.semantic.datamap.get(key), Some(DatamapData::Float { value }) if *value > 0.90);
	match &snap.spatial.input {
		InputDataType::Hand { .. } => f32_val("pinch_strength"),
		_ => f32_val("select"),
	}
}

pub trait PointerExt {
	fn intersect_plane(&self, normal: Vec3) -> Vec3;
}
impl PointerExt for stardust_xr_fusion::suis::Pointer {
	fn intersect_plane(&self, normal: Vec3) -> Vec3 {
		let origin: Vec3 = self.pose.position.into();
		let dir: Vec3 = Vec3::from(self.direction());
		let t = -origin.dot(normal) / normal.dot(dir);
		origin + dir * t
	}
}
