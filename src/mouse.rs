use crate::dbus::{AbortOnDrop, DbusObjectHandle, DbusObjectHandles, create_spatial_dbus};
use futures_util::StreamExt;
use rustc_hash::{FxHashMap, FxHashSet};
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	objects::{FieldObject, SpatialObject},
	spatial::Spatial,
	values::Vector2,
};
use std::{marker::PhantomData, path::Path};
use zbus::{
	fdo,
	message::Header,
	names::{BusName, UniqueName},
	zvariant::OwnedObjectPath,
};

pub struct MouseHandler {
	pressed_buttons: FxHashMap<UniqueName<'static>, FxHashSet<u32>>,
	on_button: Box<dyn FnMut(u32, bool) + Send + Sync + 'static>,
	on_motion: Box<dyn FnMut(Vector2<f32>) + Send + Sync + 'static>,
	on_scroll_discrete: Box<dyn FnMut(Vector2<f32>) + Send + Sync + 'static>,
	on_scroll_continuous: Box<dyn FnMut(Vector2<f32>) + Send + Sync + 'static>,
}
impl MouseHandler {
	#[allow(clippy::too_many_arguments)]
	pub fn create<
		BtnHandler: FnMut(u32, bool) + Send + Sync + 'static,
		MotionHandler: FnMut(Vector2<f32>) + Send + Sync + 'static,
		ScrollDiscreteHandler: FnMut(Vector2<f32>) + Send + Sync + 'static,
		ScrollContinuousHandler: FnMut(Vector2<f32>) + Send + Sync + 'static,
	>(
		connection: Connection,
		path: impl AsRef<Path>,
		connection_point: Option<&Spatial>,
		field: &Field,
		on_button: BtnHandler,
		on_motion: MotionHandler,
		on_scroll_discrete: ScrollDiscreteHandler,
		on_scroll_continuous: ScrollContinuousHandler,
	) -> DbusObjectHandles {
		let path: OwnedObjectPath = path.as_ref().to_str().unwrap().try_into().unwrap();
		let handler = MouseHandler {
			pressed_buttons: FxHashMap::default(),
			on_button: Box::new(on_button),
			on_motion: Box::new(on_motion),
			on_scroll_discrete: Box::new(on_scroll_discrete),
			on_scroll_continuous: Box::new(on_scroll_continuous),
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
						let Ok(mouse_handler) = connection
							.object_server()
							.interface::<_, MouseHandler>(&path)
							.await
						else {
							continue;
						};
						mouse_handler.get_mut().await.reset_buttons(bus.to_owned());
					}
				}
			}
		})
		.abort_handle();

		DbusObjectHandles(Box::new((
			AbortOnDrop(abort_handle),
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<MouseHandler>(connection, path, PhantomData),
		)))
	}

	fn reset_buttons(&mut self, sender: UniqueName<'static>) {
		if let Some(buttons) = self.pressed_buttons.remove(&sender) {
			for button in buttons {
				(self.on_button)(button, false);
			}
		}
	}
}

#[zbus::interface(name = "org.stardustxr.Mousev1", proxy())]
impl MouseHandler {
	#[zbus(proxy(no_reply))]
	fn button(&mut self, #[zbus(header)] header: Header<'_>, button: u32, pressed: bool) {
		let Some(sender) = header.sender() else {
			return;
		};
		let sender = sender.to_owned();

		let sender_entry = self.pressed_buttons.entry(sender).or_default();
		if pressed {
			sender_entry.insert(button);
		} else {
			sender_entry.remove(&button);
		}

		(self.on_button)(button, pressed)
	}

	#[zbus(proxy(no_reply))]
	fn motion(&mut self, #[zbus(header)] _header: Header<'_>, delta: (f32, f32)) {
		(self.on_motion)([delta.0, delta.1].into())
	}

