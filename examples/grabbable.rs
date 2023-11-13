#![allow(dead_code)]

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::{Client, ClientState, FrameInfo, RootHandler},
	core::values::Transform,
	drawable::{Model, ResourceID},
	fields::BoxField,
	node::{NodeError, NodeType},
};
use stardust_xr_molecules::{Grabbable, GrabbableSettings, PointerMode};
use tracing_subscriber::EnvFilter;

lazy_static! {
	static ref GRABBABLE_MODEL: ResourceID = ResourceID::new_namespaced("molecules", "grabbable");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::fmt()
		.compact()
		.with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
		.init();
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _wrapped_root = client.wrap_root(GrabbableDemo::new(&client).await?)?;

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

struct GrabbableDemo {
	grabbable: Grabbable,
	field: BoxField,
	model: Model,
}
impl GrabbableDemo {
	async fn new(client: &Client) -> Result<Self, NodeError> {
		let model = Model::create(
			client.get_root(),
			Transform::from_scale([0.5; 3]),
			&*GRABBABLE_MODEL,
		)?;
		let bounds = model.get_bounding_box(Some(client.get_root()))?.await?;
		let field = BoxField::create(
			&client.get_root(),
			Transform::from_position(bounds.center),
			bounds.size,
		)?;

		let grabbable = Grabbable::create(
			client.get_root(),
			Transform::none(),
			&field,
			GrabbableSettings {
				linear_momentum: None,
				angular_momentum: None,
				magnet: false,
				pointer_mode: PointerMode::Align,
				..Default::default()
			},
		)?;
		model.set_spatial_parent(grabbable.content_parent())?;
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
