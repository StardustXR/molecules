use gluon_ipc::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	query::QueryableInterface,
};
use stardust_xr_molecules_protocols::derezzable::{
	Derezzable as DerezzableProxy, DerezzableHandler,
};
pub mod protocol {
	pub use stardust_xr_molecules_protocols::derezzable::*;
}
use tokio::sync::mpsc;

use crate::entity::{Component, ComponentContext};

#[derive(Debug, gluon_ipc::Handler)]
struct DerezzableInner(mpsc::Sender<()>);
impl DerezzableHandler for DerezzableInner {
	async fn derez(&self, _ctx: Context) {
		let _ = self.0.send(()).await;
	}
}

// TODO: refactor this to take in an event maybe?
pub struct Derezzable {
	pub receiver: mpsc::Receiver<()>,
	_node: Node<DerezzableInner>,
	_guard: QueryableInterface,
}
impl Component for Derezzable {
	type Args = ();

	type Err = stardust_xr_fusion::Error;

	async fn new_component(
		ctx: &ComponentContext,
		_client: &Client<impl ClientHandler>,
		_args: impl Into<Self::Args>,
	) -> Result<Self> {
		let (tx, rx) = mpsc::channel(8);
		let (node, derezzable) = DerezzableProxy::new_node(DerezzableInner(tx))?;

		let guard = ctx
			.queryable
			.add_interface(&derezzable, DerezzableProxy::ID)
			.await??;

		Ok(Derezzable {
			receiver: rx,
			_node: node,
			_guard: guard,
		})
	}

	fn handle_events(&mut self) -> bool {
		false
	}

	fn frame(&mut self, _info: &stardust_xr_fusion::client::FrameInfo) {}
}
