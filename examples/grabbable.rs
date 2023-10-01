#![allow(dead_code)]

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::{Client, ClientState, FrameInfo, RootHandler},
	core::values::Transform,
	drawable::{Model, ResourceID},
	fields::SphereField,
	node::{NodeError, NodeType},
};
use stardust_xr_molecules::{Grabbable, GrabbableSettings};

lazy_static! {
	static ref ICON_RESOURCE: ResourceID = ResourceID::new_namespaced("molecules", "urchin");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _wrapped_root = client.wrap_root(GrabbableDemo::new(&client)?)?;

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
		let field = SphereField::create(client.get_root(), [0.0; 3], 0.1)?;
		let grabbable = Grabbable::create(
			client.get_root(),
			Transform::none(),
			&field,
			GrabbableSettings::default(),
		)?;
		let model = Model::create(
			grabbable.content_parent(),
			Transform::from_scale([0.1; 3]),
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
impl RootHandler for GrabbableDemo {
	fn frame(&mut self, info: FrameInfo) {
		self.grabbable.update(&info).unwrap();
	}
	fn save_state(&mut self) -> ClientState {
		ClientState {
			root: Some(self.grabbable.content_parent().alias()),
			..Default::default()
		}
	}
}
