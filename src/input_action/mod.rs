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
use stardust_xr_protocol::query::QueryableInterfaceGuard;
use std::{
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
			_queryable: OnceLock::new(),
			_interface_guard: OnceLock::new(),
			inner: Mutex::new(InputQueueState {
				current: FxHashMap::default(),
				dirty: false,
			}),
		};
		let queue_obj = client.pion_device().register_object(queue);
		let proxy = InputHandlerProxy::from_handler(&queue_obj);
		let _ = queue_obj.handler_proxy.set(proxy);

		let queryable = QueryableObject::create(client, query_spatial, field)
			.await?
			.unwrap();
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
		InputQueue {
			field,
			reference_space,
			handler_proxy: OnceLock::new(),
			_queryable: OnceLock::new(),
			_interface_guard: OnceLock::new(),
			inner: Mutex::new(InputQueueState {
				current: FxHashMap::default(),
				dirty: false,
			}),
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

#[cfg(test)]
pub(crate) mod test_helpers {
	use super::*;
	use glam::{Quat, Vec3};
	use pion_binder::PionBinderDevice;
	use stardust_xr_fusion::{
		fields::FieldRef,
		suis::{InputDataType, InputHandler, InputMethodHandler, SemanticData, SpatialData, Tip},
		types::{Posef, Timestamp},
	};

	#[derive(Debug, gluon::Handler)]
	pub struct DummyMethod;
	impl InputMethodHandler for DummyMethod {
		async fn request_capture(&self, _: gluon::Context, _: InputHandler) {}
		async fn release_capture(&self, _: gluon::Context, _: InputHandler) {}
		async fn get_spatial_data(
			&self,
			_: gluon::Context,
			_: InputHandler,
			_: Timestamp,
		) -> Option<SpatialData> {
			None
		}
	}

	pub fn make_device() -> PionBinderDevice {
		PionBinderDevice::default()
	}

	/// Create a fresh snap with a unique InputMethod each call.
	pub fn make_snapshot(device: &PionBinderDevice, captured: bool) -> Arc<InputSnapshot> {
		let obj: gluon::Object<DummyMethod> = device.register_object(Arc::new(DummyMethod));
		let method = InputMethod::from_handler(&obj);
		snap_with_method(method, captured)
	}

	/// Create an updated snap reusing an existing method identity.
	pub fn snap_with_method(method: InputMethod, captured: bool) -> Arc<InputSnapshot> {
		let pose = Posef {
			position: Vec3::ZERO.into(),
			orientation: Quat::IDENTITY.into(),
		};
		Arc::new(InputSnapshot {
			method,
			spatial: SpatialData {
				input: InputDataType::Tip {
					data: Tip {
						pose,
						chirality: None,
						grip_pose: None,
						grip_surface_pose: None,
						simulated_hand: None,
					},
				},
				distance: 0.0,
			},
			semantic: SemanticData {
				datamap: Default::default(),
				order: 0,
				captured,
			},
			time: Timestamp {
				seconds: 0,
				nanoseconds: 0,
			},
		})
	}

	pub fn make_input(
		snaps: impl IntoIterator<Item = Arc<InputSnapshot>>,
	) -> FxHashMap<InputMethod, Arc<InputSnapshot>> {
		snaps.into_iter().map(|s| (s.method.clone(), s)).collect()
	}

	/// Create a bare `InputQueue` for testing `handle_events` / `inject`.
	pub fn make_queue(device: &PionBinderDevice) -> InputQueue {
		let obj: gluon::Object<DummyMethod> = device.register_object(Arc::new(DummyMethod));
		let method = InputMethod::from_handler(&obj);
		let or_ref: gluon::ObjectOrRef = method.into();
		let field_ref = FieldRef::from_object_or_ref(or_ref.clone());
		let spatial_ref = SpatialRef::from_object_or_ref(or_ref);
		InputQueue::new_test(field_ref, spatial_ref)
	}
}

#[cfg(test)]
mod tests {
	use super::{test_helpers::*, *};

	#[tokio::test]
	async fn delta_set_tracks_added_removed_current() {
		let device = make_device();
		let s1 = make_snapshot(&device, false);
		let s2 = make_snapshot(&device, false);
		let mut ds: DeltaSet<Arc<InputSnapshot>> = DeltaSet::default();

		ds.push_new([s1.clone(), s2.clone()].into_iter());
		assert_eq!(ds.added().len(), 2);
		assert_eq!(ds.current().len(), 2);
		assert!(ds.removed().is_empty());

		ds.push_new([s1.clone()].into_iter());
		assert!(ds.added().is_empty());
		assert!(ds.current().contains(&s1));
		assert!(ds.removed().contains(&s2));
	}

	#[tokio::test]
	async fn input_queue_dirty_flag() {
		let device = make_device();
		let queue = make_queue(&device);

		assert!(!queue.handle_events());

		let snap = make_snapshot(&device, false);
		queue.inject(snap.clone());
		assert!(queue.handle_events());
		assert!(!queue.handle_events());

		queue.remove_method(&snap.method);
		assert!(queue.handle_events());
		assert!(!queue.handle_events());
	}

	#[tokio::test]
	async fn input_queue_inject_and_retrieve() {
		let device = make_device();
		let queue = make_queue(&device);
		let s1 = make_snapshot(&device, false);
		let s2 = make_snapshot(&device, false);

		queue.inject(s1.clone());
		queue.inject(s2.clone());
		let input = queue.input();
		assert_eq!(input.len(), 2);
		assert!(input.contains_key(&s1.method));
		assert!(input.contains_key(&s2.method));

		queue.remove_method(&s1.method);
		assert_eq!(queue.input().len(), 1);
	}
}
