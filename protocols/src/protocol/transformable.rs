#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
	protocol_name: "org.stardustxr.Transformable",
	types: &[],
};
pub mod proxies {
	use super::*;
}
#[derive(Debug, Clone)]
pub struct Transformable {
	obj: gluon::Ref,
}
impl gluon::Convertable for Transformable {
	fn write(&self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write(gluon_data)
	}
	fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
		let obj = gluon::Ref::read(gluon_data)?;
		Ok(Transformable::from_ref(obj))
	}
	fn write_owned(self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write_owned(gluon_data)
	}
}
impl gluon::Interface for Transformable {
	const ID: &'static str = "org.stardustxr.Transformable.Transformable";
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: TransformableHandler> gluon::HandledBy<H> for Transformable {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type TransformableLocal<H> = gluon::LocalRef<Transformable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: TransformableHandler> From<TransformableLocal<H>> for Transformable {
	fn from(value: TransformableLocal<H>) -> Transformable {
		value.into_proxy()
	}
}
impl gluon::RefExt for Transformable {
	fn from_ref(obj: gluon::Ref) -> Transformable {
		Transformable { obj }
	}
}
impl Transformable {
	///Transform this object by this partial transform relative to the provided spatialref (adds to the existing transform)
	pub fn offset_relative_transform(
		&self,
		reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
		offset_transform: impl Into<stardust_xr_protocol::spatial::PartialTransform>,
	) -> Result<(), gluon::SendError> {
		let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
		let offset_transform: stardust_xr_protocol::spatial::PartialTransform =
			offset_transform.into();
		tracing::trace!(
			interface = "Transformable",
			method = "offset_relative_transform",
			?reference,
			?offset_transform,
			"→"
		);
		let mut gluon_builder = gluon::DataBuilder::new();
		reference.write(&mut gluon_builder)?;
		offset_transform.write(&mut gluon_builder)?;
		gluon::transact(&self.obj, 8u32, gluon_builder)?;
		Ok(())
	}
	///Transform this object by this partial transform relative to the provided spatialref
	pub fn set_relative_transform(
		&self,
		reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
		transform: impl Into<stardust_xr_protocol::spatial::PartialTransform>,
	) -> Result<(), gluon::SendError> {
		let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
		let transform: stardust_xr_protocol::spatial::PartialTransform = transform.into();
		tracing::trace!(
			interface = "Transformable",
			method = "set_relative_transform",
			?reference,
			?transform,
			"→"
		);
		let mut gluon_builder = gluon::DataBuilder::new();
		reference.write(&mut gluon_builder)?;
		transform.write(&mut gluon_builder)?;
		gluon::transact(&self.obj, 9u32, gluon_builder)?;
		Ok(())
	}
	///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
	pub fn from_ref(obj: gluon::Ref) -> Transformable {
		Transformable { obj }
	}
}
impl From<Transformable> for gluon::Ref {
	fn from(value: Transformable) -> Self {
		value.obj
	}
}
impl gluon::ToRef for Transformable {
	fn to_ref(&self) -> gluon::Ref {
		self.obj.clone()
	}
}
impl gluon::Liveness for Transformable {
	fn death_notifier(&self) -> gluon::DeathNotifier {
		gluon::Liveness::death_notifier(&self.obj)
	}
}
impl std::hash::Hash for Transformable {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.obj.hash(state);
	}
}
impl PartialEq for Transformable {
	fn eq(&self, other: &Self) -> bool {
		self.obj == other.obj
	}
}
impl Eq for Transformable {}
pub trait TransformableHandler: gluon::Handler + Send + Sync + 'static {
	///Transform this object by this partial transform relative to the provided spatialref (adds to the existing transform)
	fn offset_relative_transform(
		&self,
		_ctx: gluon::Context,
		reference: stardust_xr_protocol::spatial::SpatialRef,
		offset_transform: stardust_xr_protocol::spatial::PartialTransform,
	) -> impl Future<Output = ()> + Send + Sync;
	///Transform this object by this partial transform relative to the provided spatialref
	fn set_relative_transform(
		&self,
		_ctx: gluon::Context,
		reference: stardust_xr_protocol::spatial::SpatialRef,
		transform: stardust_xr_protocol::spatial::PartialTransform,
	) -> impl Future<Output = ()> + Send + Sync;
	fn dispatch_one_way(
		&self,
		transaction_code: u32,
		mut gluon_data: gluon::DataReader,
		ctx: gluon::Context,
	) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
		async move {
			match transaction_code {
				8u32 => {
					let param_reference = gluon::Convertable::read(&mut gluon_data)?;
					let param_offset_transform = gluon::Convertable::read(&mut gluon_data)?;
					tracing::trace!(
						interface = "Transformable",
						method = "offset_relative_transform",
						?param_reference,
						?param_offset_transform,
						"dispatching"
					);
					drop(gluon_data);
					self.offset_relative_transform(ctx, param_reference, param_offset_transform)
						.instrument(tracing::trace_span!(
							"dispatching",
							interface = "Transformable",
							method = "offset_relative_transform",
							method_id = 8u32
						))
						.await;
				}
				9u32 => {
					let param_reference = gluon::Convertable::read(&mut gluon_data)?;
					let param_transform = gluon::Convertable::read(&mut gluon_data)?;
					tracing::trace!(
						interface = "Transformable",
						method = "set_relative_transform",
						?param_reference,
						?param_transform,
						"dispatching"
					);
					drop(gluon_data);
					self.set_relative_transform(ctx, param_reference, param_transform)
						.instrument(tracing::trace_span!(
							"dispatching",
							interface = "Transformable",
							method = "set_relative_transform",
							method_id = 9u32
						))
						.await;
				}
				_ => {}
			}
			Ok(())
		}
	}
}
pub mod proxied {
	use super::*;
}
