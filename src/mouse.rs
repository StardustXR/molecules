use crate::dbus::{DbusObjectHandle, DbusObjectHandles};
use stardust_xr_fusion::{
	core::schemas::zbus::{self, Connection},
	fields::Field,
	objects::{random_object_name, FieldObject, SpatialObject},
	spatial::Spatial,
	values::Vector2,
};
use std::marker::PhantomData;
use zbus::{message::Header, names::OwnedUniqueName};

pub struct MouseHandler {
	on_button: Box<dyn FnMut(OwnedUniqueName, u32, bool) + Send + Sync + 'static>,
	on_motion: Box<dyn FnMut(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static>,
	on_scroll_discrete: Box<dyn FnMut(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static>,
	on_scroll_continuous: Box<dyn FnMut(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static>,
}
impl MouseHandler {
	pub fn init<
		BtnHandler: FnMut(OwnedUniqueName, u32, bool) + Send + Sync + 'static,
		MotionHandler: FnMut(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static,
		ScrollDiscreteHandler: FnMut(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static,
		ScrollContinuousHandler: FnMut(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static,
	>(
		connection: Connection,
		connection_point: Option<&Spatial>,
		field: &Field,
		on_button: BtnHandler,
		on_motion: MotionHandler,
		on_scroll_discrete: ScrollDiscreteHandler,
		on_scroll_continuous: ScrollContinuousHandler,
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
						MouseHandler {
							on_button: Box::new(on_button),
							on_motion: Box::new(on_motion),
							on_scroll_discrete: Box::new(on_scroll_discrete),
							on_scroll_continuous: Box::new(on_scroll_continuous),
						},
					)
					.await
					.unwrap();
			};

			tokio::join!(task_1, task_2, task_3);
		});

		DbusObjectHandles(Box::new((
			DbusObjectHandle::<SpatialObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<FieldObject>(connection.clone(), path.clone(), PhantomData),
			DbusObjectHandle::<MouseHandler>(connection, path, PhantomData),
		)))
	}
}
#[zbus::interface(name = "org.stardustxr.Mousev1", proxy())]
impl MouseHandler {
	#[zbus(proxy(no_reply))]
	fn button(&mut self, #[zbus(header)] header: Header<'_>, button: u32, pressed: bool) {
		let Some(sender) = header.sender() else {
			return;
		};
		(self.on_button)(sender.to_owned().into(), button, pressed)
	}
	#[zbus(proxy(no_reply))]
	fn motion(&mut self, #[zbus(header)] header: Header<'_>, delta: (f32, f32)) {
		let Some(sender) = header.sender() else {
			return;
		};
		(self.on_motion)(sender.to_owned().into(), [delta.0, delta.1].into())
	}
	#[zbus(proxy(no_reply))]
	fn scroll_discrete(&mut self, #[zbus(header)] header: Header<'_>, scroll: (f32, f32)) {
		let Some(sender) = header.sender() else {
			return;
		};
		(self.on_scroll_discrete)(sender.to_owned().into(), [scroll.0, scroll.1].into())
	}
	#[zbus(proxy(no_reply))]
	fn scroll_continuous(&mut self, #[zbus(header)] header: Header<'_>, scroll: (f32, f32)) {
		let Some(sender) = header.sender() else {
			return;
		};
		(self.on_scroll_continuous)(sender.to_owned().into(), [scroll.0, scroll.1].into())
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
	let _mouse_objects = MouseHandler::init(
		connect_client().await.unwrap(),
		None,
		&field,
		move |sender, button, pressed| {
			button_tx.send((sender, button, pressed)).unwrap();
		},
		move |sender, motion| {
			motion_tx.send((sender, motion)).unwrap();
		},
		move |sender, scroll| {
			scroll_discrete_tx.send((sender, scroll)).unwrap();
		},
		move |sender, scroll| {
			scroll_continuous_tx.send((sender, scroll)).unwrap();
		},
	);

	println!("Waiting for event loop...");
	async_event_loop.get_event_handle().wait().await;

	println!("Receiving motion info...");
	let (_, motion) = motion_rx.recv().await.unwrap();
	assert_eq!(motion, [1.0, 2.0].into());

	println!("Receiving scroll discrete info...");
	let (_, scroll) = scroll_discrete_rx.recv().await.unwrap();
	assert_eq!(scroll, [0.5, 1.0].into());

	println!("Receiving scroll continuous info...");
	let (_, scroll) = scroll_continuous_rx.recv().await.unwrap();
	assert_eq!(scroll, [0.1, 0.2].into());

	println!("Receiving button info...");
	let (_, button, pressed) =
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
