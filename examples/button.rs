use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::Client,
	drawable::{Text, TextExt, TextStyle, XAlign, YAlign},
	spatial::{Spatial, SpatialExt, Transform},
};
use stardust_xr_molecules::{
	DebugSettings, UIElement, VisualDebug,
	button::{Button, ButtonSettings},
};
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
	let (mut client, root) = Client::auto_connect(&[]).await.unwrap();

	let root = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();
	let mut button = Button::create(
		&root,
		Transform::IDENTITY,
		[0.1; 2],
		ButtonSettings::default(),
	)
	.unwrap();
	button.set_debug(Some(DebugSettings::default()));

	let text_spatial = Spatial::new(
		&client,
		&root.spatial_ref().await.unwrap(),
		Transform::from_translation([0.0, -0.06, 0.0]),
	)
	.await
	.unwrap();
	let text = Text::new(
		&client,
		&text_spatial,
		"Unpressed",
		TextStyle {
			character_height: 0.01,
			text_align_x: XAlign::Center,
			text_align_y: YAlign::Top,
			..Default::default()
		},
	)
	.await
	.unwrap();

	let mut frame_receiver = client.frame_receiver();
	loop {
		let frame_info = frame_receiver.recv().await.unwrap();

		button.handle_events();
		if button.pressed() {
			text.set_text("Pressed").unwrap();
		}
		if button.released() {
			text.set_text("Unpressed").unwrap();
		}
	}
}
