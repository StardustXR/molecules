use stardust_xr_fusion::{
	client::Client,
	fields::{Field, FieldExt, Shape},
	spatial::{Spatial, SpatialExt, Transform},
};
use stardust_xr_molecules::{
	DebugSettings, FrameSensitive, UIElement, VisualDebug,
	grabbable::{Grabbable, GrabbableSettings},
};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::auto_connect(&[]).await.unwrap();
	let root_spatial = Spatial::create(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();
	let root_ref = root_spatial.spatial_ref().await.unwrap();

	let field_spatial = Spatial::create(&client, &root_ref, Transform::IDENTITY)
		.await
		.unwrap();
	let field = Field::create(
		&client,
		&field_spatial,
		Shape::Box {
			size: [0.1, 0.1, 0.1].into(),
		},
	)
	.await
	.unwrap();

	let mut grabbable = Grabbable::new(
		&client,
		root_ref,
		Transform::IDENTITY,
		field,
		GrabbableSettings::default(),
	)
	.await
	.unwrap();
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
