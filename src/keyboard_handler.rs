use gluon_ipc::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	query::QueryableInterface,
	types::Timestamp,
};
pub use stardust_xr_molecules_protocols::keyboard_handler::ModifierState;
pub mod protocol {
	pub use stardust_xr_molecules_protocols::keyboard_handler::*;
}
use stardust_xr_molecules_protocols::keyboard_handler::{
	KeyEvent, KeyboardHandler as KeyboardHandlerProxy, KeyboardHandlerHandler,
};

use crate::{entity::Component, event::Event};
#[derive(gluon_ipc::Handler)]
struct KeyboardHandlerInner {
	on_key: Event<(KeyEvent, Option<Timestamp>)>,
}
impl std::fmt::Debug for KeyboardHandlerInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("KeyboardHandlerInner").finish()
	}
}
impl KeyboardHandlerHandler for KeyboardHandlerInner {
	async fn key(&self, _ctx: Context, event: KeyEvent, timestamp: Option<Timestamp>) {
		self.on_key.send((event, timestamp)).await;
	}
}

#[derive(Debug)]
pub struct KeyboardHandler {
	_node: Node<KeyboardHandlerInner>,
	_ref: KeyboardHandlerProxy,
	_guard: QueryableInterface,
}
impl Component for KeyboardHandler {
	type Args = Event<(KeyEvent, Option<Timestamp>)>;

	type Err = stardust_xr_fusion::Error;

	async fn new_component(
		ctx: &crate::entity::ComponentContext,
		_client: &Client<impl ClientHandler>,
		on_key: impl Into<Self::Args>,
	) -> Result<Self> {
		let handler = KeyboardHandlerInner {
			on_key: on_key.into(),
		};
		let (_node, _ref) = KeyboardHandlerProxy::new_node(handler)?;

		let interface = ctx
			.queryable
			.add_interface(&_ref, KeyboardHandlerProxy::ID)
			.await??;

		Ok(KeyboardHandler {
			_node,
			_ref: _ref.into_proxy(),
			_guard: interface,
		})
	}

	fn handle_events(&mut self) -> bool {
		false
	}

	fn frame(&mut self, _info: &stardust_xr_fusion::client::FrameInfo) {}
}
