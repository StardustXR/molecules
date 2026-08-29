use std::thread::current;

use gluon::{Interface, Node, RefExt};
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::{FieldRef, FieldSample},
	query::{InterfaceDependency, QueriedInterface, QueryableId},
	spatial::{Spatial, SpatialRef},
	spatial_query::{
		Point, PointsQuery, PointsQueryHandle, PointsQueryHandler, PointsQueryHandlerHandler,
	},
};
use stardust_xr_molecules_protocols::container::{self, ContainerHandler};
use tokio::sync::Mutex;

#[derive(gluon::Handler)]
pub struct Container;
impl ContainerHandler for Container {}

pub struct Containable(Node<ContainableInner>, PointsQueryHandle);
impl Containable {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		containable_spatial: Spatial,
		original_parent: SpatialRef,
		reference_space: SpatialRef,
		evaluator: impl Fn(&Containers) -> Option<SpatialRef> + Send + Sync + 'static,
	) -> Result<Self> {
		let (containable, containable_ref) = PointsQueryHandler::new_node(ContainableInner {
			original_parent,
			spatial: containable_spatial,
			evaluator: Box::new(evaluator),
			current_container: Mutex::new(None),
			containers: Mutex::new(FxHashMap::default()),
		})?;
		let query = client
			.spatial_query_interface()
			.points_query(PointsQuery {
				handler: containable_ref.into_proxy(),
				interfaces: vec![InterfaceDependency {
					id: container::Container::ID.to_string(),
					optional: false,
				}],
				reference_spatial: reference_space,
				points: vec![Point {
					point: [0.0; 3].into(),
					margin: 0.0,
				}],
			})
			.await??;

		Ok(Containable(containable, query))
	}
}

type Containers = FxHashMap<QueryableId, (FieldSample, SpatialRef)>;

#[derive(gluon::Handler)]
struct ContainableInner {
	original_parent: SpatialRef,
	spatial: Spatial,
	evaluator: Box<dyn Fn(&Containers) -> Option<SpatialRef> + Send + Sync>,
	current_container: Mutex<Option<SpatialRef>>,
	containers: Mutex<FxHashMap<QueryableId, (FieldSample, SpatialRef)>>,
}
impl ContainableInner {
	async fn attempt_reparent(&self) {
		let containers = self.containers.lock().await;
		let mut current_container = self.current_container.lock().await;
		let target_spatial = (self.evaluator)(&containers);
		if target_spatial.as_ref() != current_container.as_ref() {
			let new_spatial = target_spatial
				.clone()
				.unwrap_or_else(|| self.original_parent.clone());
			let _ = self.spatial.set_parent_in_place(new_spatial);
			*current_container = target_spatial;
		}
	}
}
impl PointsQueryHandlerHandler for ContainableInner {
	async fn entered(
		&self,
		_ctx: gluon::Context,
		id: QueryableId,
		_field: FieldRef,
		spatial: SpatialRef,
		_interfaces: Vec<QueriedInterface>,
		spatial_info: FieldSample,
	) {
		self.containers
			.lock()
			.await
			.insert(id, (spatial_info, spatial));
		self.attempt_reparent().await;
	}

	async fn interfaces_changed(
		&self,
		_ctx: gluon::Context,
		_obj: QueryableId,
		_interfaces: Vec<QueriedInterface>,
	) {
	}

	async fn moved(&self, _ctx: gluon::Context, id: QueryableId, spatial_info: FieldSample) {
		let mut containers_guard = self.containers.lock().await;
		let Some(container) = containers_guard.get_mut(&id) else {
			return;
		};
		container.0 = spatial_info;
		drop(containers_guard);
		self.attempt_reparent().await;
	}

	async fn left(&self, _ctx: gluon::Context, id: QueryableId) {
		self.containers.lock().await.remove(&id);
		self.attempt_reparent().await;
	}
}
