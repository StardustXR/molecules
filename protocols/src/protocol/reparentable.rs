#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
    protocol_name: "org.stardustxr.Reparentable",
    types: &[],
};
pub mod proxies {
    use super::*;
}
#[derive(Debug, Clone)]
pub struct ReparentableLocked {
    obj: gluon::Ref,
}
impl gluon::Convertable for ReparentableLocked {
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::Ref::read(gluon_data)?;
        Ok(ReparentableLocked::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for ReparentableLocked {
    const ID: &'static str = "org.stardustxr.Reparentable.ReparentableLocked";
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: ReparentableLockedHandler> gluon::HandledBy<H> for ReparentableLocked {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type ReparentableLockedLocal<H> = gluon::LocalRef<ReparentableLocked, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: ReparentableLockedHandler> From<ReparentableLockedLocal<H>>
for ReparentableLocked {
    fn from(value: ReparentableLockedLocal<H>) -> ReparentableLocked {
        value.into_proxy()
    }
}
impl gluon::RefExt for ReparentableLocked {
    fn from_ref(obj: gluon::Ref) -> ReparentableLocked {
        ReparentableLocked { obj }
    }
}
impl ReparentableLocked {
    ///Reparents this object, locking this Reparentable to make sure others can't steal this reparent, can steal from non-locking reparents
    pub async fn reparent_locking(
        &self,
        new_parent: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        keepalive: impl Into<ReparentKeepalive>,
    ) -> Result<Option<ReparentHandle>, gluon::SendError> {
        let new_parent: stardust_xr_protocol::spatial::SpatialRef = new_parent.into();
        let keepalive: ReparentKeepalive = keepalive.into();
        tracing::trace!(
            interface = "ReparentableLocked", method = "reparent_locking", ? new_parent,
            ? keepalive, "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let (gluon_ret_node, gluon_ret) = gluon::Node::new(gluon_ret_handler)?;
        gluon_builder.write_ref(&gluon_ret)?;
        new_parent.write(&mut gluon_builder)?;
        keepalive.write(&mut gluon_builder)?;
        gluon::transact(&self.obj, 8u32, gluon_builder)?;
        let mut reader = gluon_recv.recv().await.unwrap();
        drop(gluon_ret_node);
        let __ret_handle = gluon::Convertable::read(&mut reader)?;
        tracing::trace!(
            interface = "ReparentableLocked", method = "reparent_locking", ?
            __ret_handle, "←"
        );
        Ok(__ret_handle)
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon::Ref) -> ReparentableLocked {
        ReparentableLocked { obj }
    }
}
impl From<ReparentableLocked> for gluon::Ref {
    fn from(value: ReparentableLocked) -> Self {
        value.obj
    }
}
impl gluon::ToRef for ReparentableLocked {
    fn to_ref(&self) -> gluon::Ref {
        self.obj.clone()
    }
}
impl gluon::Liveness for ReparentableLocked {
    fn death_notifier(&self) -> gluon::DeathNotifier {
        gluon::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for ReparentableLocked {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for ReparentableLocked {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for ReparentableLocked {}
pub trait ReparentableLockedHandler: gluon::Handler + Send + Sync + 'static {
    ///Reparents this object, locking this Reparentable to make sure others can't steal this reparent, can steal from non-locking reparents
    fn reparent_locking(
        &self,
        _ctx: gluon::Context,
        new_parent: stardust_xr_protocol::spatial::SpatialRef,
        keepalive: ReparentKeepalive,
    ) -> impl Future<Output = Option<ReparentHandle>> + Send + Sync;
    ///Dispatched instead of [`Self::reparent_locking`] so a slow reply doesn't hold up dispatch of the next transaction. The default implementation just awaits `reparent_locking` and sends the result through `reply`. Override this method instead of `reparent_locking` to defer the reply: stash `reply` (it's `Send + Sync + 'static`) somewhere else — a channel, a queue, another task — and return as soon as this method's future is done, without waiting for the reply to actually be sent.
    fn reparent_locking_oneway(
        &self,
        _ctx: gluon::Context,
        new_parent: stardust_xr_protocol::spatial::SpatialRef,
        keepalive: ReparentKeepalive,
        reply: gluon::ReplySender<Option<ReparentHandle>>,
    ) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
        async move {
            let handle = self.reparent_locking(_ctx, new_parent, keepalive).await;
            reply.send(handle)
        }
    }
    fn dispatch_one_way(
        &self,
        transaction_code: u32,
        mut gluon_data: gluon::DataReader,
        ctx: gluon::Context,
    ) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
        async move {
            match transaction_code {
                8u32 => {
                    let return_callback = gluon_data.read_ref()?;
                    let param_new_parent = gluon::Convertable::read(&mut gluon_data)?;
                    let param_keepalive = gluon::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "ReparentableLocked", method = "reparent_locking", ?
                        param_new_parent, ? param_keepalive, "dispatching"
                    );
                    drop(gluon_data);
                    let reply: gluon::ReplySender<Option<ReparentHandle>> = gluon::ReplySender::new(
                        return_callback,
                        |handle, gluon_out| {
                            tracing::trace!(
                                interface = "ReparentableLocked", method =
                                "reparent_locking", ? handle, "←"
                            );
                            handle.write_owned(gluon_out)?;
                            Ok(())
                        },
                    );
                    self.reparent_locking_oneway(
                            ctx,
                            param_new_parent,
                            param_keepalive,
                            reply,
                        )
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "ReparentableLocked", method =
                                "reparent_locking", method_id = 8u32
                            ),
                        )
                        .await?;
                }
                _ => {}
            }
            Ok(())
        }
    }
}
#[derive(Debug, Clone)]
pub struct Reparentable {
    obj: gluon::Ref,
}
impl gluon::Convertable for Reparentable {
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::Ref::read(gluon_data)?;
        Ok(Reparentable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for Reparentable {
    const ID: &'static str = "org.stardustxr.Reparentable.Reparentable";
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: ReparentableHandler> gluon::HandledBy<H> for Reparentable {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type ReparentableLocal<H> = gluon::LocalRef<Reparentable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: ReparentableHandler> From<ReparentableLocal<H>> for Reparentable {
    fn from(value: ReparentableLocal<H>) -> Reparentable {
        value.into_proxy()
    }
}
impl gluon::RefExt for Reparentable {
    fn from_ref(obj: gluon::Ref) -> Reparentable {
        Reparentable { obj }
    }
}
impl Reparentable {
    ///Reparents this object, this is non-locking, others can steal this reparent
    pub async fn reparent(
        &self,
        new_parent: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        keepalive: impl Into<ReparentKeepalive>,
    ) -> Result<Option<ReparentHandle>, gluon::SendError> {
        let new_parent: stardust_xr_protocol::spatial::SpatialRef = new_parent.into();
        let keepalive: ReparentKeepalive = keepalive.into();
        tracing::trace!(
            interface = "Reparentable", method = "reparent", ? new_parent, ? keepalive,
            "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let (gluon_ret_node, gluon_ret) = gluon::Node::new(gluon_ret_handler)?;
        gluon_builder.write_ref(&gluon_ret)?;
        new_parent.write(&mut gluon_builder)?;
        keepalive.write(&mut gluon_builder)?;
        gluon::transact(&self.obj, 8u32, gluon_builder)?;
        let mut reader = gluon_recv.recv().await.unwrap();
        drop(gluon_ret_node);
        let __ret_handle = gluon::Convertable::read(&mut reader)?;
        tracing::trace!(
            interface = "Reparentable", method = "reparent", ? __ret_handle, "←"
        );
        Ok(__ret_handle)
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon::Ref) -> Reparentable {
        Reparentable { obj }
    }
}
impl From<Reparentable> for gluon::Ref {
    fn from(value: Reparentable) -> Self {
        value.obj
    }
}
impl gluon::ToRef for Reparentable {
    fn to_ref(&self) -> gluon::Ref {
        self.obj.clone()
    }
}
impl gluon::Liveness for Reparentable {
    fn death_notifier(&self) -> gluon::DeathNotifier {
        gluon::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Reparentable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Reparentable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Reparentable {}
pub trait ReparentableHandler: gluon::Handler + Send + Sync + 'static {
    ///Reparents this object, this is non-locking, others can steal this reparent
    fn reparent(
        &self,
        _ctx: gluon::Context,
        new_parent: stardust_xr_protocol::spatial::SpatialRef,
        keepalive: ReparentKeepalive,
    ) -> impl Future<Output = Option<ReparentHandle>> + Send + Sync;
    ///Dispatched instead of [`Self::reparent`] so a slow reply doesn't hold up dispatch of the next transaction. The default implementation just awaits `reparent` and sends the result through `reply`. Override this method instead of `reparent` to defer the reply: stash `reply` (it's `Send + Sync + 'static`) somewhere else — a channel, a queue, another task — and return as soon as this method's future is done, without waiting for the reply to actually be sent.
    fn reparent_oneway(
        &self,
        _ctx: gluon::Context,
        new_parent: stardust_xr_protocol::spatial::SpatialRef,
        keepalive: ReparentKeepalive,
        reply: gluon::ReplySender<Option<ReparentHandle>>,
    ) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
        async move {
            let handle = self.reparent(_ctx, new_parent, keepalive).await;
            reply.send(handle)
        }
    }
    fn dispatch_one_way(
        &self,
        transaction_code: u32,
        mut gluon_data: gluon::DataReader,
        ctx: gluon::Context,
    ) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
        async move {
            match transaction_code {
                8u32 => {
                    let return_callback = gluon_data.read_ref()?;
                    let param_new_parent = gluon::Convertable::read(&mut gluon_data)?;
                    let param_keepalive = gluon::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "Reparentable", method = "reparent", ?
                        param_new_parent, ? param_keepalive, "dispatching"
                    );
                    drop(gluon_data);
                    let reply: gluon::ReplySender<Option<ReparentHandle>> = gluon::ReplySender::new(
                        return_callback,
                        |handle, gluon_out| {
                            tracing::trace!(
                                interface = "Reparentable", method = "reparent", ? handle,
                                "←"
                            );
                            handle.write_owned(gluon_out)?;
                            Ok(())
                        },
                    );
                    self.reparent_oneway(ctx, param_new_parent, param_keepalive, reply)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Reparentable", method =
                                "reparent", method_id = 8u32
                            ),
                        )
                        .await?;
                }
                _ => {}
            }
            Ok(())
        }
    }
}
#[derive(Debug, Clone)]
pub struct ReparentKeepalive {
    obj: gluon::Ref,
}
impl gluon::Convertable for ReparentKeepalive {
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::Ref::read(gluon_data)?;
        Ok(ReparentKeepalive::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for ReparentKeepalive {
    const ID: &'static str = "org.stardustxr.Reparentable.ReparentKeepalive";
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: ReparentKeepaliveHandler> gluon::HandledBy<H> for ReparentKeepalive {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type ReparentKeepaliveLocal<H> = gluon::LocalRef<ReparentKeepalive, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: ReparentKeepaliveHandler> From<ReparentKeepaliveLocal<H>> for ReparentKeepalive {
    fn from(value: ReparentKeepaliveLocal<H>) -> ReparentKeepalive {
        value.into_proxy()
    }
}
impl gluon::RefExt for ReparentKeepalive {
    fn from_ref(obj: gluon::Ref) -> ReparentKeepalive {
        ReparentKeepalive { obj }
    }
}
impl ReparentKeepalive {
    ///The reparent this object was associated with was stolen, the ReparentHandle becomes invalid
    pub fn reparent_stolen(&self) -> Result<(), gluon::SendError> {
        tracing::trace!(
            interface = "ReparentKeepalive", method = "reparent_stolen", "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        gluon::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon::Ref) -> ReparentKeepalive {
        ReparentKeepalive { obj }
    }
}
impl From<ReparentKeepalive> for gluon::Ref {
    fn from(value: ReparentKeepalive) -> Self {
        value.obj
    }
}
impl gluon::ToRef for ReparentKeepalive {
    fn to_ref(&self) -> gluon::Ref {
        self.obj.clone()
    }
}
impl gluon::Liveness for ReparentKeepalive {
    fn death_notifier(&self) -> gluon::DeathNotifier {
        gluon::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for ReparentKeepalive {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for ReparentKeepalive {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for ReparentKeepalive {}
pub trait ReparentKeepaliveHandler: gluon::Handler + Send + Sync + 'static {
    ///The reparent this object was associated with was stolen, the ReparentHandle becomes invalid
    fn reparent_stolen(
        &self,
        _ctx: gluon::Context,
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
                    tracing::trace!(
                        interface = "ReparentKeepalive", method = "reparent_stolen",
                        "dispatching"
                    );
                    drop(gluon_data);
                    self.reparent_stolen(ctx)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "ReparentKeepalive", method =
                                "reparent_stolen", method_id = 8u32
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
#[derive(Debug, Clone)]
pub struct ReparentHandle {
    obj: gluon::Ref,
}
impl gluon::Convertable for ReparentHandle {
    fn write(
        &self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::Ref::read(gluon_data)?;
        Ok(ReparentHandle::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for ReparentHandle {
    const ID: &'static str = "org.stardustxr.Reparentable.ReparentHandle";
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: ReparentHandleHandler> gluon::HandledBy<H> for ReparentHandle {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type ReparentHandleLocal<H> = gluon::LocalRef<ReparentHandle, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: ReparentHandleHandler> From<ReparentHandleLocal<H>> for ReparentHandle {
    fn from(value: ReparentHandleLocal<H>) -> ReparentHandle {
        value.into_proxy()
    }
}
impl gluon::RefExt for ReparentHandle {
    fn from_ref(obj: gluon::Ref) -> ReparentHandle {
        ReparentHandle { obj }
    }
}
impl ReparentHandle {
    ///Set transform relative to the given SpatialRef to IDENTITY
    pub fn reset_transform(
        &self,
        relative_to: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
    ) -> Result<(), gluon::SendError> {
        let relative_to: stardust_xr_protocol::spatial::SpatialRef = relative_to.into();
        tracing::trace!(
            interface = "ReparentHandle", method = "reset_transform", ? relative_to,
            "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        relative_to.write(&mut gluon_builder)?;
        gluon::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon::Ref) -> ReparentHandle {
        ReparentHandle { obj }
    }
}
impl From<ReparentHandle> for gluon::Ref {
    fn from(value: ReparentHandle) -> Self {
        value.obj
    }
}
impl gluon::ToRef for ReparentHandle {
    fn to_ref(&self) -> gluon::Ref {
        self.obj.clone()
    }
}
impl gluon::Liveness for ReparentHandle {
    fn death_notifier(&self) -> gluon::DeathNotifier {
        gluon::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for ReparentHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for ReparentHandle {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for ReparentHandle {}
pub trait ReparentHandleHandler: gluon::Handler + Send + Sync + 'static {
    ///Set transform relative to the given SpatialRef to IDENTITY
    fn reset_transform(
        &self,
        _ctx: gluon::Context,
        relative_to: stardust_xr_protocol::spatial::SpatialRef,
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
                    let param_relative_to = gluon::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "ReparentHandle", method = "reset_transform", ?
                        param_relative_to, "dispatching"
                    );
                    drop(gluon_data);
                    self.reset_transform(ctx, param_relative_to)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "ReparentHandle", method =
                                "reset_transform", method_id = 8u32
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
