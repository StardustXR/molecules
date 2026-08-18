use glam::{Mat4, vec3};
use stardust_xr_fusion::{
	client::Client,
	drawable::{Lines, LinesExt},
	fields::Shape,
	spatial::{Spatial, SpatialExt, Transform},
	types::rgba_linear,
};
use stardust_xr_molecules::lines::{LineExt, shape};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::connect(&[]).await.unwrap();
	let (root_spatial, _root_ref) = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();

	let mut x_offset = -0.375f32;

	let shapes: Vec<_> = vec![
		Shape::Box {
			size: [0.1, 0.1, 0.1].into(),
		},
		Shape::Cylinder {
			length: 0.2,
			radius: 0.1,
		},
		Shape::Sphere { radius: 0.1 },
		Shape::Torus {
			major_radius: 0.03,
			minor_radius: 0.1,
		},
	]
	.into_iter()
	.flat_map(|s| {
		let lines: Vec<_> = shape(s)
			.into_iter()
			.map(|l| {
				l.transform(Mat4::from_translation(vec3(x_offset, 0.0, 0.0)))
					.color(rgba_linear!(0.0, 1.0, 0.25, 0.5))
			})
			.collect();
		x_offset += 0.25;
		lines
	})
	.map(|l| l.thickness(0.005))
	.collect();

	let lines_obj = Lines::new(&client, &root_spatial, shapes.clone())
		.await
		.unwrap();

	let start = std::time::Instant::now();
	let mut frame_receiver = client.frame_receiver();
	loop {
		frame_receiver.recv().await.unwrap();

		let elapsed = start.elapsed().as_secs_f32();
		let animated: Vec<_> = shapes
			.iter()
			.cloned()
			.map(|l| {
				l.trace(elapsed).shimmer(
					&[[(elapsed * 0.5).sin(), 0.0, 0.0]],
					0.25,
					0.0,
					rgba_linear!(2.0, 2.0, 2.0, 1.0),
					0.5,
				)
			})
			.collect();
		let _ = lines_obj.set_lines(animated);
	}
}
