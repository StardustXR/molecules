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
	on_button: Box<dyn Fn(OwnedUniqueName, u32, bool) + Send + Sync + 'static>,
	on_motion: Box<dyn Fn(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static>,
	on_scroll_discrete: Box<dyn Fn(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static>,
	on_scroll_continuous: Box<dyn Fn(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static>,
}
impl MouseHandler {
	pub fn init<
		BtnHandler: Fn(OwnedUniqueName, u32, bool) + Send + Sync + 'static,
		MotionHandler: Fn(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static,
		ScrollDiscreteHandler: Fn(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static,
		ScrollContinuousHandler: Fn(OwnedUniqueName, Vector2<f32>) + Send + Sync + 'static,
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
async fn mouse() {
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

	let _mouse_objects = MouseHandler::init(
		connection.clone(),
		None,
		&field,
		move |mouse_id, button, pressed| {
			println!("button pressed");
			dbg!(mouse_id);
			assert_eq!(button, 10);
			assert!(pressed);
			std::process::exit(0);
		},
		move |mouse_id, motion| {
			println!("motion");
			dbg!(mouse_id);
			assert_eq!(motion, [1.0, 2.0].into());
		},
		move |mouse_id, scroll| {
			println!("discrete scroll");
			dbg!(mouse_id);
			assert_eq!(scroll, [0.5, 1.0].into());
		},
		move |mouse_id, scroll| {
			println!("continuous scroll");
			dbg!(mouse_id);
			assert_eq!(scroll, [0.1, 0.2].into());
		},
	);

	let object_registry = object_registry::ObjectRegistry::new(&connection)
		.await
		.unwrap();

	for object in object_registry
		.get_objects(&OwnedInterfaceName::try_from("org.stardustxr.Mousev1").unwrap())
	{
		dbg!(&object);
		let connection = connection.clone();
		tokio::task::spawn(async move {
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
	let _ = client.sync_event_loop(|_, _| {}).await;
}
