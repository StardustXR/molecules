use stardust_xr_fusion::{
	client::Client,
	drawable::Model,
	fields::{Field, Shape},
	node::NodeType,
	project_local_resources,
	root::{ClientState, RootAspect, RootEvent},
	spatial::{Spatial, SpatialAspect, SpatialRefAspect, Transform},
	values::ResourceID,
};
use stardust_xr_molecules::{
	DebugSettings, FrameSensitive, Grabbable, GrabbableSettings, PointerMode, UIElement,
	VisualDebug,
};
use tracing_subscriber::EnvFilter;
use zbus::{conn::Builder, fdo::ObjectManager};

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let mut client = Client::connect().await.unwrap();
	client
		.setup_resources(&[&project_local_resources!("res")])
		.unwrap();
	let client_handle = client.handle();

	let model = Model::create(
		client.get_root(),
		Transform::from_scale([0.5; 3]),
		&ResourceID::new_namespaced("molecules", "grabbable"),
	)
	.unwrap();

	let bounds = client
		.await_method(model.get_relative_bounding_box(client_handle.get_root()))
		.await
		.unwrap()
		.unwrap();
	dbg!(&bounds);
	let field = Field::create(
		&model,
		Transform::from_translation(bounds.center),
		Shape::Box(bounds.size),
	)
	.unwrap();

	let root = Spatial::create(
		client.get_root(),
		Transform::from_translation([0.1, 0.0, 0.0]),
	)
	.unwrap();
	let connection = Builder::session()
		.unwrap()
		.serve_at("/", ObjectManager)
		.unwrap()
		.build()
		.await
		.unwrap();
	let mut grabbable = Grabbable::create(
		connection,
		"/Grabbable",
		&root,
		Transform::identity(),
		&field,
		GrabbableSettings {
			pointer_mode: PointerMode::Move,
			magnet: true,
			..Default::default()
		},
	)
	.unwrap();

	grabbable.set_debug(Some(DebugSettings::default()));
	model
		.set_spatial_parent(&grabbable.content_parent())
		.unwrap();
	field
		.set_spatial_parent(&grabbable.content_parent())
		.unwrap();

	client
		.sync_event_loop(|client, _flow| {
			grabbable.handle_events();
			if grabbable.grab_action().actor_stopped() {
				grabbable.set_pose([0.0; 3], glam::Quat::IDENTITY);
			}
			while let Some(root_event) = client.get_root().recv_root_event() {
				match root_event {
					RootEvent::Frame { info } => grabbable.frame(&info),
					RootEvent::SaveState { response } => response.send_ok({
						root.set_relative_transform(
							&grabbable.content_parent(),
							Transform::identity(),
						)
						.unwrap();
						ClientState {
							data: None,
							root: root.node().id(),
							spatial_anchors: [(
								"content_parent".to_string(),
								grabbable.content_parent().node().id(),
							)]
							.into_iter()
							.collect(),
						}
					}),
					RootEvent::Ping { response } => response.send_ok(()),
				}
			}
		})
		.await
		.unwrap()
}
