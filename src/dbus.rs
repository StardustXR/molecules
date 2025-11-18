pub use crate::drop_handlers::AbortOnDrop;
use stardust_xr_fusion::{
	core::schemas::zbus::Connection,
	fields::Field,
	objects::{FieldObject, SpatialObject},
	spatial::Spatial,
};
use std::{any::Any, marker::PhantomData};
use zbus::{object_server::Interface, zvariant::OwnedObjectPath};

#[allow(dead_code)]
pub struct DbusObjectHandles(pub(crate) Box<dyn Any + Send + Sync + 'static>);

pub struct DbusObjectHandle<I: Interface>(
	pub(crate) Connection,
	pub(crate) OwnedObjectPath,
	pub(crate) PhantomData<I>,
);
impl<I: Interface> Drop for DbusObjectHandle<I> {
	fn drop(&mut self) {
		let connection = self.0.clone();
		let object_path = self.1.clone();
		tokio::task::spawn(async move {
			let _ = connection.object_server().remove::<I, _>(object_path).await;
		});
	}
}

pub(crate) async fn create_spatial_dbus<I: Interface>(
	connection: &Connection,
	path: &OwnedObjectPath,
	handler: I,
	connection_point: Option<Spatial>,
	field: &Field,
) {
	let field = field.clone();
	let task_1 = async {
		let field_object = FieldObject::new(field).await.unwrap();
		let _ = connection
			.object_server()
			.at(path.clone(), field_object)
			.await;
	};
	let task_2 = async {
		if let Some(spatial) = connection_point {
			let spatial_object = SpatialObject::new(spatial.clone()).await.unwrap();
			let _ = connection
				.object_server()
				.at(path.clone(), spatial_object)
				.await;
		}
	};
	let task_3 = async {
		let _ = connection.object_server().at(path.clone(), handler).await;
	};

	tokio::join!(task_1, task_2, task_3);
}
