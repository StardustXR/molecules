#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
    protocol_name: "org.stardustxr.MouseHandler",
    types: &[
        gluon::ExternalGluonType {
            name: "ScrollSource",
            supported_derives: gluon::Derives::from_bits_truncate(895u32),
            proxy: None,
        },
    ],
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
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
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
        Ok(
            match gluon_data.read_u16()? {
                0u16 => ScrollSource::Wheel,
                1u16 => ScrollSource::Finger,
                2u16 => ScrollSource::Continuous,
                3u16 => ScrollSource::WheelTilt,
                v => return Err(gluon::ReadError::UnknownEnumVariant(v)),
            },
        )
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
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
    obj: gluon::ObjectOrRef,
}
impl gluon::Convertable for MouseHandler {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::ObjectOrRef::read(gluon_data)?;
        Ok(MouseHandler::from_object_or_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for MouseHandler {
    const ID: &'static str = "org.stardustxr.MouseHandler.MouseHandler";
}
impl MouseHandler {
    ///delta is +Y == Up +X == Right
    pub fn motion(
        &self,
        delta: stardust_xr_protocol::types::proxies::Vec2F,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> gluon::OnewayFuture {
        use gluon::ToObjectOrRef as _;
        let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "motion", ? delta, ? timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        let gluon_ret: Option<gluon::ObjectOrRef> = Some(
            gluon_ret.to_binder_object_or_ref(),
        );
        if let Err(err) = gluon_ret.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = delta.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = timestamp.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = self
            .obj
            .device()
            .transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())
        {
            return err.into();
        }
        gluon_recv.into()
    }
    ///delta is +Y == Up +X == Right
    ///Fire and Forget, events sent to different objects may not be handled in order
    pub fn motion_event(
        &self,
        delta: stardust_xr_protocol::types::proxies::Vec2F,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> Result<(), gluon::SendError> {
        let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "motion", ? delta, ? timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let gluon_ret: Option<gluon::ObjectOrRef> = None;
        gluon_ret.write(&mut gluon_builder)?;
        delta.write(&mut gluon_builder)?;
        timestamp.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        Ok(())
    }
    ///button code from `input_event_codes.h`
    pub fn button(
        &self,
        button: impl Into<u32>,
        pressed: impl Into<bool>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> gluon::OnewayFuture {
        use gluon::ToObjectOrRef as _;
        let button: u32 = button.into();
        let pressed: bool = pressed.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "button", ? button, ? pressed, ?
            timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        let gluon_ret: Option<gluon::ObjectOrRef> = Some(
            gluon_ret.to_binder_object_or_ref(),
        );
        if let Err(err) = gluon_ret.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = button.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = pressed.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = timestamp.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = self
            .obj
            .device()
            .transact_one_way(&self.obj, 9u32, gluon_builder.to_payload())
        {
            return err.into();
        }
        gluon_recv.into()
    }
    ///button code from `input_event_codes.h`
    ///Fire and Forget, events sent to different objects may not be handled in order
    pub fn button_event(
        &self,
        button: impl Into<u32>,
        pressed: impl Into<bool>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> Result<(), gluon::SendError> {
        let button: u32 = button.into();
        let pressed: bool = pressed.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "button", ? button, ? pressed, ?
            timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let gluon_ret: Option<gluon::ObjectOrRef> = None;
        gluon_ret.write(&mut gluon_builder)?;
        button.write(&mut gluon_builder)?;
        pressed.write(&mut gluon_builder)?;
        timestamp.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 9u32, gluon_builder.to_payload())?;
        Ok(())
    }
    ///delta is +Y == Up +X == Right
    pub fn scroll_smooth(
        &self,
        delta: stardust_xr_protocol::types::proxies::Vec2F,
        source: impl Into<ScrollSource>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> gluon::OnewayFuture {
        use gluon::ToObjectOrRef as _;
        let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
        let source: ScrollSource = source.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "scroll_smooth", ? delta, ? source, ?
            timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        let gluon_ret: Option<gluon::ObjectOrRef> = Some(
            gluon_ret.to_binder_object_or_ref(),
        );
        if let Err(err) = gluon_ret.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = delta.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = source.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = timestamp.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = self
            .obj
            .device()
            .transact_one_way(&self.obj, 10u32, gluon_builder.to_payload())
        {
            return err.into();
        }
        gluon_recv.into()
    }
    ///delta is +Y == Up +X == Right
    ///Fire and Forget, events sent to different objects may not be handled in order
    pub fn scroll_smooth_event(
        &self,
        delta: stardust_xr_protocol::types::proxies::Vec2F,
        source: impl Into<ScrollSource>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> Result<(), gluon::SendError> {
        let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
        let source: ScrollSource = source.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "scroll_smooth", ? delta, ? source, ?
            timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let gluon_ret: Option<gluon::ObjectOrRef> = None;
        gluon_ret.write(&mut gluon_builder)?;
        delta.write(&mut gluon_builder)?;
        source.write(&mut gluon_builder)?;
        timestamp.write(&mut gluon_builder)?;
        self.obj
            .device()
            .transact_one_way(&self.obj, 10u32, gluon_builder.to_payload())?;
        Ok(())
    }
    ///delta is +Y == Up +X == Right
    pub fn scroll_discrete(
        &self,
        delta: stardust_xr_protocol::types::proxies::Vec2F,
        source: impl Into<ScrollSource>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> gluon::OnewayFuture {
        use gluon::ToObjectOrRef as _;
        let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
        let source: ScrollSource = source.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "scroll_discrete", ? delta, ? source, ?
            timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        let gluon_ret: Option<gluon::ObjectOrRef> = Some(
            gluon_ret.to_binder_object_or_ref(),
        );
        if let Err(err) = gluon_ret.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = delta.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = source.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = timestamp.write(&mut gluon_builder) {
            return err.into();
        }
        if let Err(err) = self
            .obj
            .device()
            .transact_one_way(&self.obj, 11u32, gluon_builder.to_payload())
        {
            return err.into();
        }
        gluon_recv.into()
    }
    ///delta is +Y == Up +X == Right
    ///Fire and Forget, events sent to different objects may not be handled in order
    pub fn scroll_discrete_event(
        &self,
        delta: stardust_xr_protocol::types::proxies::Vec2F,
        source: impl Into<ScrollSource>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> Result<(), gluon::SendError> {
        let delta: stardust_xr_protocol::types::proxied::Vec2F = delta.into();
        let source: ScrollSource = source.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "MouseHandler", method = "scroll_discrete", ? delta, ? source, ?
            timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let gluon_ret: Option<gluon::ObjectOrRef> = None;
        gluon_ret.write(&mut gluon_builder)?;
        delta.write(&mut gluon_builder)?;
        source.write(&mut gluon_builder)?;
        timestamp.write(&mut gluon_builder)?;
        self.obj
            .device()
            .transact_one_way(&self.obj, 11u32, gluon_builder.to_payload())?;
        Ok(())
    }
    pub fn from_handler<H: MouseHandlerHandler>(
        obj: &impl gluon::OwnedObjectRef<H>,
    ) -> MouseHandler {
        MouseHandler::from_object_or_ref(gluon::OwnedObjectRef::to_object_or_ref(obj))
    }
    ///only use this when you know the binder ref implements this interface, else the consquences are for you to find out
    pub fn from_object_or_ref(obj: gluon::ObjectOrRef) -> MouseHandler {
        MouseHandler { obj }
    }
}
impl From<MouseHandler> for gluon::ObjectOrRef {
    fn from(value: MouseHandler) -> Self {
        value.obj
    }
}
impl gluon::ToObjectOrRef for MouseHandler {
    fn to_binder_object_or_ref(&self) -> gluon::ObjectOrRef {
        self.obj.clone()
    }
}
impl gluon::Liveness for MouseHandler {
    fn alive(&self) -> bool {
        gluon::Liveness::alive(&self.obj)
    }
    fn death_notification(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        gluon::Liveness::death_notification(&self.obj)
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
                    let gluon_ret: Option<gluon::ObjectOrRef> = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
                    let __wire_param_delta: stardust_xr_protocol::types::proxied::Vec2F = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
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
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "MouseHandler", method =
                                "motion", method_id = 8u32
                            ),
                        )
                        .await;
                    if let Some(obj) = gluon_ret {
                        obj.device()
                            .transact_one_way(
                                &obj,
                                0,
                                gluon::DataBuilder::new().to_payload(),
                            )?;
                    }
                }
                9u32 => {
                    let gluon_ret: Option<gluon::ObjectOrRef> = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
                    let param_button = gluon::Convertable::read(&mut gluon_data)?;
                    let param_pressed = gluon::Convertable::read(&mut gluon_data)?;
                    let param_timestamp = gluon::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "MouseHandler", method = "button", ? param_button, ?
                        param_pressed, ? param_timestamp, "dispatching"
                    );
                    drop(gluon_data);
                    self.button(ctx, param_button, param_pressed, param_timestamp)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "MouseHandler", method =
                                "button", method_id = 9u32
                            ),
                        )
                        .await;
                    if let Some(obj) = gluon_ret {
                        obj.device()
                            .transact_one_way(
                                &obj,
                                0,
                                gluon::DataBuilder::new().to_payload(),
                            )?;
                    }
                }
                10u32 => {
                    let gluon_ret: Option<gluon::ObjectOrRef> = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
                    let __wire_param_delta: stardust_xr_protocol::types::proxied::Vec2F = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
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
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "MouseHandler", method =
                                "scroll_smooth", method_id = 10u32
                            ),
                        )
                        .await;
                    if let Some(obj) = gluon_ret {
                        obj.device()
                            .transact_one_way(
                                &obj,
                                0,
                                gluon::DataBuilder::new().to_payload(),
                            )?;
                    }
                }
                11u32 => {
                    let gluon_ret: Option<gluon::ObjectOrRef> = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
                    let __wire_param_delta: stardust_xr_protocol::types::proxied::Vec2F = gluon::Convertable::read(
                        &mut gluon_data,
                    )?;
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
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "MouseHandler", method =
                                "scroll_discrete", method_id = 11u32
                            ),
                        )
                        .await;
                    if let Some(obj) = gluon_ret {
                        obj.device()
                            .transact_one_way(
                                &obj,
                                0,
                                gluon::DataBuilder::new().to_payload(),
                            )?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }
}
pub mod proxied {
    use super::*;
}
