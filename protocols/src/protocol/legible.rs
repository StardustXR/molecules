#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon_ipc::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon_ipc::ExternalProtocol = gluon_ipc::ExternalProtocol {
    protocol_name: "org.stardustxr.Legible",
    types: &[],
};
pub mod proxies {
    use super::*;
}
#[derive(Debug, Clone)]
pub struct Legible {
    obj: gluon_ipc::Ref,
}
impl gluon_ipc::Convertable for Legible {
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
        Ok(Legible::from_ref(obj))
    }
    fn write_owned(
        self,
        gluon_data: &mut gluon_ipc::DataBuilder,
    ) -> Result<(), gluon_ipc::WriteError> {
        self.obj.write_owned(gluon_data)
    }
}
impl Legible {
    const ID: &'static str = "org.stardustxr.Legible.Legible";
}
impl gluon_ipc::Interface for Legible {
    const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon_ipc::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: LegibleHandler> gluon_ipc::HandledBy<H> for Legible {}
///A proxy this process made, carrying the handler behind it — see [`gluon_ipc::LocalRef`]. Handed back by [`gluon_ipc::RefExt::new_node`] and [`gluon_ipc::RefExt::new_service`].
pub type LegibleLocal<H> = gluon_ipc::LocalRef<Legible, H>;
///Drops the handler share and keeps the proxy, so a [`gluon_ipc::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: LegibleHandler> From<LegibleLocal<H>> for Legible {
    fn from(value: LegibleLocal<H>) -> Legible {
        value.into_proxy()
    }
}
impl gluon_ipc::RefExt for Legible {
    fn from_ref(obj: gluon_ipc::Ref) -> Legible {
        Legible { obj }
    }
}
impl Legible {
    ///the text exactly as it's currently shown
    pub async fn text(&self) -> Result<String, gluon_ipc::SendError> {
        tracing::trace!(interface = "Legible", method = "text", "→");
        let mut gluon_builder = gluon_ipc::DataBuilder::new();
        let (mut gluon_recv, gluon_ret) = gluon_ipc::ReturnReceiver::new()?;
        gluon_builder.write_ref(&gluon_ret)?;
        gluon_ipc::transact(&self.obj, 8u32, gluon_builder)?;
        let mut reader = gluon_recv.recv().await.unwrap();
        let __ret_text = gluon_ipc::Convertable::read(&mut reader)?;
        tracing::trace!(interface = "Legible", method = "text", ? __ret_text, "←");
        Ok(__ret_text)
    }
    ///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
    pub fn from_ref(obj: gluon_ipc::Ref) -> Legible {
        Legible { obj }
    }
}
impl From<Legible> for gluon_ipc::Ref {
    fn from(value: Legible) -> Self {
        value.obj
    }
}
impl gluon_ipc::ToRef for Legible {
    fn to_ref(&self) -> gluon_ipc::Ref {
        self.obj.clone()
    }
}
impl gluon_ipc::Liveness for Legible {
    fn death_notifier(&self) -> gluon_ipc::DeathNotifier {
        gluon_ipc::Liveness::death_notifier(&self.obj)
    }
}
impl std::hash::Hash for Legible {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.hash(state);
    }
}
impl PartialEq for Legible {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}
impl Eq for Legible {}
pub trait LegibleHandler: gluon_ipc::Handler + Send + Sync + 'static {
    ///the text exactly as it's currently shown
    fn text(
        &self,
        _ctx: gluon_ipc::Context,
    ) -> impl Future<Output = String> + Send + Sync;
    ///Dispatched instead of [`Self::text`] so a slow reply doesn't hold up dispatch of the next transaction. The default implementation just awaits `text` and sends the result through `reply`. Override this method instead of `text` to defer the reply: stash `reply` (it's `Send + Sync + 'static`) somewhere else — a channel, a queue, another task — and return as soon as this method's future is done, without waiting for the reply to actually be sent.
    fn text_oneway(
        &self,
        _ctx: gluon_ipc::Context,
        reply: gluon_ipc::ReplySender<String>,
    ) -> impl Future<Output = Result<(), gluon_ipc::SendError>> + Send + Sync {
        async move {
            let text = self.text(_ctx).await;
            reply.send(text)
        }
    }
    fn dispatch_one_way(
        &self,
        transaction_code: u32,
        mut gluon_data: gluon_ipc::DataReader,
        ctx: gluon_ipc::Context,
    ) -> impl Future<Output = Result<(), gluon_ipc::SendError>> + Send + Sync {
        async move {
            match transaction_code {
                8u32 => {
                    let return_callback = gluon_data.read_ref()?;
                    tracing::trace!(
                        interface = "Legible", method = "text", "dispatching"
                    );
                    drop(gluon_data);
                    let reply: gluon_ipc::ReplySender<String> = gluon_ipc::ReplySender::new(
                        return_callback,
                        |text, gluon_out| {
                            tracing::trace!(
                                interface = "Legible", method = "text", ? text, "←"
                            );
                            text.write_owned(gluon_out)?;
                            Ok(())
                        },
                    );
                    self.text_oneway(ctx, reply)
                        .instrument(
                            tracing::trace_span!(
                                "dispatching", interface = "Legible", method = "text",
                                method_id = 8u32
                            ),
                        )
                        .await?;
                }
                _ => {}
            }
            Ok(())
        }
    }
    fn to_node(
        self,
    ) -> Result<
        (gluon_ipc::Node<Self>, gluon_ipc::LocalRef<Legible, Self>),
        gluon_ipc::NodeError,
    >
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Legible::new_node(self)
    }
    fn to_service(
        self,
    ) -> Result<gluon_ipc::LocalRef<Legible, Self>, gluon_ipc::NodeError>
    where
        Self: Sized,
    {
        use gluon_ipc::RefExt;
        Legible::new_service(self)
    }
}
pub mod proxied {
    use super::*;
}
