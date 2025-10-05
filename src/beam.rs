use std::sync::Arc;

use glam::Vec3;
use stardust_xr_fusion::{
	ClientHandle,
	fields::{FieldRef, FieldRefAspect, RayMarchResult},
	node::NodeType,
	objects::ObjectInfo,
	query::{QueryContext, Queryable},
	query_impl::ClientQueryContext,
	spatial::SpatialRef,
};
use tracing::error;

pub struct Beam {
	beam_origin: SpatialRef,
	margin: f32,
}
impl Beam {
	pub fn new(beam_origin: SpatialRef) -> Self {
		Beam {
			beam_origin,
			margin: 0.0,
		}
	}
	pub fn new_with_margin(beam_origin: SpatialRef, margin: f32) -> Self {
		Beam {
			beam_origin,
			margin,
		}
	}
}

pub struct Beamable {
	pub field_ref: FieldRef,
	pub raymarch_result: RayMarchResult,
}
impl<Ctx: BeamQueryContext> Queryable<Ctx> for Beamable {
	async fn try_new(
		connection: &zbus::Connection,
		ctx: &Arc<Ctx>,
		object: &ObjectInfo,
		contains_interface: &(impl Fn(&str) -> bool + Send + Sync),
	) -> Option<Self> {
		let field_ref = FieldRef::try_new(connection, ctx, object, contains_interface).await?;
		let beam = ctx.get_beam();
		let raymarch_result = field_ref
			.ray_march(&beam.beam_origin, Vec3::ZERO, Vec3::NEG_Z)
			.await
			.inspect_err(|err| error!("unable to raymarch in beamable: {err}"))
			.ok()?;
		(raymarch_result.min_distance < beam.margin).then(|| Beamable {
			field_ref,
			raymarch_result,
		})
	}
}

pub trait BeamQueryContext: ClientQueryContext {
	fn get_beam(self: &Arc<Self>) -> &Arc<Beam>;
}
impl QueryContext for Beam {}
impl ClientQueryContext for Beam {
	fn get_client_handle(self: &Arc<Self>) -> &Arc<ClientHandle> {
		self.beam_origin.client()
	}
}
impl BeamQueryContext for Beam {
	fn get_beam(self: &Arc<Self>) -> &Arc<Beam> {
		self
	}
}
