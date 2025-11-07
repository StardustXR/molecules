use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use stardust_xr_fusion::{
	fields::Field,
	node::NodeResult,
	objects::{FieldObject, SpatialObject},
	spatial::Spatial,
};
use std::{marker::PhantomData, path::Path};
use tokio::sync::mpsc;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub struct Derezzable {
	pub receiver: mpsc::Receiver<()>,
	_object_handles: DbusObjectHandles,
}
impl Derezzable {
	pub fn create(
		connection: Connection,
		path: impl AsRef<Path>,
		spatial: Spatial,
		field: Option<Field>,
	) -> NodeResult<Self> {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();

		let (derez_tx, derez_rx) = mpsc::channel(6);
		let derez = DerezInner(derez_tx);

		let abort_handle = tokio::spawn({
			let connection = connection.clone();
			let path = path.clone();

			async move {
				if let Some(field) = field {
					let Ok(field_object) = FieldObject::new(field).await else {
						return;
					};
					let _ = connection
						.object_server()
						.at(path.clone(), field_object)
						.await;
				}
				let Ok(spatial_object) = SpatialObject::new(spatial).await else {
					return;
				};
				let _ = connection
					.object_server()
					.at(path.clone(), spatial_object)
					.await;
				let _ = connection.object_server().at(path.clone(), derez).await;
			}
		})
		.abort_handle();

		Ok(Derezzable {
			receiver: derez_rx,
			_object_handles: DbusObjectHandles(Box::new((
				AbortOnDrop(abort_handle),
				DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
				DbusObjectHandle::<FieldObject>(connection.clone(), path.clone(), PhantomData),
				DbusObjectHandle::<DerezInner>(connection.clone(), path.clone(), PhantomData),
			))),
		})
	}
}

struct DerezInner(mpsc::Sender<()>);
#[zbus::interface(
	name = "org.stardustxr.Derezzable",
	proxy(async_name = "DerezzableHandlerProxy")
)]
impl DerezInner {
	async fn derez(&self) {
		let _ = self.0.send(()).await;
	}
}

#[tokio::test]
async fn derezzable_dbus() {
	tokio::spawn(async {
		tokio::time::sleep(std::time::Duration::from_secs(30)).await;
		panic!("Timed out")
	});

	let client = stardust_xr_fusion::Client::connect().await.unwrap();
	let event_loop = client.async_event_loop();
	let spatial = Spatial::create(
		event_loop.client_handle.get_root(),
		stardust_xr_fusion::spatial::Transform::identity(),
		false,
	)
	.unwrap();
	let connection = stardust_xr_fusion::core::schemas::dbus::connect_client()
		.await
		.unwrap();

	let mut derez = Derezzable::create(connection.clone(), "/", spatial, None).unwrap();
	derez.receiver.recv().await.unwrap();
	println!("Received derez");
}
