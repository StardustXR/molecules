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
    obj: gluon::ObjectOrRef,
}
impl gluon::Convertable for ReparentableLocked {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::ObjectOrRef::read(gluon_data)?;
        Ok(ReparentableLocked::from_object_or_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for ReparentableLocked {
    const ID: &'static str = "org.stardustxr.Reparentable.ReparentableLocked";
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
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        gluon_builder.write_binder(&gluon_ret)?;
        new_parent.write(&mut gluon_builder)?;
        keepalive.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        let transaction = gluon_recv.recv().await.unwrap();
        let mut reader = gluon::DataReader::from_payload(transaction.payload);
        let __ret_handle = gluon::Convertable::read(&mut reader)?;
        tracing::trace!(
            interface = "ReparentableLocked", method = "reparent_locking", ?
            __ret_handle, "←"
        );
        Ok(__ret_handle)
    }
    pub fn from_handler<H: ReparentableLockedHandler>(
        obj: &impl gluon::OwnedObjectRef<H>,
    ) -> ReparentableLocked {
        ReparentableLocked::from_object_or_ref(
            gluon::OwnedObjectRef::to_object_or_ref(obj),
        )
    }
    ///only use this when you know the binder ref implements this interface, else the consquences are for you to find out
    pub fn from_object_or_ref(obj: gluon::ObjectOrRef) -> ReparentableLocked {
        ReparentableLocked { obj }
    }
}
impl From<ReparentableLocked> for gluon::ObjectOrRef {
    fn from(value: ReparentableLocked) -> Self {
        value.obj
    }
}
impl gluon::ToObjectOrRef for ReparentableLocked {
    fn to_binder_object_or_ref(&self) -> gluon::ObjectOrRef {
        self.obj.clone()
    }
}
impl gluon::Liveness for ReparentableLocked {
    fn alive(&self) -> bool {
        gluon::Liveness::alive(&self.obj)
    }
    fn death_notification(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        gluon::Liveness::death_notification(&self.obj)
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
                    let return_callback = gluon_data.read_binder()?;
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
    obj: gluon::ObjectOrRef,
}
impl gluon::Convertable for Reparentable {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::ObjectOrRef::read(gluon_data)?;
        Ok(Reparentable::from_object_or_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for Reparentable {
    const ID: &'static str = "org.stardustxr.Reparentable.Reparentable";
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
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        gluon_builder.write_binder(&gluon_ret)?;
        new_parent.write(&mut gluon_builder)?;
        keepalive.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        let transaction = gluon_recv.recv().await.unwrap();
        let mut reader = gluon::DataReader::from_payload(transaction.payload);
        let __ret_handle = gluon::Convertable::read(&mut reader)?;
        tracing::trace!(
            interface = "Reparentable", method = "reparent", ? __ret_handle, "←"
        );
        Ok(__ret_handle)
    }
    pub fn from_handler<H: ReparentableHandler>(
        obj: &impl gluon::OwnedObjectRef<H>,
    ) -> Reparentable {
        Reparentable::from_object_or_ref(gluon::OwnedObjectRef::to_object_or_ref(obj))
    }
    ///only use this when you know the binder ref implements this interface, else the consquences are for you to find out
    pub fn from_object_or_ref(obj: gluon::ObjectOrRef) -> Reparentable {
        Reparentable { obj }
    }
}
impl From<Reparentable> for gluon::ObjectOrRef {
    fn from(value: Reparentable) -> Self {
        value.obj
    }
}
impl gluon::ToObjectOrRef for Reparentable {
    fn to_binder_object_or_ref(&self) -> gluon::ObjectOrRef {
        self.obj.clone()
    }
}
impl gluon::Liveness for Reparentable {
    fn alive(&self) -> bool {
        gluon::Liveness::alive(&self.obj)
    }
    fn death_notification(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        gluon::Liveness::death_notification(&self.obj)
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
                    let return_callback = gluon_data.read_binder()?;
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
    obj: gluon::ObjectOrRef,
}
impl gluon::Convertable for ReparentKeepalive {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::ObjectOrRef::read(gluon_data)?;
        Ok(ReparentKeepalive::from_object_or_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for ReparentKeepalive {
    const ID: &'static str = "org.stardustxr.Reparentable.ReparentKeepalive";
}
impl ReparentKeepalive {
    ///The reparent this object was associated with was stolen, the ReparentHandle becomes invalid
    pub fn reparent_stolen(&self) -> Result<(), gluon::SendError> {
        tracing::trace!(
            interface = "ReparentKeepalive", method = "reparent_stolen", "→"
        );
        let mut gluon_builder = gluon::DataBuilder::new();
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        Ok(())
    }
    pub fn from_handler<H: ReparentKeepaliveHandler>(
        obj: &impl gluon::OwnedObjectRef<H>,
    ) -> ReparentKeepalive {
        ReparentKeepalive::from_object_or_ref(
            gluon::OwnedObjectRef::to_object_or_ref(obj),
        )
    }
    ///only use this when you know the binder ref implements this interface, else the consquences are for you to find out
    pub fn from_object_or_ref(obj: gluon::ObjectOrRef) -> ReparentKeepalive {
        ReparentKeepalive { obj }
    }
}
impl From<ReparentKeepalive> for gluon::ObjectOrRef {
    fn from(value: ReparentKeepalive) -> Self {
        value.obj
    }
}
impl gluon::ToObjectOrRef for ReparentKeepalive {
    fn to_binder_object_or_ref(&self) -> gluon::ObjectOrRef {
        self.obj.clone()
    }
}
impl gluon::Liveness for ReparentKeepalive {
    fn alive(&self) -> bool {
        gluon::Liveness::alive(&self.obj)
    }
    fn death_notification(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        gluon::Liveness::death_notification(&self.obj)
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
    obj: gluon::ObjectOrRef,
}
impl gluon::Convertable for ReparentHandle {
    fn write<'a, 'b: 'a>(
        &'b self,
        gluon_data: &mut gluon::DataBuilder<'a>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write(gluon_data)
    }
    fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
        let obj = gluon::ObjectOrRef::read(gluon_data)?;
        Ok(ReparentHandle::from_object_or_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon::DataBuilder<'_>,
    ) -> Result<(), gluon::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl gluon::Interface for ReparentHandle {
    const ID: &'static str = "org.stardustxr.Reparentable.ReparentHandle";
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
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        Ok(())
    }
    pub fn from_handler<H: ReparentHandleHandler>(
        obj: &impl gluon::OwnedObjectRef<H>,
    ) -> ReparentHandle {
        ReparentHandle::from_object_or_ref(gluon::OwnedObjectRef::to_object_or_ref(obj))
    }
    ///only use this when you know the binder ref implements this interface, else the consquences are for you to find out
    pub fn from_object_or_ref(obj: gluon::ObjectOrRef) -> ReparentHandle {
        ReparentHandle { obj }
    }
}
impl From<ReparentHandle> for gluon::ObjectOrRef {
    fn from(value: ReparentHandle) -> Self {
        value.obj
    }
}
impl gluon::ToObjectOrRef for ReparentHandle {
    fn to_binder_object_or_ref(&self) -> gluon::ObjectOrRef {
        self.obj.clone()
    }
}
impl gluon::Liveness for ReparentHandle {
    fn alive(&self) -> bool {
        gluon::Liveness::alive(&self.obj)
    }
    fn death_notification(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        gluon::Liveness::death_notification(&self.obj)
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
