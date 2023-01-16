#![allow(dead_code)]

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use manifest_dir_macros::directory_relative_path;
use mint::Vector3;
use stardust_xr_fusion::{
	client::{Client, LifeCycleHandler},
	core::values::Transform,
	drawable::{Model, ResourceID},
	fields::SphereField,
	node::NodeError,
};
use stardust_xr_molecules::{GrabData, Grabbable};

lazy_static! {
	static ref ICON_RESOURCE: ResourceID = ResourceID::new_namespaced("molecules", "urchin");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _wrapped_root = client.wrap_root(GrabbableDemo::new(&client)?);

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

struct GrabbableDemo {
	grabbable: Grabbable,
	field: SphereField,
	model: Model,
}
impl GrabbableDemo {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let field = SphereField::create(client.get_root(), mint::Vector3::from([0.0; 3]), 0.1)?;
		let grabbable = Grabbable::new(
			client.get_root(),
			Transform::default(),
			&field,
			GrabData { max_distance: 0.05 },
		)?;
		let model = Model::create(
			grabbable.content_parent(),
			Transform::from_scale(Vector3::from([0.1; 3])),
			&*ICON_RESOURCE,
		)?;
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
