#![allow(dead_code)]

use glam::vec3;
use lazy_static::lazy_static;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::{Client, LifeCycleHandler},
	drawable::Model,
	fields::SphereField,
	node::NodeError,
	resource::Resource,
};
use stardust_xr_molecules::Grabbable;

lazy_static! {
	static ref ICON_RESOURCE: Resource = Resource::new("molecules", "urchin.glb");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")])?;

	let _wrapped_root = client.wrap_root(GrabbableDemo::new(&client)?);

	tokio::select! {
		_ = tokio::signal::ctrl_c() => Ok(()),
		_ = event_loop => Err(anyhow::anyhow!("Server crashed")),
	}
}

struct GrabbableDemo {
	grabbable: Grabbable,
	field: SphereField,
	model: Model,
}
impl GrabbableDemo {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let field = SphereField::builder()
			.spatial_parent(client.get_root())
			.radius(0.1)
			.build()?;
		let grabbable = Grabbable::new(client.get_root(), &field)?;
		let model = Model::resource_builder()
			.spatial_parent(grabbable.content_parent())
			.resource(&ICON_RESOURCE)
			.scale(vec3(0.1, 0.1, 0.1))
			.build()?;
		field.set_spatial_parent(grabbable.content_parent())?;

		Ok(GrabbableDemo {
			grabbable,
			field,
			model,
		})
	}
}
impl LifeCycleHandler for GrabbableDemo {
	fn logic_step(&mut self, _info: stardust_xr_fusion::client::LogicStepInfo) {
		self.grabbable.update();
	}
}
