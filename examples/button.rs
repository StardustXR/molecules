#![allow(dead_code)]

use color_eyre::eyre::Result;
use manifest_dir_macros::directory_relative_path;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::{Client, ClientState, FrameInfo, RootHandler},
	core::values::Transform,
	node::NodeError,
};
use stardust_xr_molecules::{
	button::{Button, ButtonSettings},
	data::SimplePulseReceiver,
	DebugSettings, VisualDebug,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _wrapped_root = client.wrap_root(ButtonDemo::new(&client)?)?;

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

struct ButtonDemo(Button, SimplePulseReceiver<ButtonAction>);
impl ButtonDemo {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let mut button = Button::create(
			client.get_root(),
			Transform::none(),
			[0.1; 2],
			ButtonSettings::default(),
		)?;
		button.set_debug(Some(DebugSettings::default()));
		let action = SimplePulseReceiver::create(
			button.touch_plane().root(),
			Transform::none(),
			&button.touch_plane().field(),
			|_uid, data: ButtonAction| {
				if data.press {
					dbg!(data);
				}
			},
		)?;
		Ok(ButtonDemo(button, action))
	}
}
impl RootHandler for ButtonDemo {
	fn frame(&mut self, _info: FrameInfo) {
		self.0.update();
	}
	fn save_state(&mut self) -> ClientState {
		ClientState::default()
	}
}
