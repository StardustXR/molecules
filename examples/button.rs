use stardust_xr_fusion::{
	client::Client,
	drawable::{Text, TextExt, TextStyle, XAlign, YAlign},
	spatial::{Spatial, SpatialExt, Transform},
	types::rgba_linear,
};
use stardust_xr_molecules::{
	DebugSettings, UIElement, VisualDebug,
	button::{Button, ButtonSettings},
};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::auto_connect(&[]).await.unwrap();

	let (_root_spatial, root_ref) = Spatial::create(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();

	let mut button = Button::new(
		&client,
		&root_ref,
		Transform::IDENTITY,
		[0.1; 2].into(),
		ButtonSettings::default(),
	)
	.await
	.unwrap();
	button.set_debug(Some(DebugSettings::default()));

	let (text_spatial, _) = Spatial::create(
		&client,
		&root_ref,
		Transform::from_translation([0.0, -0.06, 0.0]),
	)
	.await
	.unwrap();
	let text = Text::create(
		&client,
		&text_spatial,
		"Unpressed".to_string(),
		TextStyle {
			character_height: 0.01,
			color: rgba_linear!(1.0, 1.0, 1.0, 1.0),
			text_align_x: XAlign::Center,
			text_align_y: YAlign::Top,
			font: None,
			bounds: None,
		},
	)
	.await
	.unwrap();

	let mut frame_receiver = client.frame_receiver();
	loop {
		frame_receiver.recv().await.unwrap();

		button.handle_events();
		if button.pressed() {
			let _ = text.set_text("Pressed");
		}
		if button.released() {
			let _ = text.set_text("Unpressed");
		}
	}
}
