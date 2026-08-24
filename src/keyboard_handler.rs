use gluon::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::Field,
	query::{QueryableExt as _, QueryableInterface, QueryableObject},
	spatial::Spatial,
	types::Timestamp,
};
pub use stardust_xr_molecules_protocols::keyboard_handler::ModifierState;
pub mod protocol {
	pub use stardust_xr_molecules_protocols::keyboard_handler::*;
}
use stardust_xr_molecules_protocols::keyboard_handler::{
	KeyEvent, KeyboardHandler as KeyboardHandlerProxy, KeyboardHandlerHandler,
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
	_node: Node<KeyboardHandlerInner>,
	_ref: KeyboardHandlerProxy,
	_queryable: QueryableObject,
	_guard: QueryableInterface,
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
		let (_node, _ref) = KeyboardHandlerProxy::new_node(handler)?;

		let queryable = QueryableObject::new(client, spatial, field).await?;
		let interface = queryable
			.add_interface(&_ref, KeyboardHandlerProxy::ID)
			.await??;

		Ok(KeyboardHandler {
			_node,
			_ref: _ref.into_proxy(),
			_queryable: queryable,
			_guard: interface,
		})
	}
}
