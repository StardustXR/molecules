#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon_ipc::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon_ipc::ExternalProtocol = gluon_ipc::ExternalProtocol {
    protocol_name: "org.stardustxr.Transformable",
    types: &[],
};
pub mod proxies {
    use super::*;
}
#[derive(Debug, Clone)]
pub struct Transformable {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Transformable {
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
        Ok(Transformable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Transformable {
    const ID: &'static str = "org.stardustxr.Transformable.Transformable";
}
impl gluon_ipc::Interface for Transformable {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: TransformableHandler> gluon_ipc::HandledBy<H> for Transformable {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type TransformableLocal<H> = gluon_ipc::LocalRef<Transformable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: TransformableHandler> From<TransformableLocal<H>> for Transformable {
    fn from(value: TransformableLocal<H>) -> Transformable {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Transformable {
    fn from_ref(obj: gluon_ipc::Ref) -> Transformable {
        Transformable { obj }
    }
}
impl Transformable {
    ///Transform this object by this partial transform relative to the provided spatialref (adds to the existing transform)
    pub fn offset_relative_transform(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        offset_transform: impl Into<stardust_xr_protocol::spatial::PartialTransform>,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let offset_transform: stardust_xr_protocol::spatial::PartialTransform = offset_transform
            .into();
        tracing::trace!(
            interface = "Transformable", method = "offset_relative_transform", ?
            reference, ? offset_transform, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        offset_transform.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///Set the transform this object relative to the provided spatialref
    pub fn set_relative_transform(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        transform: impl Into<stardust_xr_protocol::spatial::PartialTransform>,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let transform: stardust_xr_protocol::spatial::PartialTransform = transform
            .into();
        tracing::trace!(
            interface = "Transformable", method = "set_relative_transform", ? reference,
            ? transform, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        transform.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 9u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Transformable {
        Transformable { obj }
    }
}
impl From<Transformable> for gluon_ipc::Ref {
    fn from(value: Transformable) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Transformable {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Transformable {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Transformable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Transformable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Transformable {}
pub trait TransformableHandler: gluon_ipc::Handler + Send + Sync + 'static {
    ///Transform this object by this partial transform relative to the provided spatialref (adds to the existing transform)
    fn offset_relative_transform(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        offset_transform: stardust_xr_protocol::spatial::PartialTransform,
    ) -> impl Future<Output = ()> + Send + Sync;
    ///Set the transform this object relative to the provided spatialref
    fn set_relative_transform(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        transform: stardust_xr_protocol::spatial::PartialTransform,
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
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let param_offset_transform = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Transformable", method =
                        "offset_relative_transform", ? param_reference, ?
                        param_offset_transform, "dispatching"
                    );
                    drop(gluon_data);
                    self.offset_relative_transform(
                            ctx,
                            param_reference,
                            param_offset_transform,
                        )
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Transformable", method =
                                "offset_relative_transform", method_id = 8u32
                            ),
                        )
                        .await;
                }
                9u32 => {
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let param_transform = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "Transformable", method = "set_relative_transform", ?
                        param_reference, ? param_transform, "dispatching"
                    );
                    drop(gluon_data);
                    self.set_relative_transform(ctx, param_reference, param_transform)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Transformable", method =
                                "set_relative_transform", method_id = 9u32
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Transformable, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Transformable::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Transformable, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Transformable::new_service(self)
    }
}
#[derive(Debug, Clone)]
pub struct Translatable {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Translatable {
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
        Ok(Translatable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Translatable {
    const ID: &'static str = "org.stardustxr.Transformable.Translatable";
}
impl gluon_ipc::Interface for Translatable {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: TranslatableHandler> gluon_ipc::HandledBy<H> for Translatable {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type TranslatableLocal<H> = gluon_ipc::LocalRef<Translatable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: TranslatableHandler> From<TranslatableLocal<H>> for Translatable {
    fn from(value: TranslatableLocal<H>) -> Translatable {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Translatable {
    fn from_ref(obj: gluon_ipc::Ref) -> Translatable {
        Translatable { obj }
    }
}
impl Translatable {
    ///Move this object by this offset relative to the provided spatialref (adds to the existing transform)
    pub fn offset_relative_translation(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        offset: stardust_xr_protocol::types::proxies::Vec3F,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let offset: stardust_xr_protocol::types::proxied::Vec3F = offset.into();
        tracing::trace!(
            interface = "Translatable", method = "offset_relative_translation", ?
            reference, ? offset, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        offset.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///Set the translation of this object relative to the provided spatialref
    pub fn set_relative_translation(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        translation: stardust_xr_protocol::types::proxies::Vec3F,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let translation: stardust_xr_protocol::types::proxied::Vec3F = translation
            .into();
        tracing::trace!(
            interface = "Translatable", method = "set_relative_translation", ? reference,
            ? translation, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        translation.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 9u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Translatable {
        Translatable { obj }
    }
}
impl From<Translatable> for gluon_ipc::Ref {
    fn from(value: Translatable) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Translatable {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Translatable {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Translatable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Translatable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Translatable {}
pub trait TranslatableHandler: gluon_ipc::Handler + Send + Sync + 'static {
    ///Move this object by this offset relative to the provided spatialref (adds to the existing transform)
    fn offset_relative_translation(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        offset: stardust_xr_protocol::types::proxies::Vec3F,
    ) -> impl Future<Output = ()> + Send + Sync;
    ///Set the translation of this object relative to the provided spatialref
    fn set_relative_translation(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        translation: stardust_xr_protocol::types::proxies::Vec3F,
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
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let __wire_param_offset: stardust_xr_protocol::types::proxied::Vec3F = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Translatable", method =
                        "offset_relative_translation", ? param_reference, param_offset =
                        ? __wire_param_offset, "dispatching"
                    );
                    let param_offset: stardust_xr_protocol::types::proxies::Vec3F = {
                        let __w = __wire_param_offset;
                        __w.into()
                    };
                    drop(gluon_data);
                    self.offset_relative_translation(ctx, param_reference, param_offset)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Translatable", method =
                                "offset_relative_translation", method_id = 8u32
                            ),
                        )
                        .await;
                }
                9u32 => {
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let __wire_param_translation: stardust_xr_protocol::types::proxied::Vec3F = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Translatable", method = "set_relative_translation",
                        ? param_reference, param_translation = ?
                        __wire_param_translation, "dispatching"
                    );
                    let param_translation: stardust_xr_protocol::types::proxies::Vec3F = {
                        let __w = __wire_param_translation;
                        __w.into()
                    };
                    drop(gluon_data);
                    self.set_relative_translation(
                            ctx,
                            param_reference,
                            param_translation,
                        )
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Translatable", method =
                                "set_relative_translation", method_id = 9u32
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Translatable, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Translatable::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Translatable, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Translatable::new_service(self)
    }
}
#[derive(Debug, Clone)]
pub struct Rotatable {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Rotatable {
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
        Ok(Rotatable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Rotatable {
    const ID: &'static str = "org.stardustxr.Transformable.Rotatable";
}
impl gluon_ipc::Interface for Rotatable {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: RotatableHandler> gluon_ipc::HandledBy<H> for Rotatable {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type RotatableLocal<H> = gluon_ipc::LocalRef<Rotatable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: RotatableHandler> From<RotatableLocal<H>> for Rotatable {
    fn from(value: RotatableLocal<H>) -> Rotatable {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Rotatable {
    fn from_ref(obj: gluon_ipc::Ref) -> Rotatable {
        Rotatable { obj }
    }
}
impl Rotatable {
    ///Rotate this object by this offset relative to the provided spatialref (adds to the existing transform)
    pub fn offset_relative_rotation(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        offset: stardust_xr_protocol::types::proxies::QuatF,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let offset: stardust_xr_protocol::types::proxied::Quatf = offset.into();
        tracing::trace!(
            interface = "Rotatable", method = "offset_relative_rotation", ? reference, ?
            offset, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        offset.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///Set the rotation of this object relative to the provided spatialref
    pub fn set_relative_rotation(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        rotation: stardust_xr_protocol::types::proxies::QuatF,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let rotation: stardust_xr_protocol::types::proxied::Quatf = rotation.into();
        tracing::trace!(
            interface = "Rotatable", method = "set_relative_rotation", ? reference, ?
            rotation, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        rotation.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 9u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Rotatable {
        Rotatable { obj }
    }
}
impl From<Rotatable> for gluon_ipc::Ref {
    fn from(value: Rotatable) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Rotatable {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Rotatable {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Rotatable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Rotatable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Rotatable {}
pub trait RotatableHandler: gluon_ipc::Handler + Send + Sync + 'static {
    ///Rotate this object by this offset relative to the provided spatialref (adds to the existing transform)
    fn offset_relative_rotation(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        offset: stardust_xr_protocol::types::proxies::QuatF,
    ) -> impl Future<Output = ()> + Send + Sync;
    ///Set the rotation of this object relative to the provided spatialref
    fn set_relative_rotation(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        rotation: stardust_xr_protocol::types::proxies::QuatF,
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
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let __wire_param_offset: stardust_xr_protocol::types::proxied::Quatf = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Rotatable", method = "offset_relative_rotation", ?
                        param_reference, param_offset = ? __wire_param_offset,
                        "dispatching"
                    );
                    let param_offset: stardust_xr_protocol::types::proxies::QuatF = {
                        let __w = __wire_param_offset;
                        __w.into()
                    };
                    drop(gluon_data);
                    self.offset_relative_rotation(ctx, param_reference, param_offset)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Rotatable", method =
                                "offset_relative_rotation", method_id = 8u32
                            ),
                        )
                        .await;
                }
                9u32 => {
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let __wire_param_rotation: stardust_xr_protocol::types::proxied::Quatf = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Rotatable", method = "set_relative_rotation", ?
                        param_reference, param_rotation = ? __wire_param_rotation,
                        "dispatching"
                    );
                    let param_rotation: stardust_xr_protocol::types::proxies::QuatF = {
                        let __w = __wire_param_rotation;
                        __w.into()
                    };
                    drop(gluon_data);
                    self.set_relative_rotation(ctx, param_reference, param_rotation)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Rotatable", method =
                                "set_relative_rotation", method_id = 9u32
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Rotatable, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Rotatable::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Rotatable, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Rotatable::new_service(self)
    }
}
#[derive(Debug, Clone)]
pub struct Scalable {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Scalable {
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
        Ok(Scalable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Scalable {
    const ID: &'static str = "org.stardustxr.Transformable.Scalable";
}
impl gluon_ipc::Interface for Scalable {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: ScalableHandler> gluon_ipc::HandledBy<H> for Scalable {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type ScalableLocal<H> = gluon_ipc::LocalRef<Scalable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: ScalableHandler> From<ScalableLocal<H>> for Scalable {
    fn from(value: ScalableLocal<H>) -> Scalable {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Scalable {
    fn from_ref(obj: gluon_ipc::Ref) -> Scalable {
        Scalable { obj }
    }
}
impl Scalable {
    ///Scale this object by this offset relative to the provided spatialref (adds to the existing transform)
    pub fn offset_relative_scale(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        offset: stardust_xr_protocol::types::proxies::Vec3F,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let offset: stardust_xr_protocol::types::proxied::Vec3F = offset.into();
        tracing::trace!(
            interface = "Scalable", method = "offset_relative_scale", ? reference, ?
            offset, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        offset.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///Set the scale of this object relative to the provided spatialref
    pub fn set_relative_scale(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        scale: stardust_xr_protocol::types::proxies::Vec3F,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let scale: stardust_xr_protocol::types::proxied::Vec3F = scale.into();
        tracing::trace!(
            interface = "Scalable", method = "set_relative_scale", ? reference, ? scale,
            "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        scale.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 9u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Scalable {
        Scalable { obj }
    }
}
impl From<Scalable> for gluon_ipc::Ref {
    fn from(value: Scalable) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Scalable {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Scalable {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Scalable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Scalable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Scalable {}
pub trait ScalableHandler: gluon_ipc::Handler + Send + Sync + 'static {
    ///Scale this object by this offset relative to the provided spatialref (adds to the existing transform)
    fn offset_relative_scale(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        offset: stardust_xr_protocol::types::proxies::Vec3F,
    ) -> impl Future<Output = ()> + Send + Sync;
    ///Set the scale of this object relative to the provided spatialref
    fn set_relative_scale(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        scale: stardust_xr_protocol::types::proxies::Vec3F,
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
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let __wire_param_offset: stardust_xr_protocol::types::proxied::Vec3F = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Scalable", method = "offset_relative_scale", ?
                        param_reference, param_offset = ? __wire_param_offset,
                        "dispatching"
                    );
                    let param_offset: stardust_xr_protocol::types::proxies::Vec3F = {
                        let __w = __wire_param_offset;
                        __w.into()
                    };
                    drop(gluon_data);
                    self.offset_relative_scale(ctx, param_reference, param_offset)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Scalable", method =
                                "offset_relative_scale", method_id = 8u32
                            ),
                        )
                        .await;
                }
                9u32 => {
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let __wire_param_scale: stardust_xr_protocol::types::proxied::Vec3F = gluon_ipc::Convertable::read(
                        &mut gluon_data,
                    )?;
                    tracing::trace!(
                        interface = "Scalable", method = "set_relative_scale", ?
                        param_reference, param_scale = ? __wire_param_scale,
                        "dispatching"
                    );
                    let param_scale: stardust_xr_protocol::types::proxies::Vec3F = {
                        let __w = __wire_param_scale;
                        __w.into()
                    };
                    drop(gluon_data);
                    self.set_relative_scale(ctx, param_reference, param_scale)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Scalable", method =
                                "set_relative_scale", method_id = 9u32
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Scalable, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Scalable::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Scalable, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Scalable::new_service(self)
    }
}
#[derive(Debug, Clone)]
pub struct Poseable {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Poseable {
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
        Ok(Poseable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Poseable {
    const ID: &'static str = "org.stardustxr.Transformable.Poseable";
}
impl gluon_ipc::Interface for Poseable {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: PoseableHandler> gluon_ipc::HandledBy<H> for Poseable {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type PoseableLocal<H> = gluon_ipc::LocalRef<Poseable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: PoseableHandler> From<PoseableLocal<H>> for Poseable {
    fn from(value: PoseableLocal<H>) -> Poseable {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Poseable {
    fn from_ref(obj: gluon_ipc::Ref) -> Poseable {
        Poseable { obj }
    }
}
impl Poseable {
    ///Offset the pose of this object by this offset relative to the provided spatialref (adds to the existing transform)
    pub fn offset_relative_pse(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        offset: impl Into<stardust_xr_protocol::types::Posef>,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let offset: stardust_xr_protocol::types::Posef = offset.into();
        tracing::trace!(
            interface = "Poseable", method = "offset_relative_pse", ? reference, ?
            offset, "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        offset.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///Set the pose of this object relative to the provided spatialref
    pub fn set_relative_pose(
        &self,
        reference: impl Into<stardust_xr_protocol::spatial::SpatialRef>,
        pose: impl Into<stardust_xr_protocol::types::Posef>,
    ) -> Result<(), gluon_ipc::SendError> {
        let reference: stardust_xr_protocol::spatial::SpatialRef = reference.into();
        let pose: stardust_xr_protocol::types::Posef = pose.into();
        tracing::trace!(
            interface = "Poseable", method = "set_relative_pose", ? reference, ? pose,
            "→"
        );
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        reference.write(&mut gluon_builder)?;
        pose.write(&mut gluon_builder)?;
        gluon_ipc::transact(&self.obj, 9u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Poseable {
        Poseable { obj }
    }
}
impl From<Poseable> for gluon_ipc::Ref {
    fn from(value: Poseable) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Poseable {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Poseable {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Poseable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Poseable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Poseable {}
pub trait PoseableHandler: gluon_ipc::Handler + Send + Sync + 'static {
    ///Offset the pose of this object by this offset relative to the provided spatialref (adds to the existing transform)
    fn offset_relative_pse(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        offset: stardust_xr_protocol::types::Posef,
    ) -> impl Future<Output = ()> + Send + Sync;
    ///Set the pose of this object relative to the provided spatialref
    fn set_relative_pose(
        &self,
        _ctx: gluon_ipc::Context,
        reference: stardust_xr_protocol::spatial::SpatialRef,
        pose: stardust_xr_protocol::types::Posef,
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
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let param_offset = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "Poseable", method = "offset_relative_pse", ?
                        param_reference, ? param_offset, "dispatching"
                    );
                    drop(gluon_data);
                    self.offset_relative_pse(ctx, param_reference, param_offset)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Poseable", method =
                                "offset_relative_pse", method_id = 8u32
                            ),
                        )
                        .await;
                }
                9u32 => {
                    let param_reference = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    let param_pose = gluon_ipc::Convertable::read(&mut gluon_data)?;
                    tracing::trace!(
                        interface = "Poseable", method = "set_relative_pose", ?
                        param_reference, ? param_pose, "dispatching"
                    );
                    drop(gluon_data);
                    self.set_relative_pose(ctx, param_reference, param_pose)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Poseable", method =
                                "set_relative_pose", method_id = 9u32
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Poseable, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Poseable::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Poseable, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Poseable::new_service(self)
    }
}
pub mod proxied {
    use super::*;
}
