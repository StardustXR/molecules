use gluon::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::Result;
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	fields::Field,
	query::{QueryableExt, QueryableObject},
	spatial::{PartialTransform, Spatial, SpatialRef},
};
pub use stardust_xr_molecules_protocols::reparentable::{
	ReparentHandle, ReparentKeepalive, ReparentKeepaliveHandler, Reparentable as ReparentableProxy,
	ReparentableLocked as ReparentableLockedProxy,
};
use stardust_xr_molecules_protocols::reparentable::{
	ReparentHandleHandler, ReparentableHandler, ReparentableLockedHandler,
};
use std::{
	any::Any,
	sync::{
		Arc, Mutex,
		atomic::{AtomicBool, AtomicU64, Ordering},
	},
};

enum ReparentState {
	Idle,
	NonLocked(u64, ReparentKeepalive),
	// we want to keep this handle alive, even if we don't use it directly
	Locked(u64, #[allow(dead_code)] ReparentKeepalive),
}

struct SharedState {
	spatial: Spatial,
	initial_parent: SpatialRef,
	reparent_state: Mutex<ReparentState>,
	reparented: AtomicBool,
}

#[derive(gluon::Handler)]
struct ReparentHandleInner {
	state: Arc<SharedState>,
	id: u64,
}
impl std::fmt::Debug for ReparentHandleInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReparentHandleInner").finish()
	}
}
impl ReparentHandleHandler for ReparentHandleInner {
	async fn reset_transform(&self, _ctx: Context, relative_to: SpatialRef) {
		let _ = self
			.state
			.spatial
			.set_relative_transform(relative_to, PartialTransform::NONE);
	}
}
impl Drop for ReparentHandleInner {
	fn drop(&mut self) {
		let mut guard = self.state.reparent_state.lock().unwrap();
		let still_current = match &*guard {
			ReparentState::NonLocked(current, _) | ReparentState::Locked(current, _) => {
				*current == self.id
			}
			ReparentState::Idle => false,
		};
		if still_current {
			*guard = ReparentState::Idle;
			self.state.reparented.store(false, Ordering::Relaxed);
			let _ = self
				.state
				.spatial
				.set_parent_in_place(self.state.initial_parent.clone());
		}
	}
}

#[derive(gluon::Handler)]
struct ReparentableInner {
	state: Arc<SharedState>,
}
impl std::fmt::Debug for ReparentableInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReparentableInner").finish()
	}
}
static HANDLE_ID: AtomicU64 = AtomicU64::new(0);

impl ReparentableInner {
	/// Shared reparent logic used by both the locking and non-locking interfaces.
	async fn do_reparent(
		&self,
		new_parent: SpatialRef,
		keepalive: ReparentKeepalive,
		locking: bool,
	) -> Option<ReparentHandle> {
		let mut guard = self.state.reparent_state.lock().unwrap();
		match &*guard {
			ReparentState::Idle | ReparentState::NonLocked(_, _) => {
				if let ReparentState::NonLocked(_, old) = &*guard {
					let _ = old.reparent_stolen();
				}
				let _ = self.state.spatial.set_parent_in_place(new_parent);
				self.state.reparented.store(true, Ordering::Relaxed);
				let id = HANDLE_ID.fetch_add(1, Ordering::Relaxed);
				let handle = ReparentHandle::new_service(ReparentHandleInner {
					state: self.state.clone(),
					id,
				})
				.ok()?;
				*guard = if locking {
					ReparentState::Locked(id, keepalive.clone())
				} else {
					ReparentState::NonLocked(id, keepalive.clone())
				};
				drop(guard);
				Some(handle)
			}
			ReparentState::Locked(_, _) => None,
		}
	}
}
impl ReparentableHandler for ReparentableInner {
	async fn reparent(
		&self,
		_ctx: Context,
		new_parent: SpatialRef,
		keepalive: ReparentKeepalive,
	) -> Option<ReparentHandle> {
		self.do_reparent(new_parent, keepalive, false).await
	}
}

/// Locking counterpart to [`ReparentableInner`], registered as its own gluon object so the
/// two capabilities can be permission-gated independently. Delegates the actual work to the
/// [`ReparentableInner`] it wraps.
#[derive(gluon::Handler)]
struct ReparentableLockedInner(Arc<ReparentableInner>);
impl std::fmt::Debug for ReparentableLockedInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReparentableLockedInner").finish()
	}
}
impl ReparentableLockedHandler for ReparentableLockedInner {
	async fn reparent_locking(
		&self,
		_ctx: Context,
		new_parent: SpatialRef,
		keepalive: ReparentKeepalive,
	) -> Option<ReparentHandle> {
		self.0.do_reparent(new_parent, keepalive, true).await
	}
}

