use stardust_xr_fusion::{
	client::Client,
	drawable::{Text, TextExt, TextStyle, XAlign, YAlign},
	spatial::{Spatial, SpatialExt, Transform},
	types::rgba_linear,
};
use stardust_xr_molecules::{
	DebugSettings, VisualDebug,
	hover_plane::{HoverPlane, HoverPlaneSettings},
};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::auto_connect(&[]).await.unwrap();
	let root_spatial = Spatial::create(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();
	let root_ref = root_spatial.spatial_ref().await.unwrap();

	let mut hover_plane = HoverPlane::new(
		&client,
		&root_ref,
		Transform::IDENTITY,
		[0.1, 0.1],
		0.01,
		0.0..1.0,
		0.0..1.0,
		HoverPlaneSettings::default(),
	)
	.await
	.unwrap();
	hover_plane.set_debug(Some(DebugSettings {
		line_color: rgba_linear!(0.25, 0.0, 1.0, 1.0),
		..Default::default()
	}));

	let text = Text::create(
		&client,
		hover_plane.root(),
		"Unpinched".to_string(),
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

		hover_plane.update();
		if hover_plane.interact_status().actor_started() {
			let _ = text.set_text("Pinched");
		}
		if hover_plane.interact_status().actor_stopped() {
			let _ = text.set_text("Unpinched");
		}
	}
}
