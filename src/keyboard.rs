use gluon::{Context, Object};
use stardust_xr_fusion::{
	Result, client::{Client, ClientHandler}, fields::Field, keymap::Keymap, query::{QueryExt, QueryableObject}, spatial::Spatial, types::Timestamp
};
pub use stardust_xr_molecules_protocols::keyboard::ModifierState;
use stardust_xr_molecules_protocols::keyboard::{EXTERNAL_PROTOCOL, KeyboardHandlerHandler};
use std::any::Any;

#[derive(Clone)]
pub struct KeyInfo {
	pub keycode: u32,
	pub pressed: bool,
	pub modifiers: ModifierState,
	pub keymap: Keymap,
}

struct KeyboardInner {
	on_key: Box<dyn Fn(KeyInfo) + Send + Sync>,
}

#[derive(gluon::Handler)]
struct KeyboardHandler(KeyboardInner);
impl std::fmt::Debug for KeyboardHandler {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("KeyboardHandler").finish()
	}
}
impl KeyboardHandlerHandler for KeyboardHandler {
	async fn key_state(
		&self,
		_ctx: Context,
		keycode: u32,
		pressed: bool,
		_ts: Option<Timestamp>,
		modifiers: ModifierState,
		keymap: Keymap,
	) {
		(self.0.on_key)(KeyInfo {
			keycode,
			pressed,
			modifiers,
			keymap,
		});
	}
}

pub struct Keyboard {
	_obj: Object<KeyboardHandler>,
	_guard: Box<dyn Any + Send + Sync>,
}
impl Keyboard {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		field: Field,
		on_key: impl Fn(KeyInfo) + Send + Sync + 'static,
	) -> Result<Self> {
		let handler = KeyboardHandler(KeyboardInner {
			on_key: Box::new(on_key),
		});
		let obj = client.pion_device().register_object(handler);

		let queryable = QueryableObject::create(client, spatial, field).await?;
		let guard = queryable
			.add_interface(&obj, EXTERNAL_PROTOCOL.protocol_name)
			.await?;

		Ok(Keyboard {
			_obj: obj,
			_guard: Box::new(guard) as Box<dyn Any + Send + Sync>,
		})
	}
}
