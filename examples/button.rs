#![allow(dead_code)]

use color_eyre::eyre::Result;
use glam::Quat;
use manifest_dir_macros::directory_relative_path;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::Client,
	drawable::{Text, TextAspect, TextStyle, XAlign, YAlign},
	node::{NodeError, NodeType},
	root::{ClientState, FrameInfo, RootAspect, RootHandler},
	spatial::{Spatial, Transform},
};
use stardust_xr_molecules::{
	button::{Button, ButtonSettings},
	data::SimplePulseReceiver,
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

	let _wrapped_root = client.get_root().alias().wrap(ButtonDemo::new(&client)?)?;

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ButtonAction {
	action: (),
	button: (),
	press: bool,
}

struct ButtonDemo {
	root: Spatial,
	button: Button,
	reciever: SimplePulseReceiver<ButtonAction>,
	text: Text,
}
impl ButtonDemo {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let root = Spatial::create(client.get_root(), Transform::identity(), true)?;
		let mut button = Button::create(
			&root,
			Transform::none(),
			[0.1; 2],
			ButtonSettings::default(),
		)?;
		button.set_debug(Some(DebugSettings::default()));
		let reciever = SimplePulseReceiver::create(
			button.touch_plane().root(),
			Transform::none(),
			button.touch_plane().field(),
			|_uid, data: ButtonAction| {
				if data.press {
					dbg!(data);
				}
			},
		)?;
		let text = Text::create(
			button.touch_plane().root(),
			Transform::from_translation_rotation([0.0, -0.06, 0.0], Quat::from_rotation_y(PI)),
			"Unpressed",
			TextStyle {
				character_height: 0.01,
				text_align_x: XAlign::Center,
				text_align_y: YAlign::Top,
				..Default::default()
			},
		)?;
		Ok(ButtonDemo {
			root,
			button,
			reciever,
			text,
		})
	}
}
impl RootHandler for ButtonDemo {
	fn frame(&mut self, _info: FrameInfo) {
		self.button.update();
		if self.button.pressed() {
			self.text.set_text("Pressed").unwrap();
		}
		if self.button.released() {
			self.text.set_text("Unpressed").unwrap();
		}
	}
	fn save_state(&mut self) -> Result<ClientState> {
		ClientState::from_data_root(None::<()>, self.button.touch_plane().root())
	}
}
