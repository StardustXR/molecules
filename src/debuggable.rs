use std::marker::PhantomData;

use stardust_xr_fusion::{fields::Field, spatial::Spatial};
use tokio::sync::watch;
use zbus::{zvariant::OwnedObjectPath, Connection};

use crate::dbus::{create_spatial_dbus, DbusObjectHandle};

pub struct Debuggable {
	reader: watch::Receiver<bool>,
	handle: DbusObjectHandle<DebuggableHandler>,
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
		create_spatial_dbus(&connection, path, handler, connection_point.cloned(), field);

		Debuggable {
			reader,
			handle: DbusObjectHandle(connection, path.clone(), PhantomData),
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
		if *self.reader.borrow() != active {
			_ = self.writer.send(active);
		}
	}
}
