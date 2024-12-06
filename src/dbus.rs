use stardust_xr_fusion::core::schemas::zbus::Connection;
use std::{any::Any, marker::PhantomData};
use tokio::task::AbortHandle;
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

pub struct AbortOnDrop(pub AbortHandle);
impl Drop for AbortOnDrop {
	fn drop(&mut self) {
		self.0.abort();
	}
}
