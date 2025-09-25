use std::marker::PhantomData;

use stardust_xr_fusion::{fields::Field, spatial::Spatial};
use tokio::sync::watch;
use zbus::{Connection, zvariant::OwnedObjectPath};

use crate::dbus::{DbusObjectHandle, create_spatial_dbus};

pub struct Debuggable {
	pub reader: watch::Receiver<bool>,
	_handle: DbusObjectHandle<DebuggableHandler>,
}
impl Debuggable {
	pub fn create(
		connection: Connection,
		path: &OwnedObjectPath,
		field: &Field,
		connection_point: Option<&Spatial>,
	) -> Self {
		let (writer, reader) = watch::channel(false);
		let handler = DebuggableHandler {
			writer,
			reader: reader.clone(),
		};
		tokio::task::spawn({
			let connection = connection.clone();
			let path = path.clone();
			let field = field.clone();
			let connection_point = connection_point.cloned();
			async move {
				create_spatial_dbus(&connection, &path, handler, connection_point, &field).await
			}
		});

		Debuggable {
			reader,
			_handle: DbusObjectHandle(connection, path.clone(), PhantomData),
		}
	}
	pub fn active(&self) -> bool {
		*self.reader.borrow()
	}
}

pub struct DebuggableHandler {
	writer: watch::Sender<bool>,
	reader: watch::Receiver<bool>,
}

#[zbus::interface(
	name = "org.stardustxr.Debuggablev1",
	proxy(async_name = "DebuggableHandlerProxy")
)]
impl DebuggableHandler {
	#[zbus(property)]
	fn active(&self) -> bool {
		*self.reader.borrow()
	}
	#[zbus(property)]
	fn set_active(&self, active: bool) {
		_ = self.writer.send(active);
	}
}

#[tokio::test]
async fn debuggable() {
	use stardust_xr_fusion::{fields::Field, spatial::Spatial};
	use zbus::zvariant::OwnedObjectPath;
	// Create a mock connection
	let connection = Connection::session().await.unwrap();

	let client = stardust_xr_fusion::Client::connect().await.unwrap();
	let ch = client.handle();
	let _async_event_loop = client.async_event_loop();

	// Create a mock field and spatial
	let field = Field::create(
		ch.get_root(),
		stardust_xr_fusion::spatial::Transform::identity(),
		stardust_xr_fusion::fields::Shape::Sphere(0.05),
	)
	.unwrap();
	let spatial = Spatial::create(
		&field,
		stardust_xr_fusion::spatial::Transform::identity(),
		false,
	)
	.unwrap();

	// Create a mock object path
	let path = OwnedObjectPath::try_from("/org/stardustxr/DebuggableTest").unwrap();

	// Create the Debuggable instance
	let debuggable = Debuggable::create(connection, &path, &field, Some(&spatial));

	// Assert initial state is false
	assert!(!debuggable.active());

	// Await the change in the reader
	let mut reader = debuggable.reader.clone();
	reader.changed().await.unwrap();

	// Assert the state is now true
	assert!(debuggable.active());
}
