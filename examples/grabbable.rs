use glam::Mat4;
use stardust_xr_fusion::{
	client::Client,
	drawable::{Model, ModelExt},
	fields::{Field, FieldExt, Shape},
	project_local_resources,
	spatial::{Spatial, SpatialExt, Transform},
	types::Resource,
};
use stardust_xr_molecules::{
	DebugSettings, FrameSensitive, UIElement, VisualDebug,
	grabbable::{Grabbable, GrabbableSettings, PointerMode},
};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::auto_connect(&[&project_local_resources!("res")])
		.await
		.unwrap();
	let root_spatial = Spatial::create(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();
	let root_ref = root_spatial.spatial_ref().await.unwrap();

	let content_spatial = Spatial::create(&client, &root_ref, Transform::IDENTITY)
		.await
		.unwrap();

	let _model = Model::create(
		&client,
		&content_spatial,
		Resource::Namespaced {
			namespace: "molecules".to_string(),
			path: "grabbable".to_string(),
		},
	)
	.await
	.unwrap();

	let bounds = content_spatial.get_local_bounding_box().await.unwrap();

	let field = Field::create(
		&client,
		&content_spatial,
		Shape::Transform {
			shape: Box::new(Shape::Box {
				size: bounds.extents,
			}),
			transform: Mat4::from_translation(bounds.center.into()).into(),
		},
	)
	.await
	.unwrap();

	let mut grabbable = Grabbable::new(
		&client,
		root_ref,
		Transform::IDENTITY,
		field,
		GrabbableSettings {
			pointer_mode: PointerMode::Move,
			..Default::default()
		},
	)
	.await
	.unwrap();
	let content_parent = grabbable.content_parent().spatial_ref().await.unwrap();
	content_spatial.set_parent(content_parent).unwrap();
	grabbable.set_debug(Some(DebugSettings::default()));

	let mut frame_receiver = client.frame_receiver();
	loop {
		let info = frame_receiver.recv().await.unwrap();

		grabbable.handle_events();
		if grabbable.grab_action().actor_stopped() {
			grabbable.set_pose(glam::Vec3::ZERO, glam::Quat::IDENTITY);
		}
		grabbable.frame(&info);
	}
}
