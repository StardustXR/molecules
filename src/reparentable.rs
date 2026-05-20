use gluon::{Context, Object};
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	error::ServerError,
	fields::Field,
	query::{QueryExt, QueryableObject},
	spatial::{PartialTransform, Spatial, SpatialRef},
};
use stardust_xr_molecules_protocols::reparentable::{
	EXTERNAL_PROTOCOL, ReparentHandle, ReparentHandleHandler, ReparentKeepalive,
	ReparentableHandler,
};
use std::{
	any::Any,
	sync::{
		Arc, Mutex,
		atomic::{AtomicBool, Ordering},
	},
};

enum ReparentState {
	Idle,
	NonLocked(ReparentKeepalive),
	Locked(#[allow(dead_code)] ReparentKeepalive),
}

struct SharedState {
	spatial: Spatial,
	initial_parent: SpatialRef,
	reparent_state: Mutex<ReparentState>,
	reparented: AtomicBool,
}

#[derive(gluon::Handler)]
struct ReparentHandleInner(Arc<SharedState>);
impl std::fmt::Debug for ReparentHandleInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReparentHandleInner").finish()
	}
}
impl ReparentHandleHandler for ReparentHandleInner {
	async fn reset_transform(&self, _ctx: Context, relative_to: SpatialRef) {
		let _ = self
			.0
			.spatial
			.set_relative_transform(relative_to, PartialTransform::NONE);
	}
}

#[derive(gluon::Handler)]
struct ReparentableInner {
	state: Arc<SharedState>,
	handle_proxy: std::sync::OnceLock<ReparentHandle>,
	_handle_obj: std::sync::OnceLock<Object<ReparentHandleInner>>,
}
impl std::fmt::Debug for ReparentableInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReparentableInner").finish()
	}
}
impl ReparentableHandler for ReparentableInner {
	async fn reparent_locking(
		&self,
		_ctx: Context,
		new_parent: SpatialRef,
		keepalive: ReparentKeepalive,
	) -> Option<ReparentHandle> {
		let mut guard = self.state.reparent_state.lock().unwrap();
		match &*guard {
			ReparentState::Idle | ReparentState::NonLocked(_) => {
				if let ReparentState::NonLocked(old) = &*guard {
					let _ = old.reparent_stolen();
				}
				let _ = self.state.spatial.set_parent_in_place(new_parent);
				self.state.reparented.store(true, Ordering::Relaxed);
				*guard = ReparentState::Locked(keepalive);
				self.handle_proxy.get().cloned()
			}
			ReparentState::Locked(_) => None,
		}
	}
	async fn reparent(
		&self,
		_ctx: Context,
		new_parent: SpatialRef,
		keepalive: ReparentKeepalive,
	) -> Option<ReparentHandle> {
		let mut guard = self.state.reparent_state.lock().unwrap();
		match &*guard {
			ReparentState::Idle | ReparentState::NonLocked(_) => {
				if let ReparentState::NonLocked(old) = &*guard {
					let _ = old.reparent_stolen();
				}
				let _ = self.state.spatial.set_parent_in_place(new_parent);
				self.state.reparented.store(true, Ordering::Relaxed);
				*guard = ReparentState::NonLocked(keepalive);
				self.handle_proxy.get().cloned()
			}
			ReparentState::Locked(_) => None,
		}
	}
}

pub struct Reparentable {
	state: Arc<SharedState>,
	_obj: Object<ReparentableInner>,
	_guards: Vec<Box<dyn Any + Send + Sync>>,
}
impl Reparentable {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		initial_parent: SpatialRef,
		field: Field,
	) -> Result<Self, ServerError> {
		let _ = spatial.set_parent_in_place(initial_parent.clone());
		let state = Arc::new(SharedState {
			spatial: spatial.clone(),
			initial_parent: initial_parent.clone(),
			reparent_state: Mutex::new(ReparentState::Idle),
			reparented: AtomicBool::new(false),
		});

		let handle_inner = ReparentHandleInner(state.clone());
		let handle_obj = client.pion_device().register_object(handle_inner);
		let handle_proxy = ReparentHandle::from_handler(&handle_obj);

		let reparentable_inner = ReparentableInner {
			state: state.clone(),
			handle_proxy: std::sync::OnceLock::new(),
			_handle_obj: std::sync::OnceLock::new(),
		};
		let reparentable_obj = client.pion_device().register_object(reparentable_inner);
		let _ = reparentable_obj.handle_proxy.set(handle_proxy);
		let _ = reparentable_obj._handle_obj.set(handle_obj);

		let queryable = QueryableObject::create(client, spatial, field).await?;
		let guard = queryable
			.unwrap()
			.add_interface(&reparentable_obj, EXTERNAL_PROTOCOL.protocol_name)
			.await?;

		Ok(Reparentable {
			state,
			_obj: reparentable_obj,
			_guards: vec![Box::new(guard) as Box<dyn Any + Send + Sync>],
		})
	}

	pub fn reparented(&self) -> bool {
		self.state.reparented.load(Ordering::Relaxed)
	}

	pub fn unparent(&self) {
		let mut guard = self.state.reparent_state.lock().unwrap();
		*guard = ReparentState::Idle;
		self.state.reparented.store(false, Ordering::Relaxed);
		let _ = self
			.state
			.spatial
			.set_parent_in_place(self.state.initial_parent.clone());
	}
}
