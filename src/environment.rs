use gluon::Handler;
use stardust_xr_fusion::Result;
use stardust_xr_molecules_protocols::environment::EnvironmentHandler;

pub use stardust_xr_molecules_protocols::environment::Environment;

#[derive(Handler)]
pub struct EnvironmentObject;
impl EnvironmentHandler for EnvironmentObject {}
impl EnvironmentObject {
	#[allow(clippy::all)]
	pub fn new() -> Result<Environment> {
		Ok(EnvironmentObject.to_service()?.into_proxy())
	}
}
