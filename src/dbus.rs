use stardust_xr_fusion::core::schemas::zbus::Connection;
use std::{any::Any, marker::PhantomData};
use zbus::{object_server::Interface, zvariant::OwnedObjectPath};

#[allow(dead_code)]
pub struct DbusObjectHandles(pub(crate) Box<dyn Any>);

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
			connection
				.object_server()
				.remove::<I, _>(object_path)
				.await
				.unwrap();
		});
	}
}
