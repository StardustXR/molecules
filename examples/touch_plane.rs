#![allow(dead_code)]

use color_eyre::eyre::Result;
use glam::{Quat, Vec3};
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::Client,
	drawable::{Lines, LinesAspect},
	node::{NodeError, NodeType},
	root::{ClientState, FrameInfo, RootAspect, RootHandler},
	spatial::{Spatial, Transform},
};
use stardust_xr_molecules::{lines::LineExt, touch_plane::TouchPlane, DebugSettings, VisualDebug};
use std::f32::consts::PI;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")])?;

	let _wrapped_root = client
		.get_root()
		.alias()
		.wrap(TouchPlaneDemo::new(&client)?)?;

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

struct TouchPlaneDemo {
	root: Spatial,
	touch_plane: TouchPlane,
	touch_visualizer: Lines,
}

impl TouchPlaneDemo {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let root = Spatial::create(client.get_root(), Transform::identity(), true)?;
		let mut touch_plane = TouchPlane::create(
			&root,
			Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
			[0.3, 0.2],
			0.01,
			-0.15..0.15,
			0.1..-0.1,
		)?;
		touch_plane.set_debug(Some(DebugSettings::default()));

		let touch_visualizer = Lines::create(touch_plane.root(), Transform::identity(), &[])?;
		Ok(TouchPlaneDemo {
			root,
			touch_plane,
			touch_visualizer,
		})
	}

	fn update_touch_visualizer(&mut self) {
		let mut lines = Vec::new();
		for input in self.touch_plane.action().interact().current() {
			let (point, depth) = self.touch_plane.interact_point(input);
			let radius = 0.01 + depth.abs() * 0.1; // Increased multiplier for more pronounced radius change
			let circle = stardust_xr_molecules::lines::circle(16, 0.0, radius)
				.thickness(0.002)
				.transform(glam::Mat4::from_translation(Vec3::new(
					point.x, point.y, 0.0,
				)));
			lines.push(circle);
		}
		self.touch_visualizer.set_lines(&lines).unwrap();
	}
}

impl RootHandler for TouchPlaneDemo {
	fn frame(&mut self, _info: FrameInfo) {
		self.touch_plane.update();
		self.update_touch_visualizer();
	}

	fn save_state(&mut self) -> Result<ClientState> {
		Ok(ClientState::default())
	}
}
