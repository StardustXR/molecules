use glam::Vec3;

use std::future::ready;

use gluon::Handler;
use stardust_xr_fusion::{
	query::{QueryableExt, QueryableInterface, QueryableObject},
	spatial::{PartialTransform, Spatial, SpatialRef},
	types::{Posef, QuatF, Vec3F},
};
use stardust_xr_molecules_protocols::transformable::{
	PoseableHandler, RotatableHandler, ScalableHandler, TransformableHandler, TranslatableHandler,
};
use tracing::error;

pub mod protocol {
	pub use stardust_xr_molecules_protocols::transformable::*;
}

pub struct TransformableInterfaces {
	_transformable_node: Option<QueryableInterface>,
	_translatable_node: Option<QueryableInterface>,
	_rotatable_node: Option<QueryableInterface>,
	_scalable_node: Option<QueryableInterface>,
	_poseable_node: Option<QueryableInterface>,
}

impl TransformableInterfaces {
	pub async fn new(
		obj: &QueryableObject,
		spatial: &Spatial,
		translation: bool,
		rotation: bool,
		scale: bool,
	) -> Self {
		let _transformable_node = if translation
			&& rotation
			&& scale && let Ok(v) =
			TransformableInner(spatial.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Transformable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		let _translatable_node = if translation
			&& rotation
			&& scale && let Ok(v) =
			TranslatableInner(spatial.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Translatable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		let _rotatable_node = if rotation
			&& let Ok(v) = RotatableInner(spatial.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Rotatable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		let _scalable_node = if scale
			&& let Ok(v) = ScalableInner(spatial.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Scalable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		let _poseable_node = if translation
			&& rotation
			&& let Ok(v) = PoseableInner(spatial.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Poseable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		Self {
			_transformable_node,
			_translatable_node,
			_rotatable_node,
			_scalable_node,
			_poseable_node,
		}
	}
}

#[derive(Handler)]
struct TransformableInner(Spatial);
impl TransformableHandler for TransformableInner {
	async fn offset_relative_transform(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		offset_transform: PartialTransform,
	) {
		let Ok(Ok(transform)) = self.0.get_relative_transform(reference.clone()).await else {
			return;
		};
		let transform = transform * offset_transform;
		_ = self.0.set_relative_transform(reference, transform);
	}

	fn set_relative_transform(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		transform: PartialTransform,
	) -> impl Future<Output = ()> + Send + Sync {
		_ = self.0.set_relative_transform(reference, transform);
		ready(())
	}
}

#[derive(Handler)]
struct TranslatableInner(Spatial);
impl TranslatableHandler for TranslatableInner {
	async fn offset_relative_translation(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		offset: Vec3F,
	) {
		let Ok(Ok(mut transform)) = self.0.get_relative_transform(reference.clone()).await else {
			return;
		};
		transform.translation = (Vec3::from(transform.translation) + Vec3::from(offset)).into();
		_ = self.0.set_relative_transform(reference, transform);
	}

	fn set_relative_translation(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		translation: Vec3F,
	) -> impl Future<Output = ()> + Send + Sync {
		_ = self
			.0
			.set_relative_transform(reference, PartialTransform::from_translation(translation));
		ready(())
	}
}
#[derive(Handler)]
struct RotatableInner(Spatial);
impl RotatableHandler for RotatableInner {
	async fn offset_relative_rotation(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		offset: QuatF,
	) {
		let Ok(Ok(transform)) = self.0.get_relative_transform(reference.clone()).await else {
			return;
		};
		let transform = transform * PartialTransform::from_rotation(offset);
		_ = self.0.set_relative_transform(reference, transform);
	}

	fn set_relative_rotation(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		rotation: QuatF,
	) -> impl Future<Output = ()> + Send + Sync {
		_ = self
			.0
			.set_relative_transform(reference, PartialTransform::from_rotation(rotation));
		ready(())
	}
}
#[derive(Handler)]
struct ScalableInner(Spatial);
impl ScalableHandler for ScalableInner {
	async fn offset_relative_scale(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		offset: Vec3F,
	) {
		let Ok(Ok(transform)) = self.0.get_relative_transform(reference.clone()).await else {
			return;
		};
		let transform = transform * PartialTransform::from_scale(offset);
		_ = self.0.set_relative_transform(reference, transform);
	}

	fn set_relative_scale(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		scale: Vec3F,
	) -> impl Future<Output = ()> + Send + Sync {
		_ = self
			.0
			.set_relative_transform(reference, PartialTransform::from_scale(scale));
		ready(())
	}
}
#[derive(Handler)]
struct PoseableInner(Spatial);
impl PoseableHandler for PoseableInner {
	async fn offset_relative_pse(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		offset: Posef,
	) {
		let Ok(Ok(transform)) = self.0.get_relative_transform(reference.clone()).await else {
			return;
		};
		let transform = transform
			* PartialTransform::from_translation_rotation(offset.position, offset.orientation);
		_ = self.0.set_relative_transform(reference, transform);
	}

	fn set_relative_pose(
		&self,
		_ctx: gluon::Context,
		reference: SpatialRef,
		pose: Posef,
	) -> impl Future<Output = ()> + Send + Sync {
		_ = self.0.set_relative_transform(
			reference,
			PartialTransform::from_translation_rotation(pose.position, pose.orientation),
		);
		ready(())
	}
}
