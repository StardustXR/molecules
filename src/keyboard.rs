use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles};
use futures_util::StreamExt;
use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	items::panel::{PanelItem, PanelItemAspect, SurfaceId},
	objects::{random_object_name, FieldObject, SpatialObject},
	spatial::Spatial,
};
use std::marker::PhantomData;
use zbus::{
	fdo,
	message::Header,
	names::{BusName, UniqueName},
};

pub struct KeyboardHandler {
	keymap_ids: FxHashMap<UniqueName<'static>, u64>,
	pressed_keys: FxHashMap<UniqueName<'static>, FxHashSet<u32>>,
	on_key: Box<dyn Fn(u64, u32, bool) + Send + Sync + 'static>,
}

impl KeyboardHandler {
	pub fn init<F: Fn(u64, u32, bool) + Send + Sync + 'static>(
		connection: Connection,
		connection_point: Option<&Spatial>,
		field: &Field,
		on_key: F,
	) -> DbusObjectHandles {
		let path = random_object_name();

		let handler = KeyboardHandler {
			keymap_ids: FxHashMap::default(),
			pressed_keys: FxHashMap::default(),
			on_key: Box::new(on_key),
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
			(self.on_key)(keymap_id, key, false);
		}
	}
}

#[zbus::interface(name = "org.stardustxr.XKBv1", proxy())]
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
		let Some(keymap_id) = self.keymap_ids.get(&sender) else {
			return;
		};

		let sender_entry = self.pressed_keys.entry(sender).or_default();
		if pressed {
			sender_entry.insert(key);
		} else {
			sender_entry.remove(&key);
		}

		(self.on_key)(*keymap_id, key, pressed)
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

pub fn make_keyboard_handler(
	connection: &Connection,
	connection_point: Option<&Spatial>,
	field: &Field,
	panel_item: PanelItem,
	surface_id: SurfaceId,
) -> DbusObjectHandles {
	KeyboardHandler::init(
		connection.clone(),
		connection_point,
		field,
		move |keymap_id, key, pressed| {
			let _ = panel_item.keyboard_keys(
				surface_id.clone(),
				keymap_id,
				&[if pressed { key as i32 } else { -(key as i32) }],
			);
		},
	)
}

#[tokio::test]
async fn keyboard() {
	use stardust_xr_fusion::objects::*;
	use stardust_xr_fusion::spatial::*;
	use zbus::names::OwnedInterfaceName;

	let mut client = stardust_xr_fusion::client::Client::connect().await.unwrap();

	let field = Field::create(
		client.get_root(),
		Transform::identity(),
		stardust_xr_fusion::fields::Shape::Sphere(1.0),
	)
	.unwrap();

	let pressed_notifier = std::sync::Arc::new(tokio::sync::Notify::new());
	let _object_keyboard_handler =
		KeyboardHandler::init(connect_client().await.unwrap(), None, &field, {
			let pressed_notifier = pressed_notifier.clone();
			move |keymap_id, key, pressed| {
				if pressed {
					println!("key {key} pressed")
				} else {
					println!("key {key} unpressed")
				};
				assert_eq!(keymap_id, 20);
				assert_eq!(key, 10);
				if pressed {
					pressed_notifier.notify_waiters();
				} else {
					std::process::exit(0);
				}
			}
		});

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
	pressed_notifier.notified().await;
	drop(object_registry);
	drop(connection);
	println!("dropped object keyboard handler");
	let _ = client.sync_event_loop(|_, _| {}).await;
}
