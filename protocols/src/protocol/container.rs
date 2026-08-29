#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
	protocol_name: "org.stardustxr.Container",
	types: &[],
};
pub mod proxies {
	use super::*;
}
#[derive(Debug, Clone)]
pub struct Container {
	obj: gluon::Ref,
}
impl gluon::Convertable for Container {
	fn write(&self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write(gluon_data)
	}
	fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
		let obj = gluon::Ref::read(gluon_data)?;
		Ok(Container::from_ref(obj))
	}
	fn write_owned(self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write_owned(gluon_data)
	}
}
impl gluon::Interface for Container {
	const ID: &'static str = "org.stardustxr.Container.Container";
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: ContainerHandler> gluon::HandledBy<H> for Container {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type ContainerLocal<H> = gluon::LocalRef<Container, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: ContainerHandler> From<ContainerLocal<H>> for Container {
	fn from(value: ContainerLocal<H>) -> Container {
		value.into_proxy()
	}
}
impl gluon::RefExt for Container {
	fn from_ref(obj: gluon::Ref) -> Container {
		Container { obj }
	}
}
impl Container {
	///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
	pub fn from_ref(obj: gluon::Ref) -> Container {
		Container { obj }
	}
}
impl From<Container> for gluon::Ref {
	fn from(value: Container) -> Self {
		value.obj
	}
}
impl gluon::ToRef for Container {
	fn to_ref(&self) -> gluon::Ref {
		self.obj.clone()
	}
}
impl gluon::Liveness for Container {
	fn death_notifier(&self) -> gluon::DeathNotifier {
		gluon::Liveness::death_notifier(&self.obj)
	}
}
impl std::hash::Hash for Container {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.obj.hash(state);
	}
}
impl PartialEq for Container {
	fn eq(&self, other: &Self) -> bool {
		self.obj == other.obj
	}
}
impl Eq for Container {}
pub trait ContainerHandler: gluon::Handler + Send + Sync + 'static {
	fn dispatch_one_way(
		&self,
		transaction_code: u32,
		mut gluon_data: gluon::DataReader,
		ctx: gluon::Context,
	) -> impl Future<Output = Result<(), gluon::SendError>> + Send + Sync {
		async move {
			match transaction_code {
				_ => {}
			}
			Ok(())
		}
	}
}
pub mod proxied {
	use super::*;
}
