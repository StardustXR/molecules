#![allow(unused, clippy::all, private_bounds, private_interfaces)]
use gluon::Convertable as _;
use tracing::Instrument as _;
pub const EXTERNAL_PROTOCOL: gluon::ExternalProtocol = gluon::ExternalProtocol {
	protocol_name: "org.stardustxr.Environment",
	types: &[],
};
pub mod proxies {
	use super::*;
}
#[derive(Debug, Clone)]
pub struct Environment {
	obj: gluon::Ref,
}
impl gluon::Convertable for Environment {
	fn write(&self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write(gluon_data)
	}
	fn read(gluon_data: &mut gluon::DataReader) -> Result<Self, gluon::ReadError> {
		let obj = gluon::Ref::read(gluon_data)?;
		Ok(Environment::from_ref(obj))
	}
	fn write_owned(self, gluon_data: &mut gluon::DataBuilder) -> Result<(), gluon::WriteError> {
		self.obj.write_owned(gluon_data)
	}
}
impl Environment {
	const ID: &'static str = "org.stardustxr.Environment.Environment";
}
impl gluon::Interface for Environment {
	const ID: &'static str = Self::ID;
}
///Carries the per-interface bound for [`gluon::RefExt`]'s handler constructors: only a handler implementing this interface's handler trait can be passed to them.
impl<H: EnvironmentHandler> gluon::HandledBy<H> for Environment {}
///A proxy this process made, carrying the handler behind it — see [`gluon::LocalRef`]. Handed back by [`gluon::RefExt::new_node`] and [`gluon::RefExt::new_service`].
pub type EnvironmentLocal<H> = gluon::LocalRef<Environment, H>;
///Drops the handler share and keeps the proxy, so a [`gluon::LocalRef`] goes anywhere this proxy does — including the `impl Into<Self>` parameters generated for typed refs.
impl<H: EnvironmentHandler> From<EnvironmentLocal<H>> for Environment {
	fn from(value: EnvironmentLocal<H>) -> Environment {
		value.into_proxy()
	}
}
impl gluon::RefExt for Environment {
	fn from_ref(obj: gluon::Ref) -> Environment {
		Environment { obj }
	}
}
impl Environment {
	///only use this when you know the ref leads to something implementing this interface, else the consquences are for you to find out
	pub fn from_ref(obj: gluon::Ref) -> Environment {
		Environment { obj }
	}
}
impl From<Environment> for gluon::Ref {
	fn from(value: Environment) -> Self {
		value.obj
	}
}
impl gluon::ToRef for Environment {
	fn to_ref(&self) -> gluon::Ref {
		self.obj.clone()
	}
}
impl gluon::Liveness for Environment {
	fn death_notifier(&self) -> gluon::DeathNotifier {
		gluon::Liveness::death_notifier(&self.obj)
	}
}
impl std::hash::Hash for Environment {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.obj.hash(state);
	}
}
impl PartialEq for Environment {
	fn eq(&self, other: &Self) -> bool {
		self.obj == other.obj
	}
}
impl Eq for Environment {}
pub trait EnvironmentHandler: gluon::Handler + Send + Sync + 'static {
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
	fn to_node(
		self,
	) -> Result<(gluon::Node<Self>, gluon::LocalRef<Environment, Self>), gluon::NodeError>
	where
		Self: Sized,
	{
		use gluon::RefExt;
		Environment::new_node(self)
	}
	fn to_service(self) -> Result<gluon::LocalRef<Environment, Self>, gluon::NodeError>
	where
		Self: Sized,
	{
		use gluon::RefExt;
		Environment::new_service(self)
	}
}
pub mod proxied {
	use super::*;
}
