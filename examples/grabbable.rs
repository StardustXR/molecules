use stardust_xr_fusion::{
	client::Client,
	core::values::ResourceID,
	drawable::Model,
	fields::{Field, Shape},
	node::NodeType,
	project_local_resources,
	root::{ClientState, RootAspect, RootEvent},
	spatial::{Spatial, SpatialAspect, SpatialRefAspect, Transform},
};
use stardust_xr_molecules::{
	DebugSettings, FrameSensitive, Grabbable, GrabbableSettings, PointerMode, UIElement,
	VisualDebug,
};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

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
	let field = Arc::new(
		Field::create(
			&model,
			Transform::from_translation(bounds.center),
			Shape::Box(bounds.size),
		)
		.unwrap(),
	);

	let root = Spatial::create(client.get_root(), Transform::identity(), false).unwrap();
	let mut grabbable = Grabbable::create(
		&root,
		Transform::none(),
		&field,
		GrabbableSettings {
			pointer_mode: PointerMode::Align,
			magnet: true,
			..Default::default()
		},
	)
	.unwrap();
	grabbable.set_debug(Some(DebugSettings::default()));
	model
		.set_spatial_parent(grabbable.content_parent())
		.unwrap();
	field
		.set_spatial_parent(grabbable.content_parent())
		.unwrap();

	let client_state = client
		.await_method(client.handle().get_root().get_state())
		.await
		.unwrap()
		.unwrap();

	if let Some(content_parent_reference) = client_state
		.spatial_anchors(&client.handle())
		.get("content_parent")
	{
		grabbable
			.content_parent()
			.set_relative_transform(content_parent_reference, Transform::identity())
			.unwrap();
	}

	client
		.sync_event_loop(|client, _flow| {
			grabbable.handle_events();
			match client.get_root().recv_root_event() {
				Some(RootEvent::Frame { info }) => grabbable.frame(&info),
				Some(RootEvent::SaveState { response }) => response.wrap(|| {
					root.set_relative_transform(
						grabbable.content_parent(),
						Transform::from_translation([0.0; 3]),
					)
					.unwrap();
					Ok(ClientState {
						data: None,
						root: root.node().id(),
						spatial_anchors: [(
							"content_parent".to_string(),
							grabbable.content_parent().node().id(),
						)]
						.into_iter()
						.collect(),
					})
				}),
				_ => (),
			};
		})
		.await
		.unwrap()
}
