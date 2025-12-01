use stardust_xr_fusion::{
	client::Client,
	core::values::color::rgba_linear,
	drawable::{Text, TextAspect, TextStyle, XAlign, YAlign},
	root::{ClientState, RootAspect, RootEvent},
	spatial::{Spatial, Transform},
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
	let mut client = Client::connect().await.unwrap();
	let root = Spatial::create(client.get_root(), Transform::identity()).unwrap();
	let mut hover_plane = HoverPlane::create(
		&root,
		Transform::identity(),
		[0.1, 0.1],
		0.01,
		0.0..1.0,
		0.0..1.0,
		HoverPlaneSettings::default(),
	)
	.unwrap();
	hover_plane.set_debug(Some(DebugSettings {
		line_color: rgba_linear!(0.25, 0.0, 1.0, 1.0),
		..Default::default()
	}));
	let text = Text::create(
		hover_plane.root(),
		Transform::from_translation([0.0, -0.06, 0.0]),
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
			hover_plane.update();
			if hover_plane.interact_status().actor_started() {
				text.set_text("Pressed").unwrap();
			}
			if hover_plane.interact_status().actor_stopped() {
				text.set_text("Unpressed").unwrap();
			}

			while let Some(root_event) = client.get_root().recv_root_event() {
				match root_event {
					RootEvent::Ping { response } => response.send_ok(()),
					RootEvent::SaveState { response } => response
						.wrap(|| ClientState::from_data_root(None::<()>, hover_plane.root())),
					_ => (),
				}
			}
		})
		.await
		.unwrap();
}
