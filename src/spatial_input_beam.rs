//! What a non-spatial device is aimed at.
//!
//! A keyboard or a mouse isn't an input method, it has nothing to say about where it is in
//! space, it just needs to know which handler is under the beam right now. Point one of
//! these down a spatial and ask it whenever an event comes in.

use gluon_ipc::{Context, Handler, Node, Ref, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::{FieldRef, RayMarchResult},
	query::{InterfaceDependency, QueriedInterface, QueryableId},
	spatial::SpatialRef,
	spatial_query::{BeamQuery, BeamQueryHandle, BeamQueryHandler, BeamQueryHandlerHandler},
	types::Vec3F,
};
use std::{collections::HashMap, fmt::Debug, sync::OnceLock};
use tokio::sync::RwLock;

#[derive(Debug, Handler)]
pub struct SpatialInputBeam<T: Debug + Clone + Send + Sync + 'static> {
	hits: RwLock<HashMap<QueryableId, (T, f32)>>,
	construct: fn(&str, Ref) -> Option<T>,
	handle: OnceLock<BeamQueryHandle>,
}
impl<T: Debug + Clone + Send + Sync + 'static> SpatialInputBeam<T> {
	/// fires down -Z of `reference_spatial`, so aim it by moving that
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		reference_spatial: SpatialRef,
		construct: fn(&str, Ref) -> Option<T>,
		interface: String,
		max_length: f32,
		margin: f32,
	) -> Result<Node<Self>> {
		let (node, handler) = BeamQueryHandler::new_node(Self {
			hits: RwLock::default(),
			construct,
			handle: OnceLock::new(),
		})?;
		let handle = client
			.spatial_query_interface()
			.beam_query(BeamQuery {
				handler: handler.into_proxy(),
				interfaces: vec![InterfaceDependency {
					id: interface,
					optional: false,
				}],
				reference_spatial,
				origin: [0.0; 3].into(),
				direction: [0.0, 0.0, -1.0].into(),
				max_length,
				margin,
			})
			.await??;
		let _ = node.handle.set(handle);
		Ok(node)
	}

	/// whatever the beam hits first, None when it's pointing at nothing
	pub async fn get_handler(&self) -> Option<T> {
		self.hits
			.read()
			.await
			.values()
			.min_by(|(_, a), (_, b)| a.total_cmp(b))
			.map(|(handler, _)| handler.clone())
	}

	/// retarget the beam within its reference spatial
	pub fn update(
		&self,
		origin: impl Into<Vec3F>,
		direction: impl Into<Vec3F>,
		max_length: f32,
		margin: f32,
	) {
		let Some(handle) = self.handle.get() else {
			return;
		};
		let _ = handle.update(origin.into(), direction.into(), max_length, margin);
	}
}
impl<T: Debug + Clone + Send + Sync + 'static> BeamQueryHandlerHandler for SpatialInputBeam<T> {
	async fn intersected(
		&self,
		_ctx: Context,
		obj: QueryableId,
		_field: FieldRef,
		_spatial: SpatialRef,
		interfaces: Vec<QueriedInterface>,
		sample: RayMarchResult,
	) {
		let Some(handler) = interfaces
			.into_iter()
			.find_map(|i| (self.construct)(&i.interface_id, i.interface))
		else {
			return;
		};
		self.hits
			.write()
			.await
			.insert(obj, (handler, sample.deepest_point_distance));
	}

	async fn interfaces_changed(
		&self,
		_ctx: Context,
		_obj: QueryableId,
		_interfaces: Vec<QueriedInterface>,
	) {
	}

	// the nearest hit is worked out on read instead of tracked here, otherwise the one that
	// moves away from the beam stays "nearest" until something else happens to beat it
	async fn moved(&self, _ctx: Context, obj: QueryableId, sample: RayMarchResult) {
		if let Some((_, distance)) = self.hits.write().await.get_mut(&obj) {
			*distance = sample.deepest_point_distance;
		}
	}

	async fn left(&self, _ctx: Context, obj: QueryableId) {
		self.hits.write().await.remove(&obj);
	}
}
