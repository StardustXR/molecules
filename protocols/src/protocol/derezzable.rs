#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon_ipc::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon_ipc::ExternalProtocol = gluon_ipc::ExternalProtocol {
    protocol_name: "org.stardustxr.Derezzable",
    types: &[],
};
pub mod proxies {
    use super::*;
}
#[derive(Debug, Clone)]
pub struct Derezzable {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Derezzable {
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
        Ok(Derezzable::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Derezzable {
    const ID: &'static str = "org.stardustxr.Derezzable.Derezzable";
}
impl gluon_ipc::Interface for Derezzable {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: DerezzableHandler> gluon_ipc::HandledBy<H> for Derezzable {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type DerezzableLocal<H> = gluon_ipc::LocalRef<Derezzable, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: DerezzableHandler> From<DerezzableLocal<H>> for Derezzable {
    fn from(value: DerezzableLocal<H>) -> Derezzable {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Derezzable {
    fn from_ref(obj: gluon_ipc::Ref) -> Derezzable {
        Derezzable { obj }
    }
}
impl Derezzable {
    pub fn derez(&self) -> Result<(), gluon_ipc::SendError> {
        tracing::trace!(interface = "Derezzable", method = "derez", "→");
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        Ok(())
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Derezzable {
        Derezzable { obj }
    }
}
impl From<Derezzable> for gluon_ipc::Ref {
    fn from(value: Derezzable) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Derezzable {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Derezzable {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Derezzable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Derezzable {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Derezzable {}
pub trait DerezzableHandler: gluon_ipc::Handler + Send + Sync + 'static {
    fn derez(&self, _ctx: gluon_ipc::Context) -> impl Future<Output = ()> + Send + Sync;
    fn dispatch_one_way(
        &self,
        transaction_code: u32,
        mut gluon_data: gluon_ipc::DataReader,
        ctx: gluon_ipc::Context,
    ) -> impl Future<Output = Result<(), gluon_ipc::SendError>> + Send + Sync {
        async move {
            match transaction_code {
                8u32 => {
                    tracing::trace!(
                        interface = "Derezzable", method = "derez", "dispatching"
                    );
                    drop(gluon_data);
                    self.derez(ctx)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Derezzable", method = "derez",
                                method_id = 8u32
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
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Derezzable, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Derezzable::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Derezzable, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Derezzable::new_service(self)
    }
}
pub mod proxied {
    use super::*;
}