	#[zbus(proxy(no_reply))]
	fn scroll_discrete(&mut self, #[zbus(header)] _header: Header<'_>, scroll: (f32, f32)) {
		(self.on_scroll_discrete)([scroll.0, scroll.1].into())
	}

	#[zbus(proxy(no_reply))]
	fn scroll_continuous(&mut self, #[zbus(header)] _header: Header<'_>, scroll: (f32, f32)) {
		(self.on_scroll_continuous)([scroll.0, scroll.1].into())
	}

	#[zbus(proxy(no_reply))]
	fn reset(&mut self, #[zbus(header)] header: Header<'_>) {
		let Some(sender) = header.sender() else {
			return;
		};
		let sender = sender.to_owned();
		self.reset_buttons(sender);
	}
}

#[tokio::test]
async fn mouse_receive() {
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
	let (button_tx, mut button_rx) = mpsc::unbounded_channel();
	let (motion_tx, mut motion_rx) = mpsc::unbounded_channel();
	let (scroll_discrete_tx, mut scroll_discrete_rx) = mpsc::unbounded_channel();
	let (scroll_continuous_tx, mut scroll_continuous_rx) = mpsc::unbounded_channel();

	println!("Creating mouse handler...");
	let _mouse_objects = MouseHandler::create(
		connect_client().await.unwrap(),
		"/mouse_test",
		None,
		&field,
		move |button, pressed| {
			button_tx.send((button, pressed)).unwrap();
		},
		move |motion| {
			motion_tx.send(motion).unwrap();
		},
		move |scroll| {
			scroll_discrete_tx.send(scroll).unwrap();
		},
		move |scroll| {
			scroll_continuous_tx.send(scroll).unwrap();
		},
	);

	println!("Waiting for event loop...");
	async_event_loop.get_event_handle().wait().await;

	println!("Receiving motion info...");
	let motion = motion_rx.recv().await.unwrap();
	assert_eq!(motion, [1.0, 2.0].into());

	println!("Receiving scroll discrete info...");
	let scroll = scroll_discrete_rx.recv().await.unwrap();
	assert_eq!(scroll, [0.5, 1.0].into());

	println!("Receiving scroll continuous info...");
	let scroll = scroll_continuous_rx.recv().await.unwrap();
	assert_eq!(scroll, [0.1, 0.2].into());

	println!("Receiving button info...");
	let (button, pressed) =
		tokio::time::timeout(std::time::Duration::from_secs(3), button_rx.recv())
			.await
			.expect("Test timed out waiting for button event - likely hang detected")
			.expect("Channel was closed unexpectedly");
	assert_eq!(button, 10);
	assert!(pressed);
}

#[tokio::test]
async fn mouse_send() {
	use stardust_xr_fusion::objects::*;
	use zbus::names::OwnedInterfaceName;

	let client = stardust_xr_fusion::client::Client::connect().await.unwrap();
	let async_loop = client.async_event_loop();

	let connection = connect_client().await.unwrap();
	let object_registry = object_registry::ObjectRegistry::new(&connection)
		.await
		.unwrap();

	let objects = object_registry
		.get_objects(&OwnedInterfaceName::try_from("org.stardustxr.Mousev1").unwrap());
	dbg!(&objects);
	let mut join_set = tokio::task::JoinSet::new();
	for object in objects {
		let connection = connection.clone();
		join_set.spawn(async move {
			let mouse_handler = object
				.to_typed_proxy::<MouseHandlerProxy>(&connection)
				.await
				.unwrap();
			mouse_handler.motion((1.0, 2.0)).await.unwrap();
			mouse_handler.scroll_discrete((0.5, 1.0)).await.unwrap();
			mouse_handler.scroll_continuous((0.1, 0.2)).await.unwrap();
			mouse_handler.button(10, true).await.unwrap();
		});
	}
	while let Some(result) = join_set.join_next().await {
		result.unwrap();
	}

	async_loop.stop().await.unwrap();
}
