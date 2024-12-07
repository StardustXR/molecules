use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use futures_util::StreamExt;
use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	objects::{random_object_name, FieldObject, SpatialObject},
	spatial::Spatial,
};
use std::marker::PhantomData;
use tokio::sync::mpsc;
use zbus::{
	fdo,
	message::Header,
	names::{BusName, UniqueName},
};

pub struct KeypressInfo {
	pub key: u32,
	pub pressed: bool,
	pub keymap_id: u64,
}

pub struct KeyboardHandler {
	pub key_rx: mpsc::UnboundedReceiver<KeypressInfo>,
	_object_handles: DbusObjectHandles,
}
impl KeyboardHandler {
	pub fn create(
		connection: Connection,
		connection_point: Option<&Spatial>,
		field: &Field,
	) -> Self {
		let path = random_object_name();

		let (key_tx, key_rx) = mpsc::unbounded_channel::<KeypressInfo>();

		let handler = KeyboardHandlerInner {
			keymap_ids: FxHashMap::default(),
			pressed_keys: FxHashMap::default(),
			key_tx,
		};

		let abort_handle = tokio::spawn({
			let connection_point = connection_point.cloned();
			let field = field.clone();
			let path = path.clone();
			let connection = connection.clone();

			async move {
				let task_1 = async {
					let field_object = FieldObject::new(field.clone()).await.unwrap();
					connection
						.object_server()
						.at(path.clone(), field_object)
						.await
						.unwrap();
				};
				let task_2 = async {
					if let Some(spatial) = connection_point {
						let spatial_object = SpatialObject::new(spatial.clone()).await.unwrap();
						connection
							.object_server()
							.at(path.clone(), spatial_object)
							.await
							.unwrap();
					}
				};
				let task_3 = async {
					connection
						.object_server()
						.at(path.clone(), handler)
						.await
						.unwrap();
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
							.interface::<_, KeyboardHandlerInner>(&path)
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

		let _object_handles = DbusObjectHandles(Box::new((
			AbortOnDrop(abort_handle),
			DbusObjectHandle::<KeyboardHandlerInner>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection, path, PhantomData),
		)));

		KeyboardHandler {
			key_rx,
			_object_handles,
		}
	}
}

pub struct KeyboardHandlerInner {
	keymap_ids: FxHashMap<UniqueName<'static>, u64>,
	pressed_keys: FxHashMap<UniqueName<'static>, FxHashSet<u32>>,
	key_tx: mpsc::UnboundedSender<KeypressInfo>,
}

impl KeyboardHandlerInner {
	fn reset_keys(&mut self, sender: UniqueName<'static>) {
		let Some(keymap_id) = self.keymap_ids.remove(&sender) else {
			return;
		};
		let Some(keys) = self.pressed_keys.remove(&sender) else {
			return;
		};
		for key in keys {
			let _ = self.key_tx.send(KeypressInfo {
				key,
				pressed: false,
				keymap_id,
			});
		}
	}
}

#[zbus::interface(
	name = "org.stardustxr.XKBv1",
	proxy(async_name = "KeyboardHandlerProxy")
)]
impl KeyboardHandlerInner {
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

		let _ = self.key_tx.send(KeypressInfo {
			key,
			pressed,
			keymap_id,
		});
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

#[tokio::test]
async fn keyboard() {
	use stardust_xr_fusion::objects::*;
	use stardust_xr_fusion::spatial::*;
	use zbus::names::OwnedInterfaceName;

	let client = stardust_xr_fusion::client::Client::connect().await.unwrap();
	let root = client.get_root().clone();
	let async_loop = client.async_event_loop();

	let field = Field::create(
		&root,
		Transform::identity(),
		stardust_xr_fusion::fields::Shape::Sphere(1.0),
	)
	.unwrap();

	let mut object_keyboard_handler =
		KeyboardHandler::create(connect_client().await.unwrap(), None, &field);

	let connection = connect_client().await.unwrap();
	let object_registry = object_registry::ObjectRegistry::new(&connection)
		.await
		.unwrap();

	for object in
		object_registry.get_objects(&OwnedInterfaceName::try_from("org.stardustxr.XKBv1").unwrap())
	{
		dbg!(&object);
		let connection = connection.clone();
		tokio::task::spawn(async move {
			let keyboard_handler = object
				.to_typed_proxy::<KeyboardHandlerProxy>(&connection)
				.await
				.unwrap();
			keyboard_handler.keymap(20).await.unwrap();
			keyboard_handler.key_state(10, true).await.unwrap();
		});
	}

	while let Ok(key_info) = object_keyboard_handler.key_rx.try_recv() {
		if key_info.pressed {
			println!("key {} pressed", key_info.key)
		} else {
			println!("key {} unpressed", key_info.key)
		}
		assert!(key_info.pressed);
		assert_eq!(key_info.keymap_id, 20);
		assert_eq!(key_info.key, 10);
	}

	drop(object_registry);
	drop(connection);
	println!("dropped object keyboard handler");

	while let Ok(key_info) = object_keyboard_handler.key_rx.try_recv() {
		if key_info.pressed {
			println!("key {} pressed", key_info.key)
		} else {
			println!("key {} unpressed", key_info.key)
		}
		assert_eq!(key_info.keymap_id, 20);
		assert_eq!(key_info.key, 10);
		assert!(!key_info.pressed);
	}
	async_loop.stop().await.unwrap();
}
