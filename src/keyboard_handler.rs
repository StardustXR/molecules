use gluon::{Context, Object};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::Field,
	query::{QueryExt, QueryableInterfaceGuard, QueryableObject},
	spatial::Spatial,
	types::Timestamp,
};
pub use stardust_xr_molecules_protocols::keyboard_handler::ModifierState;
pub mod protocol {
	pub use stardust_xr_molecules_protocols::keyboard_handler::*;
}
use stardust_xr_molecules_protocols::keyboard_handler::{
	EXTERNAL_PROTOCOL, KeyEvent, KeyboardHandlerHandler,
};
#[derive(gluon::Handler)]
struct KeyboardHandlerInner {
	on_key: Box<dyn Fn(KeyEvent, Option<Timestamp>) + Send + Sync>,
}
impl std::fmt::Debug for KeyboardHandlerInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("KeyboardHandlerInner").finish()
	}
}
impl KeyboardHandlerHandler for KeyboardHandlerInner {
	async fn key(&self, _ctx: Context, event: KeyEvent, timestamp: Option<Timestamp>) {
		(self.on_key)(event, timestamp);
	}
}

#[derive(Debug)]
pub struct KeyboardHandler {
	_obj: Object<KeyboardHandlerInner>,
	_queryable: QueryableObject,
	_guard: QueryableInterfaceGuard,
}
impl KeyboardHandler {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		field: Field,
		on_key: impl Fn(KeyEvent, Option<Timestamp>) + Send + Sync + 'static,
	) -> Result<Self> {
		let handler = KeyboardHandlerInner {
			on_key: Box::new(on_key),
		};
		let obj = client.pion_device().register_object(handler);

		let queryable = QueryableObject::create(client, spatial, field).await?;
		let guard = queryable
			.add_interface(&obj, EXTERNAL_PROTOCOL.protocol_name)
			.await?;

		Ok(KeyboardHandler {
			_obj: obj,
			_queryable: queryable,
			_guard: guard,
		})
	}
}
