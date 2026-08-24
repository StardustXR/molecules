mod single_action;
use futures::FutureExt;
pub use single_action::*;
mod simple_action;
pub use simple_action::*;
mod multi_action;
pub use multi_action::*;

use glam::Vec3;
use gluon::{Context, Handler, Node, RefExt};
use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::{Field, FieldRef},
	query::{QueryableExt, QueryableInterface, QueryableObject},
	spatial::{Spatial, SpatialRef},
	suis::{
		DatamapData, InputDataType, InputHandler as InputHandlerProxy, InputHandlerHandler,
		InputMethod, InputMethodCapture, SemanticData, SpatialData,
	},
	types::{Timestamp, Vec2F},
};
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
	sync::{Arc, Mutex, OnceLock},
};
use tokio::task::JoinHandle;

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
	pub fn datamap_vec2(&self, key: &str) -> Vec2F {
		match self.semantic.datamap.get(key) {
			Some(DatamapData::Vec2 { value }) => *value,
			_ => [0.0, 0.0].into(),
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

pub struct InputQueue(Node<InputQueueInner>, InputHandlerProxy);

struct InputQueueState {
	current: FxHashMap<InputMethod, Arc<InputSnapshot>>,
	capture_queue: FxHashMap<InputMethod, JoinHandle<Option<InputMethodCapture>>>,
	capture_requests: FxHashMap<InputMethod, InputMethodCapture>,
	dirty: bool,
}

#[derive(Handler)]
struct InputQueueInner {
	field: FieldRef,
	reference_space: SpatialRef,
	_queryable: OnceLock<QueryableObject>,
	_interface: OnceLock<QueryableInterface>,
	state: Mutex<InputQueueState>,
}
impl Debug for InputQueueInner {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("InputQueue")
			.field(
				"current",
				&self
					.state
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
	) -> Result<Self> {
		let queue = InputQueueInner {
			field: field.field_ref().await?,
			reference_space,
			_queryable: OnceLock::new(),
			_interface: OnceLock::new(),
			state: Mutex::new(InputQueueState {
				current: FxHashMap::default(),
				capture_requests: FxHashMap::default(),
				capture_queue: FxHashMap::default(),
				dirty: false,
			}),
		};
		let (queue_node, queue) = InputHandlerProxy::new_node(queue)?;

		let queryable = QueryableObject::new(client, query_spatial, field).await?;
		let interface = queryable
			.add_interface(&queue, InputHandlerProxy::QUERY_INTERFACE)
			.await??;
		let _ = queue_node._queryable.set(queryable);
		let _ = queue_node._interface.set(interface);

		Ok(InputQueue(queue_node, queue.into_proxy()))
	}

	/// Returns `true` if any input arrived or left since the last call. Resets the dirty flag.
	pub fn handle_events(&self) -> bool {
		let mut s = self.0.state.lock().unwrap();
		let s = &mut *s;

		// drain the queue of all capture requests that got their handler
		s.capture_requests.extend(
			s.capture_queue
				.extract_if(|_, v| v.is_finished())
				.filter_map(|(method, capture)| Some((method, capture.now_or_never()?.ok()??)))
				.filter(|(method, _)| s.current.contains_key(method)),
		);

		let dirty = s.dirty;
		s.dirty = false;
		dirty
	}

	/// Current snapshot of all active input methods.
	pub fn input(&self) -> FxHashMap<InputMethod, Arc<InputSnapshot>> {
		self.0.state.lock().unwrap().current.clone()
	}

	pub fn start_capture(&self, snap: &InputSnapshot) {
		let method = snap.method.clone();
		let proxy = self.1.clone();
		let capture_return_task =
			tokio::spawn(async move { method.request_capture(proxy).await.ok().flatten() });
		self.0
			.state
			.lock()
			.unwrap()
			.capture_queue
			.insert(snap.method.clone(), capture_return_task);
	}

	pub fn release_capture(&self, snap: &InputSnapshot) {
		self.0
			.state
			.lock()
			.unwrap()
			.capture_requests
			.remove(&snap.method);
	}
}

impl InputHandlerHandler for InputQueueInner {
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
		let mut s = self.state.lock().unwrap();
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
		let mut s = self.state.lock().unwrap();
		s.current.insert(method, snap);
		s.dirty = true;
	}
	async fn input_left(&self, _ctx: Context, method: InputMethod, _time: Timestamp) {
		let mut s = self.state.lock().unwrap();
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
