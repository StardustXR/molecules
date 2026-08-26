use gluon::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::Result;
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	fields::Field,
	query::{QueryableExt, QueryableObject},
	spatial::{Spatial, SpatialRef, Transform},
};
pub use stardust_xr_molecules_protocols::reparentable::{
	ReparentHandle, ReparentKeepalive, ReparentKeepaliveHandler, Reparentable as ReparentableProxy,
	ReparentableLocked as ReparentableLockedProxy,
};
use stardust_xr_molecules_protocols::reparentable::{
	ReparentHandleHandler, ReparentHandleLocal, ReparentableHandler, ReparentableLockedHandler,
};
use std::{
	any::Any,
	sync::{
		Arc, Mutex,
		atomic::{AtomicU64, Ordering},
	},
};

/// A live reparent. `locked` grants are not stealable.
struct Grant {
	id: u64,
	locked: bool,
	keepalive: ReparentKeepalive,
}

struct SharedState {
	spatial: Spatial,
	initial_parent: SpatialRef,
	grant: Mutex<Option<Grant>>,
}
impl SharedState {
	/// Grants a reparent to `new_parent`, stealing an existing non-locked grant.
	/// Returns `None` if the current grant is locked.
	fn begin_grant(
		&self,
		new_parent: SpatialRef,
		keepalive: ReparentKeepalive,
		locked: bool,
	) -> Option<u64> {
		let mut guard = self.grant.lock().unwrap();
		if let Some(current) = &*guard {
			if current.locked {
				return None;
			}
			let _ = current.keepalive.reparent_stolen();
		}
		let id = HANDLE_ID.fetch_add(1, Ordering::Relaxed);
		let _ = self.spatial.set_parent_in_place(new_parent);
		*guard = Some(Grant {
			id,
			locked,
			keepalive,
		});
		Some(id)
	}

	/// Ends the current grant and returns the spatial to its initial parent.
	/// `expect_id` limits this to that specific grant; `notify` tells the holder it lost it.
	fn end_grant(&self, expect_id: Option<u64>, notify: bool) {
		let mut guard = self.grant.lock().unwrap();
		let Some(current) = &*guard else {
			return;
		};
		if let Some(id) = expect_id
			&& current.id != id
		{
			return;
		}
		if notify {
			let _ = current.keepalive.reparent_stolen();
		}
		let _ = self
			.spatial
			.set_parent_in_place(self.initial_parent.clone());
		*guard = None;
	}

	fn reparented(&self) -> bool {
		self.grant.lock().unwrap().is_some()
	}
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
			.set_relative_transform(relative_to, Transform::IDENTITY);
	}
}
impl Drop for ReparentHandleInner {
	fn drop(&mut self) {
		self.state.end_grant(Some(self.id), false);
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
	) -> Option<ReparentHandleLocal<ReparentHandleInner>> {
		let id = self.state.begin_grant(new_parent, keepalive, locking)?;
		let handle = ReparentHandle::new_service(ReparentHandleInner {
			state: self.state.clone(),
			id,
		});
		match handle {
			Ok(handle) => Some(handle),
			Err(_) => {
				self.state.end_grant(Some(id), false);
				None
			}
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
		Some(
			self.do_reparent(new_parent, keepalive, false)
				.await?
				.into_proxy(),
		)
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
		Some(
			self.0
				.do_reparent(new_parent, keepalive, true)
				.await?
				.into_proxy(),
		)
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
			grant: Mutex::new(None),
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
		self.state.reparented()
	}

	pub fn unparent(&self) {
		self.state.end_grant(None, true);
	}
}

#[tokio::test]
async fn reparentable_query() {
	use gluon::Handler;
	use stardust_xr_fusion::{
		client::Client,
		fields::{Field, FieldExt, FieldRef, FieldSample, Shape},
		project_local_resources,
		query::{InterfaceDependency, QueriedInterface, QueryableId},
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
			id: QueryableId,
			_field: FieldRef,
			_spatial: SpatialRef,
			interfaces: Vec<QueriedInterface>,
			_spatial_info: FieldSample,
		) {
			tracing::info!(?id, ?interfaces, "ENTERED");
		}
		async fn interfaces_changed(
			&self,
			_ctx: gluon::Context,
			id: QueryableId,
			interfaces: Vec<QueriedInterface>,
		) {
			tracing::info!(?id, ?interfaces, "INTERFACES CHANGED");
		}
		async fn moved(&self, _ctx: gluon::Context, _id: QueryableId, _spatial_info: FieldSample) {}
		async fn left(&self, _ctx: gluon::Context, id: QueryableId) {
			tracing::info!(?id, "LEFT");
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
			handler: handler.into_proxy(),
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
