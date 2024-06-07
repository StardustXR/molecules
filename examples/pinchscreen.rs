#![allow(dead_code)]

use color_eyre::eyre::Result;
use glam::Quat;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::Client,
	core::values::color::rgba_linear,
	drawable::{Text, TextAspect, TextStyle, XAlign, YAlign},
	node::{NodeError, NodeType},
	root::{ClientState, FrameInfo, RootAspect, RootHandler},
	spatial::{Spatial, Transform},
};
use stardust_xr_molecules::{
	hover_plane::{HoverPlane, HoverPlaneSettings},
	DebugSettings, VisualDebug,
};
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

	let _wrapped_root = client.get_root().alias().wrap(Pinchscreen::new(&client)?)?;

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

struct Pinchscreen {
	root: Spatial,
	hover_plane: HoverPlane,
	text: Text,
}
impl Pinchscreen {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let root = Spatial::create(client.get_root(), Transform::identity(), true)?;
		let mut hover_plane = HoverPlane::create(
			&root,
			Transform::identity(),
			[0.1, 0.1],
			0.01,
			0.0..1.0,
			0.0..1.0,
			HoverPlaneSettings::default(),
		)?;
		hover_plane.set_debug(Some(DebugSettings {
			line_color: rgba_linear!(0.25, 0.0, 1.0, 1.0),
			..Default::default()
		}));
		let text = Text::create(
			hover_plane.root(),
			Transform::from_translation_rotation([0.0, -0.06, 0.0], Quat::from_rotation_y(PI)),
			"Unpressed",
			TextStyle {
				character_height: 0.01,
				text_align_x: XAlign::Center,
				text_align_y: YAlign::Top,
				..Default::default()
			},
		)?;
		Ok(Pinchscreen {
			root,
			hover_plane,
			text,
		})
	}
}
impl RootHandler for Pinchscreen {
	fn frame(&mut self, _info: FrameInfo) {
		self.hover_plane.update();
		if self.hover_plane.interact_status().actor_started() {
			self.text.set_text("Pressed").unwrap();
		}
		if self.hover_plane.interact_status().actor_stopped() {
			self.text.set_text("Unpressed").unwrap();
		}
	}
	fn save_state(&mut self) -> Result<ClientState> {
		Ok(ClientState::default())
	}
}
