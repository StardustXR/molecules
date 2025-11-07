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

pub struct Destroy {
	pub receiver: mpsc::Receiver<()>,
	_object_handles: DbusObjectHandles,
}
impl Destroy {
	pub fn create(
		connection: Connection,
		path: impl AsRef<Path>,
		spatial: Spatial,
		field: Option<Field>,
	) -> NodeResult<Self> {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();

		let (destroy_tx, destroy_rx) = mpsc::channel(6);
		let destroy = DestroyInner(destroy_tx);

		let abort_handle = tokio::spawn({
			let connection = connection.clone();
			let path = path.clone();

			async move {
				println!("[destroy] Starting object registration");
				if let Some(field) = field {
					println!("[destroy] Creating field object");
					let field_object = match FieldObject::new(field).await {
						Ok(obj) => obj,
						Err(e) => {
							eprintln!("[destroy] Failed to create field object: {:?}", e);
							return;
						}
					};
					if let Err(e) = connection
						.object_server()
						.at(path.clone(), field_object)
						.await
					{
						eprintln!("[destroy] Failed to register field object: {:?}", e);
					}
				}
				println!("[destroy] Creating spatial object");
				let spatial_object = match SpatialObject::new(spatial).await {
					Ok(obj) => obj,
					Err(e) => {
						eprintln!("[destroy] Failed to create spatial object: {:?}", e);
						return;
					}
				};
				if let Err(e) = connection
					.object_server()
					.at(path.clone(), spatial_object)
					.await
				{
					eprintln!("[destroy] Failed to register spatial object: {:?}", e);
				}
				println!("[destroy] Registering destroy interface");
				if let Err(e) = connection.object_server().at(path.clone(), destroy).await {
					eprintln!("[destroy] Failed to register destroy interface: {:?}", e);
				}
				println!("[destroy] All registrations complete");
			}
		})
		.abort_handle();

		Ok(Destroy {
			receiver: destroy_rx,
			_object_handles: DbusObjectHandles(Box::new((
				AbortOnDrop(abort_handle),
				DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
				DbusObjectHandle::<FieldObject>(connection.clone(), path.clone(), PhantomData),
				DbusObjectHandle::<DestroyInner>(connection.clone(), path.clone(), PhantomData),
			))),
		})
	}
}

struct DestroyInner(mpsc::Sender<()>);
#[zbus::interface(name = "org.stardustxr.Destroy")]
impl DestroyInner {
	async fn destroy(&self) {
		println!("Destroy called");
		let _ = self.0.send(()).await;
	}
}

#[tokio::test]
async fn destroy_dbus() {
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

	let mut destroy = Destroy::create(connection.clone(), "/", spatial, None).unwrap();
	destroy.receiver.recv().await.unwrap();
	println!("Received destroy");
}
