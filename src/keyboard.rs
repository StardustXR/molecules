use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	items::panel::{PanelItem, PanelItemAspect, SurfaceId},
	objects::{random_object_name, FieldObject, SpatialObject},
	spatial::Spatial,
};
use std::{any::Any, marker::PhantomData};
use zbus::{
	message::Header, names::UniqueName, object_server::Interface, zvariant::OwnedObjectPath,
};

#[allow(dead_code)]
pub struct DbusObjectHandles(Box<dyn Any>);

pub struct DbusObjectHandle<I: Interface>(Connection, OwnedObjectPath, PhantomData<I>);
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

pub struct KeyboardHandler<F: Fn(u64, u32, bool) + Send + Sync + 'static> {
	keymap_ids: FxHashMap<UniqueName<'static>, u64>,
	on_key: F,
}
impl<F: Fn(u64, u32, bool) + Send + Sync + 'static> KeyboardHandler<F> {
	pub fn init(
		connection: Connection,
		connection_point: Option<&Spatial>,
		field: &Field,
		on_key: F,
	) -> DbusObjectHandles {
		let path = random_object_name();
		let path_clone = path.clone();

		let connection_clone = connection.clone();
		let connection_point = connection_point.cloned();
		let field = field.clone();
		tokio::spawn(async move {
			let task_1 = async {
				let field_object = FieldObject::new(field.clone()).await.unwrap();
				connection_clone
					.object_server()
					.at(path_clone.clone(), field_object)
					.await
					.unwrap();
			};
			let task_2 = async {
				if let Some(spatial) = connection_point {
					let spatial_object = SpatialObject::new(spatial.clone()).await.unwrap();
					connection_clone
						.object_server()
						.at(path_clone.clone(), spatial_object)
						.await
						.unwrap();
				}
			};
			let task_3 = async {
				connection_clone
					.object_server()
					.at(
						path_clone.clone(),
						KeyboardHandler {
							keymap_ids: FxHashMap::default(),
							on_key,
						},
					)
					.await
					.unwrap();
			};

			tokio::join!(task_1, task_2, task_3);
		});

		DbusObjectHandles(Box::new((
			DbusObjectHandle::<KeyboardHandler<F>>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection, path, PhantomData),
		)))
	}
}
#[zbus::interface(name = "org.stardustxr.XKBv1", proxy())]
impl<F: Fn(u64, u32, bool) + Send + Sync + 'static> KeyboardHandler<F> {
	#[zbus(proxy(no_reply))]
	fn keymap(
		&mut self,
		#[zbus(header)] header: Header<'_>,
		keymap_id: u64,
	) -> zbus::fdo::Result<()> {
		let Some(sender) = header.sender() else {
			return Ok(());
		};
		self.keymap_ids.insert(sender.to_owned(), keymap_id);
		Ok(())
	}
	#[zbus(proxy(no_reply))]
	fn key_state(&mut self, #[zbus(header)] header: Header<'_>, key: u32, pressed: bool) {
		let Some(sender) = header.sender() else {
			return;
		};
		let Some(keymap_id) = self.keymap_ids.get(sender) else {
			return;
		};

		(self.on_key)(*keymap_id, key, pressed)
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

	let connection = connect_client().await.unwrap();

	let _keyboard_objects = KeyboardHandler::init(
		connection.clone(),
		None,
		&field,
		move |keymap_id, key, pressed| {
			println!("key pressed");
			assert_eq!(keymap_id, 20);
			assert_eq!(key, 10);
			assert!(pressed);
			std::process::exit(0);
		},
	);

	let object_registry = object_registry::ObjectRegistry::new(&connection)
		.await
		.unwrap();

	// dbg!(&*object_registry.get_watch().borrow());

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
	let _ = client.sync_event_loop(|_, _| {}).await;
}
