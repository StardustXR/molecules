#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
    protocol_name: "org.stardustxr.KeyboardHandler",
    types: &[
        gluon::ExternalGluonType {
            name: "KeyEvent",
            supported_derives: gluon::Derives::from_bits_truncate(30u32),
            proxy: None,
        },
        gluon::ExternalGluonType {
            name: "ModifierState",
            supported_derives: gluon::Derives::from_bits_truncate(1023u32),
            proxy: None,
        },
    ],
};
pub mod proxies {
    use super::*;
}
///A event for a key state change
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyEvent {
    ///Linux event code, to get an xkbcommon keycode add 8
    pub keycode: u32,
    pub pressed: bool,
    ///Current modifier state
    pub modifiers: ModifierState,
    ///Current keymap
    pub keymap: stardust_xr_protocol::keymap::Keymap,
}
impl gluon::Convertable for KeyEvent {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.keycode.write(gluon_data)?;
        self.pressed.write(gluon_data)?;
        self.modifiers.write(gluon_data)?;
        self.keymap.write(gluon_data)?;
        Ok(())
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let keycode = gluon::Convertable::read(gluon_data)?;
        let pressed = gluon::Convertable::read(gluon_data)?;
        let modifiers = gluon::Convertable::read(gluon_data)?;
        let keymap = gluon::Convertable::read(gluon_data)?;
        Ok(KeyEvent {
            keycode,
            pressed,
            modifiers,
            keymap,
        })
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.keycode.write_owned(gluon_data)?;
        self.pressed.write_owned(gluon_data)?;
        self.modifiers.write_owned(gluon_data)?;
        self.keymap.write_owned(gluon_data)?;
        Ok(())
    }
}
///Modifier state driven by xkbcommon
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierState {
    pub depressed: u32,
    pub latched: u32,
    pub locked: u32,
    pub layout_group: u32,
}
impl gluon::Convertable for ModifierState {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.depressed.write(gluon_data)?;
        self.latched.write(gluon_data)?;
        self.locked.write(gluon_data)?;
        self.layout_group.write(gluon_data)?;
        Ok(())
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let depressed = gluon::Convertable::read(gluon_data)?;
        let latched = gluon::Convertable::read(gluon_data)?;
        let locked = gluon::Convertable::read(gluon_data)?;
        let layout_group = gluon::Convertable::read(gluon_data)?;
        Ok(ModifierState {
            depressed,
            latched,
            locked,
            layout_group,
        })
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.depressed.write_owned(gluon_data)?;
        self.latched.write_owned(gluon_data)?;
        self.locked.write_owned(gluon_data)?;
        self.layout_group.write_owned(gluon_data)?;
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct KeyboardHandler {
    obj: gluon::ObjectOrRef,
}
impl gluon::Convertable for KeyboardHandler {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::ObjectOrRef::read(gluon_data)?;
        Ok(KeyboardHandler::from_object_or_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for KeyboardHandler {
    const ID: &'static str = "org.stardustxr.KeyboardHandler.KeyboardHandler";
}
impl KeyboardHandler {
    pub fn key(
        &self,
        event: impl Into<KeyEvent>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> Result<(), gluon::SendError> {
        let event: KeyEvent = event.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "KeyboardHandler", method = "key", ? event, ? timestamp, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        event.write(&mut gluon_builder)?;
        timestamp.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        Ok(())
    }
    pub fn from_handler<H: KeyboardHandlerHandler>(
        obj: &impl gluon::OwnedObjectRef<H>,
    ) -> KeyboardHandler {
        KeyboardHandler::from_object_or_ref(gluon::OwnedObjectRef::to_object_or_ref(obj))
    }
    ///only use this when you know the binder ref implements this interface, else the consquences are for you to find out
    pub fn from_object_or_ref(obj: gluon::ObjectOrRef) -> KeyboardHandler {
        KeyboardHandler { obj }
    }
}
impl From<KeyboardHandler> for gluon::ObjectOrRef {
    fn from(value: KeyboardHandler) -> Self {
        value.obj
    }
}
impl gluon::ToObjectOrRef for KeyboardHandler {
    fn to_binder_object_or_ref(&self) -> gluon::ObjectOrRef {
        self.obj.clone()
    }
}
impl gluon::Liveness for KeyboardHandler {
    fn alive(&self) -> bool {
        gluon::Liveness::alive(&self.obj)
    }
    fn death_notification(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        gluon::Liveness::death_notification(&self.obj)
    }
}
impl std::hash::Hash for KeyboardHandler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for KeyboardHandler {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for KeyboardHandler {}
pub trait KeyboardHandlerHandler: gluon::Handler + Send + Sync + 'static {
    fn key(
        &self,
        _ctx: gluon::Context,
        event: KeyEvent,
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
                    let param_event = gluon::Convertable::read(&mut gluon_data)?;
                    let param_timestamp = gluon::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "KeyboardHandler", method = "key", ? param_event, ?
                        param_timestamp, "dispatching"
                    );
                    drop(gluon_data);
                    self.key(ctx, param_event, param_timestamp)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "KeyboardHandler", method =
                                "key", method_id = 8u32
                            ),
                        )
                        .await;
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