pub struct Reparentable {
	state: Arc<SharedState>,
	_node: Node<ReparentableInner>,
	_locked_node: Node<ReparentableLockedInner>,
	_queryable: QueryableObject,
	_guards: Vec<Box<dyn Any + Send + Sync>>,
}
impl Reparentable {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		initial_parent: SpatialRef,
		field: Field,
	) -> Result<Self> {
		let _ = spatial.set_parent_in_place(initial_parent.clone());
		let state = Arc::new(SharedState {
			spatial: spatial.clone(),
			initial_parent: initial_parent.clone(),
			reparent_state: Mutex::new(ReparentState::Idle),
			reparented: AtomicBool::new(false),
		});

		let reparentable_inner = ReparentableInner {
			state: state.clone(),
		};
		let (reparentable_node, reparentable) = ReparentableProxy::new_node(reparentable_inner)?;

		let (reparentable_locked_node, reparentable_locked) = ReparentableLockedProxy::new_node(
			ReparentableLockedInner(reparentable_node.handler().clone()),
		)?;

		let queryable = QueryableObject::new(client, spatial, field).await?;
		let guard = queryable
			.add_interface(&reparentable, ReparentableProxy::ID)
			.await?;
		let locked_guard = queryable
			.add_interface(&reparentable_locked, ReparentableLockedProxy::ID)
			.await?;

		Ok(Reparentable {
			state,
			_node: reparentable_node,
			_locked_node: reparentable_locked_node,
			_queryable: queryable,
			_guards: vec![
				Box::new(guard) as Box<dyn Any + Send + Sync>,
				Box::new(locked_guard) as Box<dyn Any + Send + Sync>,
			],
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

#[tokio::test]
async fn reparentable_query() {
	use gluon::Handler;
	use stardust_xr_fusion::{
		client::Client,
		fields::{Field, FieldExt, FieldRef, FieldSample, Shape},
		project_local_resources,
		query::{InterfaceDependency, QueriedInterface, QueryableObjectRef},
		spatial::{Spatial, SpatialExt, SpatialRef, Transform},
		spatial_query::{Point, PointsQuery, PointsQueryHandler, PointsQueryHandlerHandler},
	};
	use tokio::sync::broadcast::error::RecvError;

	#[derive(Debug, Handler)]
	struct Logger;
	impl PointsQueryHandlerHandler for Logger {
		async fn entered(
			&self,
			_ctx: gluon::Context,
			obj: QueryableObjectRef,
			_field: FieldRef,
			_spatial: SpatialRef,
			interfaces: Vec<QueriedInterface>,
			_spatial_info: FieldSample,
		) {
			tracing::info!(?obj, ?interfaces, "ENTERED");
		}
		async fn interfaces_changed(
			&self,
			_ctx: gluon::Context,
			obj: QueryableObjectRef,
			interfaces: Vec<QueriedInterface>,
		) {
			tracing::info!(?obj, ?interfaces, "INTERFACES CHANGED");
		}
		async fn moved(
			&self,
			_ctx: gluon::Context,
			_obj: QueryableObjectRef,
			_spatial_info: FieldSample,
		) {
		}
		async fn left(&self, _ctx: gluon::Context, obj: QueryableObjectRef) {
			tracing::info!(?obj, "LEFT");
		}
	}

	tracing_subscriber::fmt().pretty().with_file(false).init();

	let (client, root) = Client::connect(&[&project_local_resources!("res")])
		.await
		.expect("Unable to connect to server");

	// provider: a dummy object that registers itself as Reparentable, sitting at root
	let (dummy_spatial, dummy_ref) = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.expect("failed to create dummy spatial");
	let (dummy_field, _) = Field::new(&client, &dummy_spatial, Shape::Sphere { radius: 0.1 })
		.await
		.expect("failed to create dummy field");
	let _reparentable = Reparentable::new(&client, dummy_spatial, dummy_ref, dummy_field)
		.await
		.expect("failed to register dummy reparentable");
	tracing::info!("dummy reparentable object registered at root");

	// consumer: a points query with a single point at root, looking for Reparentable objects
	let (handler_node, handler) =
		PointsQueryHandler::new_node(Logger).expect("failed to create node");
	let _query = client
		.spatial_query_interface()
		.points_query(PointsQuery {
			handler,
			interfaces: vec![InterfaceDependency {
				id: ReparentableProxy::ID.into(),
				optional: false,
			}],
			reference_spatial: root.clone(),
			points: vec![Point {
				point: [0.0, 0.0, 0.0].into(),
				margin: 1.0,
			}],
		})
		.await
		.expect("failed to send points_query")
		.expect("points_query rejected");
	tracing::info!("points query registered, watching for reparentable objects");

	let mut recv = client.frame_receiver();
	loop {
		match recv.recv().await {
			Ok(_) => {}
			Err(RecvError::Closed) => break,
			Err(RecvError::Lagged(_)) => continue,
		}
	}
    drop(handler_node);
}
