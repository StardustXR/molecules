use ashpd::desktop::settings::Settings;
use futures_util::StreamExt;
use stardust_xr_fusion::values::{
	Color,
	color::{rgba, rgba_linear},
};
use tokio::{sync::watch, task::AbortHandle};
use zbus::Connection;

fn accent_color_to_color(accent_color: ashpd::desktop::Color) -> Color {
	rgba!(
		accent_color.red() as f32,
		accent_color.green() as f32,
		accent_color.blue() as f32,
		1.0
	)
	.to_linear()
}

async fn accent_color_loop(
	_dbus_connection: Connection,
	accent_color_sender: watch::Sender<Color>,
) -> Result<(), ashpd::Error> {
	// let _ = ashpd::set_session_connection(dbus_connection);
	let settings = Settings::new().await?;
	let initial_color = accent_color_to_color(settings.accent_color().await?);
	let _ = accent_color_sender.send(initial_color);
	tracing::info!("Accent color initialized to {:?}", initial_color);
	let mut accent_color_stream = settings.receive_accent_color_changed().await?;
	tracing::info!("Got accent color stream");

	while let Some(accent_color) = accent_color_stream.next().await {
		let accent_color = accent_color_to_color(accent_color);
		tracing::info!("Accent color changed to {:?}", accent_color);
		let _ = accent_color_sender.send(accent_color);
	}

	tracing::error!("why the sigma is this activating");
	Ok(())
}

pub struct AccentColor {
	pub color: watch::Receiver<Color>,
	abort_handle: AbortHandle,
}
impl AccentColor {
	pub fn new(dbus_connection: Connection) -> Self {
		let (color_tx, color) = watch::channel(rgba_linear!(1.0, 1.0, 1.0, 1.0));
		let abort_handle =
			tokio::task::spawn(accent_color_loop(dbus_connection, color_tx)).abort_handle();
		Self {
			color,
			abort_handle,
		}
	}

	pub fn color(&self) -> Color {
		*self.color.borrow()
	}
}
impl Drop for AccentColor {
	fn drop(&mut self) {
		self.abort_handle.abort();
	}
}

#[tokio::test]
async fn accent_color() {
	let dbus_connection = Connection::session().await.unwrap();
	let mut accent_color = AccentColor::new(dbus_connection);

	accent_color.color.changed().await.unwrap();
	assert_ne!(accent_color.color(), rgba_linear!(1.0, 1.0, 1.0, 1.0));
	println!("Accent color is {:#?}", accent_color.color());
}
