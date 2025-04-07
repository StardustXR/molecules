use glam::Quat;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::Client,
	drawable::{Text, TextAspect, TextStyle, XAlign, YAlign},
	root::{ClientState, RootAspect, RootEvent},
	spatial::{Spatial, Transform},
};
use stardust_xr_molecules::{
	button::{Button, ButtonSettings},
	DebugSettings, UIElement, VisualDebug,
};
use std::f32::consts::PI;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ButtonAction {
	action: (),
	button: (),
	press: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let mut client = Client::connect().await.unwrap();

	let root = Spatial::create(client.get_root(), Transform::identity(), true).unwrap();
	let mut button = Button::create(
		&root,
		Transform::none(),
		[0.1; 2],
		ButtonSettings::default(),
	)
	.unwrap();
	button.set_debug(Some(DebugSettings::default()));

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
	)
	.unwrap();
	client
		.sync_event_loop(|client, _flow| {
			while let Some(root_event) = client.get_root().recv_root_event() {
				match root_event {
					RootEvent::Ping { response } => response.send(Ok(())),
					RootEvent::SaveState { response } => response.wrap(|| {
						ClientState::from_data_root(None::<()>, button.touch_plane().root())
					}),
					_ => (),
				}
			}

			button.handle_events();
			if button.pressed() {
				text.set_text("Pressed").unwrap();
			}
			if button.released() {
				text.set_text("Unpressed").unwrap();
			}
		})
		.await
		.unwrap()
}
