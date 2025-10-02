use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use futures_util::StreamExt;
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	node::{NodeResult, NodeType},
	objects::{FieldObject, SpatialObject},
	spatial::{Spatial, SpatialAspect, SpatialRef, Transform},
};
use std::{marker::PhantomData, ops::Deref, path::Path};
use tokio::sync::watch;
use zbus::{
	fdo,
	message::Header,
	names::{BusName, UniqueName},
	zvariant::OwnedObjectPath,
};

pub struct Reparentable {
	initial_parent: SpatialRef,
	spatial: Spatial,
	captured_by: watch::Receiver<Option<UniqueName<'static>>>,
}
impl Reparentable {
	pub fn create(
		connection: Connection,
		path: impl AsRef<Path>,
		parent: SpatialRef,
		field: Option<Field>,
	) -> NodeResult<DbusObjectHandles> {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();

		let spatial = Spatial::create(&parent, Transform::identity(), false)?;

		let (captured_by_sender, captured_by) = watch::channel(None);
		let zoneable = Reparentable {
			initial_parent: parent.clone(),
			spatial: spatial.clone(),
			captured_by,
		};
		let capture_zoneable = ReparentLock(captured_by_sender);

		let abort_handle = tokio::spawn({
			let connection = connection.clone();
			let path = path.clone();
			let field = field.clone();

			async move {
				if let Some(field) = field {
					let field_object = FieldObject::new(field).await.unwrap();
					let _ = connection
						.object_server()
						.at(path.clone(), field_object)
						.await;
				}
				let spatial_object = SpatialObject::new(spatial).await.unwrap();
				let _ = connection
					.object_server()
					.at(path.clone(), spatial_object)
					.await;
				let _ = connection.object_server().at(path.clone(), zoneable).await;
				let _ = connection
					.object_server()
					.at(path.clone(), capture_zoneable)
					.await;

				let Ok(dbus_proxy) = fdo::DBusProxy::new(&connection).await else {
					return;
				};
				let Ok(mut name_changes) = dbus_proxy.receive_name_owner_changed().await else {
					return;
				};
				while let Some(signal) = name_changes.next().await {
					let args = signal.args().unwrap();

					if args.new_owner.is_none() {
						let BusName::Unique(bus) = args.name else {
							continue;
						};
						let Ok(interface) = connection
							.object_server()
							.interface::<_, ReparentLock>(&path)
							.await
						else {
							continue;
						};
						interface.get_mut().await.release_body(bus.to_owned()).await;
					}
				}
			}
		})
		.abort_handle();

		Ok(DbusObjectHandles(Box::new((
			AbortOnDrop(abort_handle),
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<Reparentable>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<ReparentLock>(connection.clone(), path.clone(), PhantomData),
		))))
	}
}
#[zbus::interface(
	name = "org.stardustxr.Reparentable",
	proxy(async_name = "ReparentableProxy", gen_blocking = false)
)]
impl Reparentable {
	async fn parent(&mut self, #[zbus(header)] header: Header<'_>, spatial: u64) {
		if let Some(captured) = self.captured_by.borrow_and_update().deref()
			&& let Some(sender) = header.sender()
			&& captured != sender
		{
			return;
		}
		let Ok(spatial_ref) = SpatialRef::import(self.initial_parent.client(), spatial).await
		else {
			return;
		};
		let _ = self.spatial.set_spatial_parent_in_place(&spatial_ref);
	}
	async fn unparent(&mut self, #[zbus(header)] header: Header<'_>) {
		if let Some(captured) = self.captured_by.borrow_and_update().deref()
			&& let Some(sender) = header.sender()
			&& captured != sender
		{
			return;
		}
		let _ = self
			.spatial
			.set_spatial_parent_in_place(&self.initial_parent);
	}
	/// Use this to reset the local transform of the zoneable object relative to an object.
	async fn reset_local_transform(
		&mut self,
		#[zbus(header)] header: Header<'_>,
		relative_to: u64,
	) {
		if let Some(captured) = self.captured_by.borrow_and_update().deref()
			&& let Some(sender) = header.sender()
			&& captured != sender
		{
			return;
		}

		let Ok(relative_to) = SpatialRef::import(self.initial_parent.client(), relative_to).await
		else {
			return;
		};
		let _ = self
			.spatial
			.set_relative_transform(&relative_to, Transform::identity());
	}
}

struct ReparentLock(watch::Sender<Option<UniqueName<'static>>>);
impl ReparentLock {
	async fn release_body(&mut self, sender: UniqueName<'static>) {
		self.0.send_modify(move |capture| {
			if let Some(current_capture) = capture
				&& current_capture == &sender
			{
				*capture = None;
			}
		});
	}
}
#[zbus::interface(
	name = "org.stardustxr.ReparentLock",
	proxy(async_name = "ReparentLockProxy", gen_blocking = false)
)]
impl ReparentLock {
	async fn lock(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};

		let _ = self.0.send(Some(sender.to_owned()));
	}
	async fn unlock(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};
		self.release_body(sender.to_owned()).await;
	}
}
