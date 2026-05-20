use stardust_xr_fusion::{
	client::Client,
	fields::{Field, FieldExt, Shape},
	spatial::{Spatial, SpatialExt, Transform},
};
use stardust_xr_molecules::input_action::InputQueue;
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt::init();
	let (client, root) = Client::auto_connect(&[]).await.unwrap();

	let root_spatial = Spatial::create(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();
	let root_ref = root_spatial.spatial_ref().await.unwrap();

	let field = Field::create(&client, &root_spatial, Shape::Sphere { radius: 0.1 })
		.await
		.unwrap();

	let input_queue = InputQueue::new(&client, root_spatial, field, root_ref)
		.await
		.unwrap();

	let mut frame_recv = client.frame_receiver();
	loop {
		match frame_recv.recv().await {
			Ok(_) => {}
			Err(RecvError::Lagged(n)) => {
				warn!("lost {n} frame events");
				continue;
			}
			Err(RecvError::Closed) => break,
		}

		if input_queue.handle_events() {
			let snapshots = input_queue.input();
			println!("--- frame ({} input methods) ---", snapshots.len());
			for snap in snapshots.values() {
				println!("{snap:#?}");
			}
		}
	}
}
