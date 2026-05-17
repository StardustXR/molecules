#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
    protocol_name: "org.stardustxr.Reparentable",
    types: &[],
};
pub mod proxies {
    use super::*;
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
impl Reparentable {
    ///Reparents this object, locking this Reparentable to make sure others can't steal this reparent, can steal from non-locking reparents
    pub async fn reparent_locking(
        &self,
        new_parent: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        keepalive: impl Into<ReparentKeepalive>,
    ) -> Result<Option<ReparentHandle>, gluon::SendError> {
        let new_parent: stardust_xr_protocol::spatial::SpatialRef = new_parent.into();
        let keepalive: ReparentKeepalive = keepalive.into();
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        gluon_builder.write_binder(&gluon_ret)?;
        new_parent.write(&mut gluon_builder)?;
        keepalive.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        let transaction = gluon_recv.recv().await.unwrap();
        let mut reader = gluon::DataReader::from_payload(transaction.payload);
        Ok(gluon::Convertable::read(&mut reader)?)
    }
    ///Reparents this object, this is non-locking, others can steal this reparent
    pub async fn reparent(
        &self,
        new_parent: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        keepalive: impl Into<ReparentKeepalive>,
    ) -> Result<Option<ReparentHandle>, gluon::SendError> {
        let new_parent: stardust_xr_protocol::spatial::SpatialRef = new_parent.into();
        let keepalive: ReparentKeepalive = keepalive.into();
        let mut gluon_builder = gluon::DataBuilder::new();
        let (gluon_ret_handler, mut gluon_recv) = gluon::ReturnHandler::new();
        let gluon_ret = self.obj.device().register_object(gluon_ret_handler);
        gluon_builder.write_binder(&gluon_ret)?;
        new_parent.write(&mut gluon_builder)?;
        keepalive.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 9u32, gluon_builder.to_payload())?;
        let transaction = gluon_recv.recv().await.unwrap();
        let mut reader = gluon::DataReader::from_payload(transaction.payload);
        Ok(gluon::Convertable::read(&mut reader)?)
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
    ///Reparents this object, locking this Reparentable to make sure others can't steal this reparent, can steal from non-locking reparents
    fn reparent_locking(
        &self,
        _ctx: gluon::Context,
        new_parent: stardust_xr_protocol::spatial::SpatialRef,
        keepalive: ReparentKeepalive,
    ) -> impl Future<Output = Option<ReparentHandle>> + Send + Sync;
    ///Reparents this object, this is non-locking, others can steal this reparent
    fn reparent(
        &self,
        _ctx: gluon::Context,
        new_parent: stardust_xr_protocol::spatial::SpatialRef,
        keepalive: ReparentKeepalive,
    ) -> impl Future<Output = Option<ReparentHandle>> + Send + Sync;
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
                    let mut gluon_out = gluon::DataBuilder::new();
                    let param_new_parent = gluon::Convertable::read(&mut gluon_data)?;
                    let param_keepalive = gluon::Convertable::read(&mut gluon_data)?;
                    let (handle) = self
                        .reparent_locking(ctx, param_new_parent, param_keepalive)
                        .await;
                    drop(gluon_data);
                    handle.write_owned(&mut gluon_out)?;
                    return_callback
                        .device()
                        .transact_one_way(&return_callback, 0, gluon_out.to_payload())?;
                }
                9u32 => {
                    let return_callback = gluon_data.read_binder()?;
                    let mut gluon_out = gluon::DataBuilder::new();
                    let param_new_parent = gluon::Convertable::read(&mut gluon_data)?;
                    let param_keepalive = gluon::Convertable::read(&mut gluon_data)?;
                    let (handle) = self
                        .reparent(ctx, param_new_parent, param_keepalive)
                        .await;
                    drop(gluon_data);
                    handle.write_owned(&mut gluon_out)?;
                    return_callback
                        .device()
                        .transact_one_way(&return_callback, 0, gluon_out.to_payload())?;
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
impl ReparentKeepalive {
    ///The reparent this object was associated with was stolen, the ReparentHandle becomes invalid
    pub fn reparent_stolen(&self) -> Result<(), gluon::SendError> {
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
                    drop(gluon_data);
                    self.reparent_stolen(ctx).await;
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
impl ReparentHandle {
    ///Set transform relative to the given SpatialRef to IDENTITY
    pub fn reset_transform(
        &self,
        relative_to: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
    ) -> Result<(), gluon::SendError> {
        let relative_to: stardust_xr_protocol::spatial::SpatialRef = relative_to.into();
        let mut gluon_builder = gluon::DataBuilder::new();
        relative_to.write(&mut gluon_builder)?;
        self.obj.device().transact_one_way(&self.obj, 8u32, gluon_builder.to_payload())?;
        Ok(())
    }
    ///Finish reparenting, invalidates handle
    pub fn unparent(&self) -> Result<(), gluon::SendError> {
        let mut gluon_builder = gluon::DataBuilder::new();
        self.obj.device().transact_one_way(&self.obj, 9u32, gluon_builder.to_payload())?;
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
    ///Finish reparenting, invalidates handle
    fn unparent(&self, _ctx: gluon::Context) -> impl Future<Output = ()> + Send + Sync;
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
                    drop(gluon_data);
                    self.reset_transform(ctx, param_relative_to).await;
                }
                9u32 => {
                    drop(gluon_data);
                    self.unparent(ctx).await;
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
