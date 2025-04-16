use crate::dbus::{create_spatial_dbus, AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use futures_util::StreamExt;
use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	objects::{FieldObject, SpatialObject},
	spatial::Spatial,
};
use std::{marker::PhantomData, path::Path};
use zbus::{
	fdo,
	message::Header,
	names::{BusName, UniqueName},
	zvariant::OwnedObjectPath,
};

pub struct KeypressInfo {
	pub key: u32,
	pub pressed: bool,
	pub keymap_id: u64,
}

pub struct KeyboardHandler {
	keymap_ids: FxHashMap<UniqueName<'static>, u64>,
	pressed_keys: FxHashMap<UniqueName<'static>, FxHashSet<u32>>,
	on_key: Box<dyn FnMut(KeypressInfo) + Send + Sync + 'static>,
}

impl KeyboardHandler {
	pub fn create<F: FnMut(KeypressInfo) + Send + Sync + 'static>(
		connection: Connection,
		path: impl AsRef<Path>,
		connection_point: Option<&Spatial>,
		field: &Field,
		handler: F,
	) -> DbusObjectHandles {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();

		let handler = KeyboardHandler {
			keymap_ids: FxHashMap::default(),
			pressed_keys: FxHashMap::default(),
			on_key: Box::new(handler),
		};

		let abort_handle = tokio::spawn({
			let connection = connection.clone();
			let path = path.clone();
			let connection_point = connection_point.cloned();
			let field = field.clone();

			async move {
				create_spatial_dbus(&connection, &path, handler, connection_point, &field).await;

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
							.interface::<_, KeyboardHandler>(&path)
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

		DbusObjectHandles(Box::new((
			AbortOnDrop(abort_handle),
			DbusObjectHandle::<KeyboardHandler>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection, path, PhantomData),
		)))
	}

	fn reset_keys(&mut self, sender: UniqueName<'static>) {
		let Some(keymap_id) = self.keymap_ids.remove(&sender) else {
			return;
		};
		let Some(keys) = self.pressed_keys.remove(&sender) else {
			return;
		};
		for key in keys {
			let key_info = KeypressInfo {
				key,
				pressed: false,
				keymap_id,
			};
			(self.on_key)(key_info);
		}
	}
}

#[zbus::interface(
	name = "org.stardustxr.XKBv1",
	proxy(async_name = "KeyboardHandlerProxy")
)]
impl KeyboardHandler {
	#[zbus(proxy(no_reply))]
	fn keymap(
		&mut self,
		#[zbus(header)] header: Header<'_>,
		keymap_id: u64,
	) -> zbus::fdo::Result<()> {
		let Some(sender) = header.sender() else {
			return Ok(());
		};
		let sender = sender.to_owned();
		self.keymap_ids.insert(sender.clone(), keymap_id);
		Ok(())
	}

	#[zbus(proxy(no_reply))]
	fn key_state(&mut self, #[zbus(header)] header: Header<'_>, key: u32, pressed: bool) {
		let Some(sender) = header.sender() else {
			return;
		};
		let sender = sender.to_owned();
		let Some(keymap_id) = self.keymap_ids.get(&sender).cloned() else {
			return;
		};

		let sender_entry = self.pressed_keys.entry(sender).or_default();
		if pressed {
			sender_entry.insert(key);
		} else {
			sender_entry.remove(&key);
		}

		let key_info = KeypressInfo {
			key,
			pressed,
			keymap_id,
		};
		(self.on_key)(key_info);
	}

	#[zbus(proxy(no_reply))]
	fn reset(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};
		let sender = sender.to_owned();
		self.reset_keys(sender)
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
	let _object_keyboard_handler = KeyboardHandler::create(
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
					.to_typed_proxy::<KeyboardHandlerProxy>(&connection)
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
