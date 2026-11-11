use gluon_ipc::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	query::QueryableInterface,
	types::{Timestamp, proxies::Vec2F},
};
pub use stardust_xr_molecules_protocols::mouse_handler::ScrollSource;
pub mod protocol {
	pub use stardust_xr_molecules_protocols::mouse_handler::*;
}
use stardust_xr_molecules_protocols::mouse_handler::{
	MouseHandler as MouseHandlerProxy, MouseHandlerHandler,
};

use crate::{entity::Component, event::Event};

#[derive(gluon_ipc::Handler)]
struct MouseHandlerInner {
	on_button: Event<(u32, bool, Option<Timestamp>)>,
	on_motion: Event<(Vec2F, Option<Timestamp>)>,
	on_scroll_smooth: Event<(Vec2F, ScrollSource, Option<Timestamp>)>,
	on_scroll_discrete: Event<(Vec2F, ScrollSource, Option<Timestamp>)>,
}
impl std::fmt::Debug for MouseHandlerInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MouseHandlerInner").finish()
	}
}
impl MouseHandlerHandler for MouseHandlerInner {
	async fn motion(&self, _ctx: Context, delta: Vec2F, ts: Option<Timestamp>) {
		self.on_motion.send((delta, ts)).await;
	}
	async fn button(&self, _ctx: Context, button: u32, pressed: bool, ts: Option<Timestamp>) {
		self.on_button.send((button, pressed, ts)).await;
	}
	async fn scroll_smooth(
		&self,
		_ctx: Context,
		delta: Vec2F,
		source: ScrollSource,
		ts: Option<Timestamp>,
	) {
		self.on_scroll_smooth.send((delta, source, ts)).await;
	}
	async fn scroll_discrete(
		&self,
		_ctx: Context,
		delta: Vec2F,
		source: ScrollSource,
		ts: Option<Timestamp>,
	) {
		self.on_scroll_discrete.send((delta, source, ts)).await;
	}
}

#[derive(Debug)]
pub struct MouseHandler {
	_node: Node<MouseHandlerInner>,
	_guard: QueryableInterface,
}

pub struct MouseHandlerArgs {
	on_button: Event<(u32, bool, Option<Timestamp>)>,
	on_motion: Event<(Vec2F, Option<Timestamp>)>,
	on_scroll_smooth: Event<(Vec2F, ScrollSource, Option<Timestamp>)>,
	on_scroll_discrete: Event<(Vec2F, ScrollSource, Option<Timestamp>)>,
}
impl MouseHandlerArgs {
	/// all Vec2F are +Y == Up and +X == right
	pub fn new(
		on_button: impl Into<Event<(u32, bool, Option<Timestamp>)>>,
		on_motion: impl Into<Event<(Vec2F, Option<Timestamp>)>>,
		on_scroll_smooth: impl Into<Event<(Vec2F, ScrollSource, Option<Timestamp>)>>,
		on_scroll_discrete: impl Into<Event<(Vec2F, ScrollSource, Option<Timestamp>)>>,
	) -> Self {
		Self {
			on_button: on_button.into(),
			on_motion: on_motion.into(),
			on_scroll_smooth: on_scroll_smooth.into(),
			on_scroll_discrete: on_scroll_discrete.into(),
		}
	}
}
impl Component for MouseHandler {
	type Args = MouseHandlerArgs;

	type Err = stardust_xr_fusion::Error;

	async fn new_component(
		ctx: &crate::entity::ComponentContext,
		_client: &Client<impl ClientHandler>,
		args: impl Into<Self::Args> + Send,
	) -> Result<Self> {
		let MouseHandlerArgs {
			on_button,
			on_motion,
			on_scroll_smooth,
			on_scroll_discrete,
		} = args.into();
		let handler = MouseHandlerInner {
			on_button,
			on_motion,
			on_scroll_smooth,
			on_scroll_discrete,
		};
		let (node, handler) = MouseHandlerProxy::new_node(handler)?;

		let guard = ctx
			.queryable
			.add_interface(&handler, MouseHandlerProxy::ID)
			.await??;

		Ok(MouseHandler {
			_node: node,
			_guard: guard,
		})
	}

	fn handle_events(&mut self) -> bool {
		false
	}

	fn frame(&mut self, _info: &stardust_xr_fusion::client::FrameInfo) {}
}
