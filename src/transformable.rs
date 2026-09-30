use futures::FutureExt;
use tokio::sync::{Mutex, Notify};

use std::sync::Arc;

use gluon_ipc::Handler;
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	query::{QueryableExt, QueryableInterface, QueryableObject},
	spatial::{PartialTransform, Spatial, SpatialInterface, SpatialRef, Transform},
	types::{Posef, QuatF, Vec3F},
};
use stardust_xr_molecules_protocols::transformable::{
	PoseableHandler, RotatableHandler, ScalableHandler, TransformableHandler, TranslatableHandler,
};
use tracing::error;

use crate::drop_handlers::AbortOnDrop;

pub mod protocol {
	pub use stardust_xr_molecules_protocols::transformable::*;
}

pub struct TransformableInterfaces {
	_transformable_node: Option<QueryableInterface>,
	_translatable_node: Option<QueryableInterface>,
	_rotatable_node: Option<QueryableInterface>,
	_scalable_node: Option<QueryableInterface>,
	_poseable_node: Option<QueryableInterface>,
	transform: Arc<Mutex<Option<Transform>>>,
	transform_updated: Arc<Notify>,
}
pub struct TransformableSpatialInterfaces {
	_task: AbortOnDrop,
}
impl TransformableSpatialInterfaces {
	pub async fn new(
		client: &Client<impl ClientHandler>,
		obj: &QueryableObject,
		spatial: &Spatial,
		translation: bool,
		rotation: bool,
		scale: bool,
	) -> Self {
		// TODO: maybe figure out something better than the client root here?
		Self::new_with_ref_space(
			client.spatial_interface(),
			obj,
			spatial,
			client.root(),
			translation,
			rotation,
			scale,
		)
		.await
	}
	pub async fn new_with_ref_space(
		spatial_interface: &SpatialInterface,
		obj: &QueryableObject,
		spatial: &Spatial,
		reference_space: &SpatialRef,
		translation: bool,
		rotation: bool,
		scale: bool,
	) -> Self {
		// TODO: remove this unwrap
		let spatial_ref = spatial.spatial_ref().await.unwrap();
		let mut interfaces = TransformableInterfaces::new_with_interface(
			spatial_interface,
			obj,
			&spatial_ref,
			reference_space,
			translation,
			rotation,
			scale,
		)
		.await;
		let ref_space = reference_space.clone();
		let spatial = spatial.clone();
		let _task = tokio::spawn(async move {
			loop {
				let t = interfaces.recv().await;
				_ = spatial.set_relative_transform(ref_space.clone(), t);
			}
		})
		.into();
		Self { _task }
	}
}

impl TransformableInterfaces {
	pub async fn new(
		client: &Client<impl ClientHandler>,
		obj: &QueryableObject,
		spatial: &SpatialRef,
		reference_space: &SpatialRef,
		translation: bool,
		rotation: bool,
		scale: bool,
	) -> Self {
		Self::new_with_interface(
			client.spatial_interface(),
			obj,
			spatial,
			reference_space,
			translation,
			rotation,
			scale,
		)
		.await
	}
	pub fn try_recv(&mut self) -> Option<Transform> {
		let lock = self.transform.try_lock().ok();
		// consume any remainding permits on the notify
		tokio::task::unconstrained(self.transform_updated.notified()).now_or_never();
		lock.map(|mut v| v.take()).flatten()
	}
	pub async fn recv(&mut self) -> Transform {
		self.transform_updated.notified().await;
		// this unwrap should be fine since we only trigger the notify after setting the transform,
		// and we never unset it except in try_recv and this recv, which both consume permits
		self.transform.lock().await.unwrap()
	}
	pub async fn new_with_interface(
		spatial_interface: &SpatialInterface,
		obj: &QueryableObject,
		spatial: &SpatialRef,
		reference_space: &SpatialRef,
		translation: bool,
		rotation: bool,
		scale: bool,
	) -> Self {
		let core = TransformableCore {
			transform: Arc::new(Mutex::new(None)),
			target_spatial: spatial.clone(),
			reference_space: reference_space.clone(),
			si: spatial_interface.clone(),
			transform_updated: Arc::default(),
		};
		// let _transformable_node = todo!();
		let _transformable_node = if translation
			&& rotation
			&& scale && let Ok(v) =
			TransformableInner(core.clone()).to_service()
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
			TranslatableInner(core.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Translatable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		let _rotatable_node = if rotation
			&& let Ok(v) = RotatableInner(core.clone()).to_service()
			&& let Ok(v) = QueryableExt::add_interface(obj, v.proxy())
				.await
				.inspect_err(|err| error!("failed to create Rotatable node: {err}"))
		{
			Some(v)
		} else {
			None
		};
		let _scalable_node = if scale
			&& let Ok(v) = ScalableInner(core.clone()).to_service()
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
			&& let Ok(v) = PoseableInner(core.clone()).to_service()
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
			transform: core.transform.clone(),
			transform_updated: core.transform_updated.clone(),
		}
	}
}

