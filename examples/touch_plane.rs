use glam::{Quat, Vec3};
use stardust_xr_fusion::{
	client::Client,
	drawable::{Lines, LinesAspect},
	spatial::{Spatial, Transform},
};
use stardust_xr_molecules::{
	lines::LineExt, touch_plane::TouchPlane, DebugSettings, UIElement, VisualDebug,
};
use std::f32::consts::PI;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let mut client = Client::connect().await.unwrap();

	let root = Spatial::create(client.get_root(), Transform::identity(), true).unwrap();
	let mut touch_plane = TouchPlane::create(
		&root,
		Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
		[0.3, 0.2],
		0.01,
		-0.15..0.15,
		0.1..-0.1,
	)
	.unwrap();
	touch_plane.set_debug(Some(DebugSettings::default()));

	let touch_visualizer = Lines::create(touch_plane.root(), Transform::identity(), &[]).unwrap();

	client
		.event_loop(|_client, _flow| {
			if touch_plane.handle_events() {
				let mut lines = Vec::new();
				for input in touch_plane.action().interact().current() {
					let (point, depth) = touch_plane.interact_point(input);
					let radius = 0.01 + depth.abs() * 0.1; // Increased multiplier for more pronounced radius change
					let circle = stardust_xr_molecules::lines::circle(16, 0.0, radius)
						.thickness(0.002)
						.transform(glam::Mat4::from_translation(Vec3::new(
							point.x, point.y, 0.0,
						)));
					lines.push(circle);
				}
				touch_visualizer.set_lines(&lines).unwrap();
			}
		})
		.await
		.unwrap()
}
