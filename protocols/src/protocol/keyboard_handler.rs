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
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
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
        gluon_data: &mut gluon::DataBuilder,
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
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
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
        gluon_data: &mut gluon::DataBuilder,
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
    obj: gluon::Ref,
}
impl gluon::Convertable for KeyboardHandler {
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::Ref::read(gluon_data)?;
        Ok(KeyboardHandler::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl KeyboardHandler {
    const ID: &'static str = "org.stardustxr.KeyboardHandler.KeyboardHandler";
}
impl gluon::Interface for KeyboardHandler {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: KeyboardHandlerHandler> gluon::HandledBy<H> for KeyboardHandler {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type KeyboardHandlerLocal<H> = gluon::LocalRef<KeyboardHandler, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: KeyboardHandlerHandler> From<KeyboardHandlerLocal<H>> for KeyboardHandler {
    fn from(value: KeyboardHandlerLocal<H>) -> KeyboardHandler {
        value.into_proxy()
    }
}
impl gluon::RefExt for KeyboardHandler {
    fn from_ref(obj: gluon::Ref) -> KeyboardHandler {
        KeyboardHandler { obj }
    }
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
        gluon::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon::Ref) -> KeyboardHandler {
        KeyboardHandler { obj }
    }
}
impl From<KeyboardHandler> for gluon::Ref {
    fn from(value: KeyboardHandler) -> Self {
        value.obj
    }
}
impl gluon::ToRef for KeyboardHandler {
    fn to_ref(&self) -> gluon::Ref {
        self.obj.clone()
    }
}
impl gluon::Liveness for KeyboardHandler {
    fn death_notifier(&self) -> gluon::DeathNotifier {
        gluon::Liveness::death_notifier(&self.obj)
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
    fn to_node(
        self,
    ) -> Result<
        (gluon::Node<Self>, gluon::LocalRef<KeyboardHandler, Self>),
        gluon::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon::RefExt;
        KeyboardHandler::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon::LocalRef<KeyboardHandler, Self>, gluon::NodeError>
    where
        Self: Sized,
    {
        use gluon::RefExt;
        KeyboardHandler::new_service(self)
    }
}
pub mod proxied {
    use super::*;
}
