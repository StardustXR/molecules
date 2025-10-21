use std::sync::Arc;

use stardust_xr_fusion::{
	ClientHandle,
	fields::{Field, FieldRefAspect},
	node::NodeType,
	objects::ObjectInfo,
	query::{QueryContext, Queryable},
	query_impl::ClientQueryContext,
	spatial::SpatialRef,
};
use tracing::error;
use zbus::names::InterfaceName;

pub struct Zone {
	field: Field,
	margin: f32,
}
impl Zone {
	pub fn new(field: Field) -> Self {
		Zone { field, margin: 0.0 }
	}
	pub fn new_with_margin(field: Field, margin: f32) -> Self {
		Zone { field, margin }
	}
}

pub struct Zoneable {
	pub spatial_ref: SpatialRef,
	pub distance: f32,
}
impl<Ctx: ZoneQueryContext> Queryable<Ctx> for Zoneable {
	async fn try_new(
		connection: &zbus::Connection,
		ctx: &Arc<Ctx>,
		object: &ObjectInfo,
		contains_interface: &(impl Fn(&InterfaceName) -> bool + Send + Sync),
	) -> Option<Self> {
		let spatial_ref = SpatialRef::try_new(connection, ctx, object, contains_interface).await?;
		let zone = ctx.get_zone();
		let distance = zone
			.field
			.distance(&spatial_ref, [0.0; 3])
			.await
			.inspect_err(|err| error!("unable to get distance for Zoneable: {err}"))
			.ok()?;
		(distance <= zone.margin).then(|| Zoneable {
			spatial_ref,
			distance,
		})
	}
}

pub trait ZoneQueryContext: ClientQueryContext {
	fn get_zone(self: &Arc<Self>) -> &Arc<Zone>;
}
impl QueryContext for Zone {}
impl ClientQueryContext for Zone {
	fn get_client_handle(self: &Arc<Self>) -> &Arc<ClientHandle> {
		self.field.client()
	}
}
impl ZoneQueryContext for Zone {
	fn get_zone(self: &Arc<Self>) -> &Arc<Zone> {
		self
	}
}
