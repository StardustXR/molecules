use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use futures_util::StreamExt;
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	node::{NodeResult, NodeType},
	objects::{object_registry::ObjectInfo, FieldObject, SpatialObject},
	spatial::{Spatial, SpatialAspect, SpatialRef, Transform},
};
use std::{marker::PhantomData, path::Path};
use zbus::{
	fdo,
	message::Header,
	names::{BusName, OwnedUniqueName, UniqueName},
	zvariant::OwnedObjectPath,
};

pub struct Zoneable {
	initial_parent: SpatialRef,
	spatial: Spatial,
	captured_by: Option<UniqueName<'static>>,
}

impl Zoneable {
	pub fn create(
		connection: Connection,
		path: impl AsRef<Path>,
		parent: SpatialRef,
		field: Option<Field>,
	) -> NodeResult<DbusObjectHandles> {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();

		let spatial = Spatial::create(&parent, Transform::identity(), false)?;

		let handler = Zoneable {
			initial_parent: parent.clone(),
			spatial: spatial.clone(),
			captured_by: None,
		};

		let abort_handle = tokio::spawn({
			let connection = connection.clone();
			let path = path.clone();
			let field = field.clone();

			async move {
				let task_1 = async {
					if let Some(field) = field {
						let field_object = FieldObject::new(field).await.unwrap();
						let _ = connection
							.object_server()
							.at(path.clone(), field_object)
							.await;
					}
				};
				let task_2 = async {
					let spatial_object = SpatialObject::new(spatial).await.unwrap();
					let _ = connection
						.object_server()
						.at(path.clone(), spatial_object)
						.await;
				};
				let task_3 = async {
					let _ = connection.object_server().at(path.clone(), handler).await;
				};

				tokio::join!(task_1, task_2, task_3);

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
						let Ok(keyboard_handler) = connection
							.object_server()
							.interface::<_, Zoneable>(&path)
							.await
						else {
							continue;
						};
						keyboard_handler.get_mut().await.reset_keys(bus.to_owned());
					}
				}
			}
		})
		.abort_handle();

		Ok(DbusObjectHandles(Box::new((
			AbortOnDrop(abort_handle),
			DbusObjectHandle::<Zoneable>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection, path, PhantomData),
		))))
	}
}

#[zbus::interface(
	name = "org.stardustxr.Zoneable",
	proxy(async_name = "ZoneableProxy", gen_blocking = false)
)]
impl Zoneable {
	async fn parent(&mut self, spatial: u64) {
		let Ok(spatial_ref) = SpatialRef::import(self.initial_parent.client(), spatial).await
		else {
			return;
		};
		self.spatial.set_spatial_parent_in_place(&spatial_ref);
	}
	async fn unparent(&mut self) {
		self.spatial
			.set_spatial_parent_in_place(&self.initial_parent);
	}
}
#[zbus::interface(
	name = "org.stardustxr.CaptureZoneable",
	proxy(async_name = "CaptureZoneableProxy", gen_blocking = false)
)]
impl Zoneable {
	async fn capture(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};

		self.captured_by.replace(sender.to_owned());
	}
}

// run this one first, then send
#[tokio::test]
async fn keyboard_receive() {
	use stardust_xr_fusion::objects::*;
	use stardust_xr_fusion::spatial::*;
	use tokio::sync::mpsc;

	let client = stardust_xr_fusion::client::Client::connect().await.unwrap();
	let root = client.get_root().clone();
	let async_event_loop = client.async_event_loop();

	let field = Field::create(
		&root,
		Transform::identity(),
		stardust_xr_fusion::fields::Shape::Sphere(1.0),
	)
	.unwrap();

	async_event_loop.get_event_handle().wait().await;
	let (tx, mut rx) = mpsc::unbounded_channel();
	println!("Creating keyboard handler...");
	let _object_keyboard_handler = Zoneable::create(
		connect_client().await.unwrap(),
		"/keyboard_test",
		None,
		&field,
		move |key_info| {
			tx.send(key_info).unwrap();
		},
	);

	println!("Waiting for event loop...");
	async_event_loop.get_event_handle().wait().await;

	println!("Receiving first key info...");
	let key_info = rx.recv().await.unwrap();
	assert!(key_info.pressed);
	assert_eq!(key_info.keymap_id, 20);
	assert_eq!(key_info.key, 10);
	println!("Waiting for second key info...");
	let key_info = tokio::time::timeout(std::time::Duration::from_secs(3), rx.recv())
		.await
		.expect("Test timed out waiting for keyup event - likely hang detected")
		.expect("Channel was closed unexpectedly");
	assert!(!key_info.pressed);
	assert_eq!(key_info.keymap_id, 20);
	assert_eq!(key_info.key, 10);
}

#[tokio::test]
async fn keyboard_send() {
	use stardust_xr_fusion::objects::*;
	use zbus::names::OwnedInterfaceName;

	let client = stardust_xr_fusion::client::Client::connect().await.unwrap();
	let async_loop = client.async_event_loop();

	let connection = connect_client().await.unwrap();
	let object_registry = object_registry::ObjectRegistry::new(&connection)
		.await
		.unwrap();

	let objects =
		object_registry.get_objects(&OwnedInterfaceName::try_from("org.stardustxr.XKBv1").unwrap());
	dbg!(&objects);
	let mut join_set = tokio::task::JoinSet::new();
	for object in objects {
		if object.object_path.as_str().ends_with("/keyboard_test") {
			let connection = connection.clone();
			join_set.spawn(async move {
				let keyboard_handler = object
					.to_typed_proxy::<ZoneableProxy>(&connection)
					.await
					.unwrap();
				keyboard_handler.keymap(20).await.unwrap();
				keyboard_handler.key_state(10, true).await.unwrap();
			});
		}
	}
	while let Some(result) = join_set.join_next().await {
		result.unwrap();
	}

	async_loop.stop().await.unwrap();
}