#[derive(Debug, Clone)]
struct TransformableCore {
	transform: Arc<Mutex<Option<Transform>>>,
	target_spatial: SpatialRef,
	reference_space: SpatialRef,
	si: SpatialInterface,
	transform_updated: Arc<Notify>,
}
impl TransformableCore {
	async fn offset(&self, reference: SpatialRef, offset: PartialTransform) {
		let mut lock = self.transform.lock().await;
		let Ok(Ok(v)) = self
			.si
			.get_relative_transform(self.reference_space.clone(), reference)
			.await
		else {
			return;
		};
		let transform = if let Some(lock) = lock.as_ref() {
			*lock
		} else {
			let Ok(Ok(v)) = self
				.si
				.get_relative_transform(self.reference_space.clone(), self.target_spatial.clone())
				.await
			else {
				return;
			};
			v
		};
		let transform = transform * (v * offset);
		lock.replace(transform);
		self.transform_updated.notify_one();
	}
	async fn set(&self, reference: SpatialRef, transform: PartialTransform) {
		let mut lock = self.transform.lock().await;
		let Ok(Ok(v)) = self
			.si
			.get_relative_transform(self.reference_space.clone(), reference)
			.await
		else {
			return;
		};
		let transform = v * transform;
		lock.replace(transform);
		self.transform_updated.notify_one();
	}
}

macro_rules! transformable {
	($name:ident, $trait:ident, $type:ty, $offset_name:ident,$set_name:ident, $convert:expr) => {
		#[derive(Handler)]
		struct $name(TransformableCore);
		impl $trait for $name {
			fn $offset_name(
				&self,
				_ctx: gluon_ipc::Context,
				reference: SpatialRef,
				offset_transform: $type,
			) -> impl Future<Output = ()> {
				self.0.offset(reference, ($convert)(offset_transform))
			}

			fn $set_name(
				&self,
				_ctx: gluon_ipc::Context,
				reference: SpatialRef,
				transform: $type,
			) -> impl Future<Output = ()> {
				self.0.set(reference, $convert(transform))
			}
		}
	};
}
transformable!(
	TransformableInner,
	TransformableHandler,
	PartialTransform,
	offset_relative_transform,
	set_relative_transform,
	|v| v
);
transformable!(
	TranslatableInner,
	TranslatableHandler,
	Vec3F,
	offset_relative_translation,
	set_relative_translation,
	PartialTransform::from_translation
);
transformable!(
	RotatableInner,
	RotatableHandler,
	QuatF,
	offset_relative_rotation,
	set_relative_rotation,
	PartialTransform::from_rotation
);
transformable!(
	ScalableInner,
	ScalableHandler,
	Vec3F,
	offset_relative_scale,
	set_relative_scale,
	PartialTransform::from_scale
);
transformable!(
	PoseableInner,
	PoseableHandler,
	Posef,
	offset_relative_pose,
	set_relative_pose,
	|pose: Posef| PartialTransform::from_translation_rotation(pose.position, pose.orientation)
);
