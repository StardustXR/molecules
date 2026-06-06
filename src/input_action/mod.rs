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
	Result,
	client::{Client, ClientHandler},
	fields::{Field, FieldRef},
	query::{QueryExt, QueryableInterfaceGuard, QueryableObject},
	spatial::{Spatial, SpatialRef},
	suis::{
		DatamapData, InputDataType, InputHandler as InputHandlerProxy, InputHandlerHandler,
		InputMethod, InputMethodCapture, SemanticData, SpatialData,
	},
	types::Timestamp,
};
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
	sync::{Arc, Mutex, OnceLock},
};
use tokio::sync::mpsc;

/// Snapshot of a single input method's state at the time of a callback.
#[derive(Debug)]
pub struct InputSnapshot {
	pub method: InputMethod,
	pub spatial: SpatialData,
	pub semantic: SemanticData,
	pub time: Timestamp,
}
impl InputSnapshot {
	pub fn distance(&self) -> f32 {
		self.spatial.distance
	}
	pub fn input(&self) -> &InputDataType {
		&self.spatial.input
	}
	pub fn captured(&self) -> bool {
		self.semantic.captured
	}
	pub fn datamap_f32(&self, key: &str) -> f32 {
		match self.semantic.datamap.get(key) {
			Some(DatamapData::Float { value }) => *value,
			_ => 0.0,
		}
	}
	pub fn datamap_bool(&self, key: &str) -> bool {
		match self.semantic.datamap.get(key) {
			Some(DatamapData::Bool { value }) => *value,
			_ => false,
		}
	}
	pub fn datamap_vec2(&self, key: &str) -> [f32; 2] {
		match self.semantic.datamap.get(key) {
			Some(DatamapData::Vec2 { value }) => [value.x, value.y],
			_ => [0.0, 0.0],
		}
	}
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
	capture_requests: FxHashMap<InputMethod, InputMethodCapture>,
	capture_rx: mpsc::UnboundedReceiver<(InputMethod, Option<InputMethodCapture>)>,
	dirty: bool,
}

#[derive(Handler)]
pub struct InputQueue {
	field: FieldRef,
	reference_space: SpatialRef,
	handler_proxy: OnceLock<InputHandlerProxy>,
	_queryable: OnceLock<QueryableObject>,
	_interface_guard: OnceLock<QueryableInterfaceGuard>,
	inner: Mutex<InputQueueState>,
	capture_tx: mpsc::UnboundedSender<(InputMethod, Option<InputMethodCapture>)>,
}
impl Debug for InputQueue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
	) -> Result<Object<InputQueue>> {
		let (capture_tx, capture_rx) = mpsc::unbounded_channel();
		let queue = InputQueue {
			field: field.field_ref().await?,
			reference_space,
			handler_proxy: OnceLock::new(),
			_queryable: OnceLock::new(),
			_interface_guard: OnceLock::new(),
			inner: Mutex::new(InputQueueState {
				current: FxHashMap::default(),
				capture_requests: FxHashMap::default(),
				capture_rx,
				dirty: false,
			}),
			capture_tx,
		};
		let queue_obj = client.pion_device().register_object(queue);
		let proxy = InputHandlerProxy::from_handler(&queue_obj);
		let _ = queue_obj.handler_proxy.set(proxy);

		let queryable = QueryableObject::create(client, query_spatial, field).await?;
		let guard = queryable
			.add_interface(&queue_obj, InputHandlerProxy::QUERY_INTERFACE)
			.await?;
		let _ = queue_obj._queryable.set(queryable);
		let _ = queue_obj._interface_guard.set(guard);

		Ok(queue_obj)
	}

	/// Returns `true` if any input arrived or left since the last call. Resets the dirty flag.
	pub fn handle_events(&self) -> bool {
		let mut s = self.inner.lock().unwrap();
		while let Ok((method, capture)) = s.capture_rx.try_recv() {
			match capture {
				Some(c) => {
					if s.current.contains_key(&method) {
						s.capture_requests.insert(method, c);
					}
				}
				None => {
					s.capture_requests.remove(&method);
				}
			}
		}
		let dirty = s.dirty;
		s.dirty = false;
		dirty
	}

	/// Current snapshot of all active input methods.
	pub fn input(&self) -> FxHashMap<InputMethod, Arc<InputSnapshot>> {
		self.inner.lock().unwrap().current.clone()
	}

	#[cfg(test)]
	pub fn new_test(
		field: stardust_xr_fusion::fields::FieldRef,
		reference_space: SpatialRef,
	) -> Self {
		let (capture_tx, capture_rx) = mpsc::unbounded_channel();
		InputQueue {
			field,
			reference_space,
			handler_proxy: OnceLock::new(),
			_queryable: OnceLock::new(),
			_interface_guard: OnceLock::new(),
			inner: Mutex::new(InputQueueState {
				current: FxHashMap::default(),
				capture_requests: FxHashMap::default(),
				capture_rx,
				dirty: false,
			}),
			capture_tx,
		}
	}

	#[cfg(test)]
	pub fn inject(&self, snap: Arc<InputSnapshot>) {
		let mut s = self.inner.lock().unwrap();
		s.current.insert(snap.method.clone(), snap);
		s.dirty = true;
	}

	#[cfg(test)]
	pub fn remove_method(&self, method: &InputMethod) {
		let mut s = self.inner.lock().unwrap();
		s.current.remove(method);
		s.dirty = true;
	}

	pub fn start_capture(&self, snap: &InputSnapshot) {
		let Some(proxy) = self.handler_proxy.get() else {
			return;
		};
		let method = snap.method.clone();
		let proxy = proxy.clone();
		let tx = self.capture_tx.clone();
		tokio::spawn(async move {
			let capture = method.request_capture(proxy).await.ok().flatten();
			let _ = tx.send((method, capture));
		});
	}

	pub fn release_capture(&self, snap: &InputSnapshot) {
		self.inner
			.lock()
			.unwrap()
			.capture_requests
			.remove(&snap.method);
	}
}

impl InputHandlerHandler for InputQueue {
	async fn get_spatial(&self, _ctx: Context) -> SpatialRef {
		self.reference_space.clone()
	}
	async fn get_field(&self, _ctx: Context) -> FieldRef {
		self.field.clone()
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
		s.capture_requests.remove(&method);
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
