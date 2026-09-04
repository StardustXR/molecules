#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
	protocol_name: "org.stardustxr.MouseHandler",
	types: &[gluon::ExternalGluonType {
		name: "ScrollSource",
		supported_derives: gluon::Derives::from_bits_truncate(895u32),
		proxy: None,
	}],
};
pub mod proxies {
	use super::*;
}
///The physical source type of a scroll event
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScrollSource {
	Wheel,
	Finger,
	Continuous,
	WheelTilt,
}
impl gluon::Convertable for ScrollSource {
	fn write(&self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		match self {
			ScrollSource::Wheel => {
				gluon_data.write_u16(0u16)?;
			}
			ScrollSource::Finger => {
				gluon_data.write_u16(1u16)?;
			}
			ScrollSource::Continuous => {
				gluon_data.write_u16(2u16)?;
			}
			ScrollSource::WheelTilt => {
				gluon_data.write_u16(3u16)?;
			}
		};
		Ok(())
	}
	fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
		Ok(match gluon_data.read_u16()? {
			0u16 => ScrollSource::Wheel,
			1u16 => ScrollSource::Finger,
			2u16 => ScrollSource::Continuous,
			3u16 => ScrollSource::WheelTilt,
			v => return Err(gluon::ReadError::UnknownEnumVariant(v)),
		})
	}
	fn write_owned(self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		match self {
			ScrollSource::Wheel => {
				gluon_data.write_u16(0u16)?;
			}
			ScrollSource::Finger => {
				gluon_data.write_u16(1u16)?;
			}
			ScrollSource::Continuous => {
				gluon_data.write_u16(2u16)?;
			}
			ScrollSource::WheelTilt => {
				gluon_data.write_u16(3u16)?;
			}
		};
		Ok(())
	}
}
#[derive(Debug, Clone)]
pub struct MouseHandler {
	obj: gluon::Ref,
}
impl gluon::Convertable for MouseHandler {
	fn write(&self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write(gluon_data)
	}
	fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
		let obj = gluon::Ref::read(gluon_data)?;
		Ok(MouseHandler::from_ref(obj))
	}
	fn write_owned(self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write_owned(gluon_data)
	}
}
impl MouseHandler {
	const ID: &'static str = "org.stardustxr.MouseHandler.MouseHandler";
}
impl gluon::Interface for MouseHandler {
	const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: MouseHandlerHandler> gluon::HandledBy<H> for MouseHandler {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type MouseHandlerLocal<H> = gluon::LocalRef<MouseHandler, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: MouseHandlerHandler> From<MouseHandlerLocal<H>> for MouseHandler {
	fn from(value: MouseHandlerLocal<H>) -> MouseHandler {
		value.into_proxy()
	}
}
impl gluon::RefExt for MouseHandler {
	fn from_ref(obj: gluon::Ref) -> MouseHandler {
		MouseHandler { obj }
	}
}
impl MouseHandler {
	///delta is +Y == Up +X == Right
	pub fn motion(
		&self,
		delta: stardust_xr_protocol::types::proxies::Vec2F,
		timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
	) -> Result<(), gluon::SendError> {
		let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
		let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
		tracing::trace!(
			interface = "MouseHandler",
			method = "motion",
			?delta,
			?timestamp,
			"→"
		);
		let mut gluon_builder = gluon::DataBuilder::new();
		delta.write(&mut gluon_builder)?;
		timestamp.write(&mut gluon_builder)?;
		gluon::transact(&self.obj, 8u32, gluon_builder)?;
		Ok(())
	}
	///button code from `input_event_codes.h`
	pub fn button(
		&self,
		button: impl Into<u32>,
		pressed: impl Into<bool>,
		timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
	) -> Result<(), gluon::SendError> {
		let button: u32 = button.into();
		let pressed: bool = pressed.into();
		let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
		tracing::trace!(
			interface = "MouseHandler",
			method = "button",
			?button,
			?pressed,
			?timestamp,
			"→"
		);
		let mut gluon_builder = gluon::DataBuilder::new();
		button.write(&mut gluon_builder)?;
		pressed.write(&mut gluon_builder)?;
		timestamp.write(&mut gluon_builder)?;
		gluon::transact(&self.obj, 9u32, gluon_builder)?;
		Ok(())
	}
	///delta is +Y == Up +X == Right
	pub fn scroll_smooth(
		&self,
		delta: stardust_xr_protocol::types::proxies::Vec2F,
		source: impl Into<ScrollSource>,
		timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
	) -> Result<(), gluon::SendError> {
		let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
		let source: ScrollSource = source.into();
		let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
		tracing::trace!(
			interface = "MouseHandler",
			method = "scroll_smooth",
			?delta,
			?source,
			?timestamp,
			"→"
		);
		let mut gluon_builder = gluon::DataBuilder::new();
		delta.write(&mut gluon_builder)?;
		source.write(&mut gluon_builder)?;
		timestamp.write(&mut gluon_builder)?;
		gluon::transact(&self.obj, 10u32, gluon_builder)?;
		Ok(())
	}
	///delta is +Y == Up +X == Right
	pub fn scroll_discrete(
		&self,
		delta: stardust_xr_protocol::types::proxies::Vec2F,
		source: impl Into<ScrollSource>,
		timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
	) -> Result<(), gluon::SendError> {
		let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
		let source: ScrollSource = source.into();
		let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
		tracing::trace!(
			interface = "MouseHandler",
			method = "scroll_discrete",
			?delta,
			?source,
			?timestamp,
			"→"
		);
		let mut gluon_builder = gluon::DataBuilder::new();
		delta.write(&mut gluon_builder)?;
		source.write(&mut gluon_builder)?;
		timestamp.write(&mut gluon_builder)?;
		gluon::transact(&self.obj, 11u32, gluon_builder)?;
		Ok(())
	}
	///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
	pub fn from_ref(obj: gluon::Ref) -> MouseHandler {
		MouseHandler { obj }
	}
}
impl From<MouseHandler> for gluon::Ref {
	fn from(value: MouseHandler) -> Self {
		value.obj
	}
}
impl gluon::ToRef for MouseHandler {
	fn to_ref(&self) -> gluon::Ref {
		self.obj.clone()
	}
}
impl gluon::Liveness for MouseHandler {
	fn death_notifier(&self) -> gluon::DeathNotifier {
		gluon::Liveness::death_notifier(&self.obj)
	}
}
impl std::hash::Hash for MouseHandler {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.obj.hash(state);
	}
}
impl PartialEq for MouseHandler {
	fn eq(&self, other: &Self) -> bool {
		self.obj == other.obj
	}
}
impl Eq for MouseHandler {}
pub trait MouseHandlerHandler: gluon::Handler + Send + Sync + 'static {
	///delta is +Y == Up +X == Right
	fn motion(
		&self,
		_ctx: gluon::Context,
		delta: stardust_xr_protocol::types::proxies::Vec2F,
		timestamp: Option<stardust_xr_protocol::types::Timestamp>,
	) -> impl Future<Output = ()> + Send + Sync;
	///button code from `input_event_codes.h`
	fn button(
		&self,
		_ctx: gluon::Context,
		button: u32,
		pressed: bool,
		timestamp: Option<stardust_xr_protocol::types::Timestamp>,
	) -> impl Future<Output = ()> + Send + Sync;
	///delta is +Y == Up +X == Right
	fn scroll_smooth(
		&self,
		_ctx: gluon::Context,
		delta: stardust_xr_protocol::types::proxies::Vec2F,
		source: ScrollSource,
		timestamp: Option<stardust_xr_protocol::types::Timestamp>,
	) -> impl Future<Output = ()> + Send + Sync;
	///delta is +Y == Up +X == Right
	fn scroll_discrete(
		&self,
		_ctx: gluon::Context,
		delta: stardust_xr_protocol::types::proxies::Vec2F,
		source: ScrollSource,
		timestamp: Option<stardust_xr_protocol::types::Timestamp>,
	) -> impl Future<Output = ()> + Send + Sync;
	fn dispatch_one_way(
		&self,
		transaction_code: u32,
		mut gluon_data: gluon::DataReader,
		ctx: gluon::Context,
	) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
		async move {
			match transaction_code {
				8u32 => {
					let __wire_param_delta: stardust_xr_protocol::types::proxied::Vec2F =
						gluon::Convertable::read(&mut gluon_data)?;
					let param_timestamp = gluon::Convertable::read(&mut gluon_data)?;
					tracing::trace!(
						interface = "MouseHandler", method = "motion", param_delta = ?
						__wire_param_delta, ? param_timestamp, "dispatching"
					);
					let param_delta: stardust_xr_protocol::types::proxies::Vec2F = {
						let __w = __wire_param_delta;
						__w.into()
					};
					drop(gluon_data);
					self.motion(ctx, param_delta, param_timestamp)
						.instrument(tracing::trace_span!(
							"dispatching",
							interface = "MouseHandler",
							method = "motion",
							method_id = 8u32
						))
						.await;
				}
				9u32 => {
					let param_button = gluon::Convertable::read(&mut gluon_data)?;
					let param_pressed = gluon::Convertable::read(&mut gluon_data)?;
					let param_timestamp = gluon::Convertable::read(&mut gluon_data)?;
					tracing::trace!(
						interface = "MouseHandler",
						method = "button",
						?param_button,
						?param_pressed,
						?param_timestamp,
						"dispatching"
					);
					drop(gluon_data);
					self.button(ctx, param_button, param_pressed, param_timestamp)
						.instrument(tracing::trace_span!(
							"dispatching",
							interface = "MouseHandler",
							method = "button",
							method_id = 9u32
						))
						.await;
				}
				10u32 => {
					let __wire_param_delta: stardust_xr_protocol::types::proxied::Vec2F =
						gluon::Convertable::read(&mut gluon_data)?;
					let param_source = gluon::Convertable::read(&mut gluon_data)?;
					let param_timestamp = gluon::Convertable::read(&mut gluon_data)?;
					tracing::trace!(
						interface = "MouseHandler", method = "scroll_smooth", param_delta
						= ? __wire_param_delta, ? param_source, ? param_timestamp,
						"dispatching"
					);
					let param_delta: stardust_xr_protocol::types::proxies::Vec2F = {
						let __w = __wire_param_delta;
						__w.into()
					};
					drop(gluon_data);
					self.scroll_smooth(ctx, param_delta, param_source, param_timestamp)
						.instrument(tracing::trace_span!(
							"dispatching",
							interface = "MouseHandler",
							method = "scroll_smooth",
							method_id = 10u32
						))
						.await;
				}
				11u32 => {
					let __wire_param_delta: stardust_xr_protocol::types::proxied::Vec2F =
						gluon::Convertable::read(&mut gluon_data)?;
					let param_source = gluon::Convertable::read(&mut gluon_data)?;
					let param_timestamp = gluon::Convertable::read(&mut gluon_data)?;
					tracing::trace!(
						interface = "MouseHandler", method = "scroll_discrete",
						param_delta = ? __wire_param_delta, ? param_source, ?
						param_timestamp, "dispatching"
					);
					let param_delta: stardust_xr_protocol::types::proxies::Vec2F = {
						let __w = __wire_param_delta;
						__w.into()
					};
					drop(gluon_data);
					self.scroll_discrete(ctx, param_delta, param_source, param_timestamp)
						.instrument(tracing::trace_span!(
							"dispatching",
							interface = "MouseHandler",
							method = "scroll_discrete",
							method_id = 11u32
						))
						.await;
				}
				_ => {}
			}
			Ok(())
		}
	}
	fn to_node(
		self,
	) -> Result<(gluon::Node<Self>, gluon::LocalRef<MouseHandler, Self>), gluon::NodeError>
	where
		Self: Sized,
	{
		use gluon::RefExt;
		MouseHandler::new_node(self)
	}
	fn to_service(self) -> Result<gluon::LocalRef<MouseHandler, Self>, gluon::NodeError>
	where
		Self: Sized,
	{
		use gluon::RefExt;
		MouseHandler::new_service(self)
	}
}
pub mod proxied {
	use super::*;
}
