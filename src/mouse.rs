use gluon::{Context, Object};
use stardust_xr_fusion::{
	client::{Client, ClientHandler},
	error::ServerError,
	fields::Field,
	query::{QueryExt, QueryableObject},
	spatial::Spatial,
	types::{Timestamp, proxies::Vec2F},
};
pub use stardust_xr_molecules_protocols::mouse::ScrollSource;
use stardust_xr_molecules_protocols::mouse::{EXTERNAL_PROTOCOL, MouseHandlerHandler};
use std::any::Any;

struct MouseCallbacks {
	on_button: Box<dyn Fn(u32, bool) + Send + Sync>,
	on_motion: Box<dyn Fn([f32; 2]) + Send + Sync>,
	on_scroll_smooth: Box<dyn Fn([f32; 2], ScrollSource) + Send + Sync>,
	on_scroll_discrete: Box<dyn Fn([f32; 2], ScrollSource) + Send + Sync>,
}

#[derive(gluon::Handler)]
struct MouseHandler(MouseCallbacks);
impl std::fmt::Debug for MouseHandler {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MouseHandler").finish()
	}
}
impl MouseHandlerHandler for MouseHandler {
	async fn motion(&self, _ctx: Context, delta: Vec2F, _ts: Option<Timestamp>) {
		(self.0.on_motion)([delta.x, delta.y]);
	}
	async fn button(&self, _ctx: Context, button: u32, pressed: bool, _ts: Option<Timestamp>) {
		(self.0.on_button)(button, pressed);
	}
	async fn scroll_smooth(
		&self,
		_ctx: Context,
		delta: Vec2F,
		source: ScrollSource,
		_ts: Option<Timestamp>,
	) {
		(self.0.on_scroll_smooth)([delta.x, delta.y], source);
	}
	async fn scroll_discrete(
		&self,
		_ctx: Context,
		delta: Vec2F,
		source: ScrollSource,
		_ts: Option<Timestamp>,
	) {
		(self.0.on_scroll_discrete)([delta.x, delta.y], source);
	}
}

pub struct Mouse {
	_obj: Object<MouseHandler>,
	_guard: Box<dyn Any + Send + Sync>,
}

impl Mouse {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		field: Field,
		on_button: impl Fn(u32, bool) + Send + Sync + 'static,
		on_motion: impl Fn([f32; 2]) + Send + Sync + 'static,
		on_scroll_smooth: impl Fn([f32; 2], ScrollSource) + Send + Sync + 'static,
		on_scroll_discrete: impl Fn([f32; 2], ScrollSource) + Send + Sync + 'static,
	) -> Result<Self, ServerError> {
		let handler = MouseHandler(MouseCallbacks {
			on_button: Box::new(on_button),
			on_motion: Box::new(on_motion),
			on_scroll_smooth: Box::new(on_scroll_smooth),
			on_scroll_discrete: Box::new(on_scroll_discrete),
		});
		let obj = client.pion_device().register_object(handler);

		let queryable = QueryableObject::create(client, spatial, field).await?;
		let guard = queryable
			.unwrap()
			.add_interface(&obj, EXTERNAL_PROTOCOL.protocol_name)
			.await?;

		Ok(Mouse {
			_obj: obj,
			_guard: Box::new(guard) as Box<dyn Any + Send + Sync>,
		})
	}
}
