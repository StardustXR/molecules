use gluon::{Context, Interface, Node, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::Field,
	query::{QueryableExt, QueryableObject},
	spatial::Spatial,
	types::{Timestamp, proxies::Vec2F},
};
pub use stardust_xr_molecules_protocols::mouse_handler::ScrollSource;
pub mod protocol {
	pub use stardust_xr_molecules_protocols::mouse_handler::*;
}
use stardust_xr_molecules_protocols::mouse_handler::{
	MouseHandler as MouseHandlerProxy, MouseHandlerHandler,
};
use std::any::Any;

#[derive(gluon::Handler)]
struct MouseHandlerInner {
	on_button: Box<dyn Fn(u32, bool, Option<Timestamp>) + Send + Sync>,
	on_motion: Box<dyn Fn(Vec2F, Option<Timestamp>) + Send + Sync>,
	on_scroll_smooth: Box<dyn Fn(Vec2F, ScrollSource, Option<Timestamp>) + Send + Sync>,
	on_scroll_discrete: Box<dyn Fn(Vec2F, ScrollSource, Option<Timestamp>) + Send + Sync>,
}
impl std::fmt::Debug for MouseHandlerInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MouseHandlerInner").finish()
	}
}
impl MouseHandlerHandler for MouseHandlerInner {
	async fn motion(&self, _ctx: Context, delta: Vec2F, ts: Option<Timestamp>) {
		(self.on_motion)(delta, ts);
	}
	async fn button(&self, _ctx: Context, button: u32, pressed: bool, ts: Option<Timestamp>) {
		(self.on_button)(button, pressed, ts);
	}
	async fn scroll_smooth(
		&self,
		_ctx: Context,
		delta: Vec2F,
		source: ScrollSource,
		ts: Option<Timestamp>,
	) {
		(self.on_scroll_smooth)(delta, source, ts);
	}
	async fn scroll_discrete(
		&self,
		_ctx: Context,
		delta: Vec2F,
		source: ScrollSource,
		ts: Option<Timestamp>,
	) {
		(self.on_scroll_discrete)(delta, source, ts);
	}
}

#[derive(Debug)]
pub struct MouseHandler {
	_node: Node<MouseHandlerInner>,
	_queryable: QueryableObject,
	_guard: Box<dyn Any + Send + Sync>,
}

impl MouseHandler {
	/// all Vec2F are +Y == Up and +X == right
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		field: Field,
		on_button: impl Fn(u32, bool, Option<Timestamp>) + Send + Sync + 'static,
		on_motion: impl Fn(Vec2F, Option<Timestamp>) + Send + Sync + 'static,
		on_scroll_smooth: impl Fn(Vec2F, ScrollSource, Option<Timestamp>) + Send + Sync + 'static,
		on_scroll_discrete: impl Fn(Vec2F, ScrollSource, Option<Timestamp>) + Send + Sync + 'static,
	) -> Result<Self> {
		let handler = MouseHandlerInner {
			on_button: Box::new(on_button),
			on_motion: Box::new(on_motion),
			on_scroll_smooth: Box::new(on_scroll_smooth),
			on_scroll_discrete: Box::new(on_scroll_discrete),
		};
		let (node, handler) = MouseHandlerProxy::new_node(handler)?;

		let queryable = QueryableObject::new(client, spatial, field).await?;
		let guard = queryable
			.add_interface(&handler, MouseHandlerProxy::ID)
			.await?;

		Ok(MouseHandler {
			_node: node,
			_queryable: queryable,
			_guard: Box::new(guard) as Box<dyn Any + Send + Sync>,
		})
	}
}
