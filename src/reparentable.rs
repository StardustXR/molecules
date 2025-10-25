use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use futures_util::StreamExt;
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	node::{NodeResult, NodeType},
	objects::{FieldObject, SpatialObject},
	spatial::{Spatial, SpatialAspect, SpatialRef, SpatialRefAspect, Transform},
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
	pub spatial: SpatialRef,
	_object_handles: DbusObjectHandles,
}
impl Reparentable {
	pub fn create(
		connection: Connection,
		path: impl AsRef<Path>,
		initial_parent: SpatialRef,
		spatial: Spatial,
		field: Option<Field>,
	) -> NodeResult<Self> {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();

		spatial.set_spatial_parent_in_place(&initial_parent)?;

		let (captured_by_sender, captured_by) = watch::channel(None);
		let reparentable = ReparentableInner {
			initial_parent: initial_parent.clone(),
			spatial: spatial.clone(),
			captured_by,
			parented_to: None,
		};
		let reparent_lock = ReparentLock {
			watch: captured_by_sender,
			initial_parent,
			spatial: spatial.clone().as_spatial_ref(),
			lock_transform: None,
		};

		let abort_handle = tokio::spawn({
			let connection = connection.clone();
			let path = path.clone();
			let field = field.clone();
			let spatial = spatial.clone();

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
				let _ = connection
					.object_server()
					.at(path.clone(), reparentable)
					.await;
				let _ = connection
					.object_server()
					.at(path.clone(), reparent_lock)
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
						let Ok(lock_interface) = connection
							.object_server()
							.interface::<_, ReparentLock>(&path)
							.await
						else {
							continue;
						};
						let unlock_transform =
							lock_interface.get_mut().await.release_body(bus.to_owned());

						let Ok(interface) = connection
							.object_server()
							.interface::<_, ReparentableInner>(&path)
							.await
						else {
							continue;
						};
						interface
							.get_mut()
							.await
							.client_lost(bus.to_owned(), unlock_transform);
					}
				}
			}
		})
		.abort_handle();

		Ok(Reparentable {
			spatial: spatial.as_spatial_ref(),
			_object_handles: DbusObjectHandles(Box::new((
				AbortOnDrop(abort_handle),
				DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
				DbusObjectHandle::<FieldObject>(connection.clone(), path.clone(), PhantomData),
				DbusObjectHandle::<ReparentableInner>(
					connection.clone(),
					path.clone(),
					PhantomData,
				),
				DbusObjectHandle::<ReparentLock>(connection.clone(), path.clone(), PhantomData),
			))),
		})
	}
}

struct ReparentableInner {
	initial_parent: SpatialRef,
	spatial: Spatial,
	captured_by: watch::Receiver<Option<UniqueName<'static>>>,
	parented_to: Option<UniqueName<'static>>,
}
impl ReparentableInner {
	fn client_lost(&mut self, name: UniqueName<'static>, lock_transform: Option<Transform>) {
		if self.parented_to.as_ref() == Some(&name) {
			self.parented_to = None;
			if let Some(transform) = lock_transform {
				self.spatial.set_spatial_parent(&self.initial_parent);
				self.spatial.set_local_transform(transform);
			} else {
				self.spatial
					.set_spatial_parent_in_place(&self.initial_parent);
			}
		}
	}
}
#[zbus::interface(name = "org.stardustxr.Reparentable")]
impl ReparentableInner {
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
		if let Some(sender) = header.sender() {
			self.parented_to = Some(sender.to_owned());
		}
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
		self.parented_to.take();
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

struct ReparentLock {
	watch: watch::Sender<Option<UniqueName<'static>>>,
	initial_parent: SpatialRef,
	spatial: SpatialRef,
	lock_transform: Option<Transform>,
}
impl ReparentLock {
	fn release_body(&mut self, sender: UniqueName<'static>) -> Option<Transform> {
		let uncaptured = self.watch.send_if_modified(move |capture| {
			if let Some(current_capture) = capture
				&& current_capture == &sender
			{
				*capture = None;
				true
			} else {
				false
			}
		});
		if uncaptured {
			self.lock_transform.take()
		} else {
			None
		}
	}
}
#[zbus::interface(name = "org.stardustxr.ReparentLock")]
impl ReparentLock {
	async fn lock(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};

		self.lock_transform = self.spatial.get_transform(&self.initial_parent).await.ok();
		let _ = self.watch.send(Some(sender.to_owned()));
	}
	async fn unlock(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};
		self.release_body(sender.to_owned());
	}
}
