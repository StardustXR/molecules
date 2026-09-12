#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon_ipc::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon_ipc::ExternalProtocol = gluon_ipc::ExternalProtocol {
    protocol_name: "org.stardustxr.KeyboardHandler",
    types: &[
        gluon_ipc::ExternalGluonType {
            name: "KeyEvent",
            supported_derives: gluon_ipc::Derives::from_bits_truncate(30u32),
            proxy: None,
        },
        gluon_ipc::ExternalGluonType {
            name: "ModifierState",
            supported_derives: gluon_ipc::Derives::from_bits_truncate(1023u32),
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
impl gluon_ipc::Convertable for KeyEvent {
    fn write(
        &self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.keycode.write(gluon_data)?;
        self.pressed.write(gluon_data)?;
        self.modifiers.write(gluon_data)?;
        self.keymap.write(gluon_data)?;
        Ok(())
    }
    fn read(
        gluon_data: &mut gluon_ipc::DataReader,
    ) -> Result<Self, gluon_ipc::ReadError> {
        let keycode = gluon_ipc::Convertable::read(gluon_data)?;
        let pressed = gluon_ipc::Convertable::read(gluon_data)?;
        let modifiers = gluon_ipc::Convertable::read(gluon_data)?;
        let keymap = gluon_ipc::Convertable::read(gluon_data)?;
        Ok(KeyEvent {
            keycode,
            pressed,
            modifiers,
            keymap,
        })
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
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
impl gluon_ipc::Convertable for ModifierState {
    fn write(
        &self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.depressed.write(gluon_data)?;
        self.latched.write(gluon_data)?;
        self.locked.write(gluon_data)?;
        self.layout_group.write(gluon_data)?;
        Ok(())
    }
    fn read(
        gluon_data: &mut gluon_ipc::DataReader,
    ) -> Result<Self, gluon_ipc::ReadError> {
        let depressed = gluon_ipc::Convertable::read(gluon_data)?;
        let latched = gluon_ipc::Convertable::read(gluon_data)?;
        let locked = gluon_ipc::Convertable::read(gluon_data)?;
        let layout_group = gluon_ipc::Convertable::read(gluon_data)?;
        Ok(ModifierState {
            depressed,
            latched,
            locked,
            layout_group,
        })
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.depressed.write_owned(gluon_data)?;
        self.latched.write_owned(gluon_data)?;
        self.locked.write_owned(gluon_data)?;
        self.layout_group.write_owned(gluon_data)?;
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct KeyboardHandler {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for KeyboardHandler {
    fn write(
        &self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(
        gluon_data: &mut gluon_ipc::DataReader,
    ) -> Result<Self, gluon_ipc::ReadError> {
        let obj = gluon_ipc::Ref::read(gluon_data)?;
        Ok(KeyboardHandler::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl KeyboardHandler {
    const ID: &'static str = "org.stardustxr.KeyboardHandler.KeyboardHandler";
}
impl gluon_ipc::Interface for KeyboardHandler {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: KeyboardHandlerHandler> gluon_ipc::HandledBy<H> for KeyboardHandler {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type KeyboardHandlerLocal<H> = gluon_ipc::LocalRef<KeyboardHandler, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: KeyboardHandlerHandler> From<KeyboardHandlerLocal<H>> for KeyboardHandler {
    fn from(value: KeyboardHandlerLocal<H>) -> KeyboardHandler {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for KeyboardHandler {
    fn from_ref(obj: gluon_ipc::Ref) -> KeyboardHandler {
        KeyboardHandler { obj }
    }
}
impl KeyboardHandler {
    pub fn key(
        &self,
        event: impl Into<KeyEvent>,
        timestamp: impl Into<Option<stardust_xr_protocol::types::Timestamp>>,
    ) -> Result<(), gluon_ipc::SendError> {
        let event: KeyEvent = event.into();
        let timestamp: Option<stardust_xr_protocol::types::Timestamp> = timestamp.into();
        tracing::trace!(
            interface = "KeyboardHandler", method = "key", ? event, ? timestamp, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        event.write(&mut gluon_builder)?;
        timestamp.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> KeyboardHandler {
        KeyboardHandler { obj }
    }
}
impl From<KeyboardHandler> for gluon_ipc::Ref {
    fn from(value: KeyboardHandler) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for KeyboardHandler {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for KeyboardHandler {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
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
pub trait KeyboardHandlerHandler: gluon_ipc::Handler + Send + Sync + 'static {
    fn key(
        &self,
        _ctx: gluon_ipc::Context,
        event: KeyEvent,
        timestamp: Option<stardust_xr_protocol::types::Timestamp>,
    ) -> impl Future<Output = ()> + Send + Sync;
    fn dispatch_one_way(
        &self,
        transaction_code: u32,
        mut gluon_data: gluon_ipc::DataReader,
        ctx: gluon_ipc::Context,
    ) -> impl Future<Output = Result<(), gluon_ipc::SendError>> + Send + Sync {
        async move {
            match transaction_code {
                8u32 => {
                    let param_event = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let param_timestamp = gluon_ipc::Convertable::read(&mut gluon_data)?;
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<KeyboardHandler, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        KeyboardHandler::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<KeyboardHandler, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        KeyboardHandler::new_service(self)
    }
}
pub mod proxied {
    use super::*;
}
