use glam::Vec3;
use stardust_xr_fusion::{
	client::Client,
	drawable::{Lines, LinesExt},
	spatial::{Spatial, SpatialExt, Transform},
};
use stardust_xr_molecules::{
	DebugSettings, UIElement, VisualDebug, lines::LineExt, touch_plane::TouchPlane,
};
use std::f32::consts::FRAC_PI_2;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::connect(&[]).await.unwrap();
	let (_root_spatial, root_ref) = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();

	let mut touch_plane = TouchPlane::new(
		&client,
		&root_ref,
		Transform::from_rotation(glam::Quat::from_rotation_x(-std::f32::consts::PI / 4.0)),
		[0.3, 0.2].into(),
		0.01,
		-0.15..0.15,
		0.1..-0.1,
	)
	.await
	.unwrap();
	touch_plane.set_debug(Some(DebugSettings::default()));

	let touch_visualizer = Lines::new(&client, touch_plane.root(), vec![])
		.await
		.unwrap();

	let mut frame_receiver = client.frame_receiver();
	loop {
		frame_receiver.recv().await.unwrap();

		if touch_plane.handle_events() {
			let mut lines = Vec::new();
			for input in touch_plane.action().interact().current() {
				let (point, depth) = touch_plane.interact_point(input);
				let radius = 0.01 + depth.abs() * 0.1;
				let circle = stardust_xr_molecules::lines::circle(16, 0.0, radius)
					.thickness(0.002)
					.transform(
						glam::Mat4::from_translation(Vec3::new(point[0], point[1], 0.0))
							* glam::Mat4::from_rotation_x(FRAC_PI_2),
					);
				lines.push(circle);
			}
			let _ = touch_visualizer.set_lines(lines);
		}
	}
}
