use futures_util::StreamExt;
use stardust_xr_fusion::types::{Color, rgba_linear};
use tokio::{sync::watch, task::AbortHandle};
use zbus::{
	Connection, Proxy,
	proxy::Builder,
	zvariant::{OwnedValue, Value},
};

const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";
const ACCENT_COLOR_KEY: &str = "accent-color";

fn extract_accent_color(value: OwnedValue) -> Option<Color> {
	// Read returns v; the inner value may itself be a v wrapping (ddd)
	let value: Value = value.into();
	let value = match value {
		Value::Value(inner) => *inner,
		other => other,
	};
	let Value::Structure(s) = value else {
		return None;
	};
	let fields = s.into_fields();
	if fields.len() != 3 {
		return None;
	}
	let r = f64::try_from(fields[0].clone()).ok()? as f32;
	let g = f64::try_from(fields[1].clone()).ok()? as f32;
	let b = f64::try_from(fields[2].clone()).ok()? as f32;
	Some(rgba_linear!(r, g, b, 1.0))
}

async fn accent_color_loop(
	dbus_connection: Connection,
	accent_color_sender: watch::Sender<Color>,
) -> zbus::Result<()> {
	let proxy: Proxy = Builder::new(&dbus_connection)
		.destination("org.freedesktop.portal.Desktop")?
		.path("/org/freedesktop/portal/desktop")?
		.interface("org.freedesktop.portal.Settings")?
		.build()
		.await?;

	let initial: OwnedValue = proxy.call("Read", &(APPEARANCE_NAMESPACE, ACCENT_COLOR_KEY)).await?;
	if let Some(color) = extract_accent_color(initial) {
		let _ = accent_color_sender.send(color);
		tracing::info!("Accent color initialized to {:?}", color);
	}

	let mut stream = proxy
		.receive_signal_with_args("SettingChanged", &[(0, APPEARANCE_NAMESPACE), (1, ACCENT_COLOR_KEY)])
		.await?;
	tracing::info!("Got accent color stream");

	while let Some(msg) = stream.next().await {
		// Signal body: (String, String, OwnedValue)
		let Ok((_, _, value)) = msg.body().deserialize::<(String, String, OwnedValue)>() else {
			continue;
		};
		if let Some(color) = extract_accent_color(value) {
			tracing::info!("Accent color changed to {:?}", color);
			let _ = accent_color_sender.send(color);
		}
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
