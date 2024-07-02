#![allow(dead_code)]

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::Client,
	core::values::ResourceID,
	drawable::Model,
	fields::{Field, Shape},
	node::{NodeError, NodeType},
	root::{ClientState, FrameInfo, RootAspect, RootHandler},
	spatial::{Spatial, SpatialAspect, SpatialRefAspect, Transform},
};
use stardust_xr_molecules::{
	DebugSettings, Grabbable, GrabbableSettings, PointerMode, VisualDebug,
};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

lazy_static! {
	static ref GRABBABLE_MODEL: ResourceID = ResourceID::new_namespaced("molecules", "grabbable");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")])?;

	let _wrapped_root = client
		.get_root()
		.alias()
		.wrap(GrabbableDemo::new(&client).await?)?;

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

struct GrabbableDemo {
	root: Spatial,
	grabbable: Grabbable,
	field: Field,
	model: Model,
}
impl GrabbableDemo {
	async fn new(client: &Arc<Client>) -> Result<Self, NodeError> {
		let state_root = Spatial::create(client.get_root(), Transform::identity(), false)?;
		let model = Model::create(
			client.get_root(),
			Transform::from_scale([0.5; 3]),
			&*GRABBABLE_MODEL,
		)?;
		let bounds = model.get_relative_bounding_box(client.get_root()).await?;
		let field = Field::create(
			&model,
			Transform::from_translation(bounds.center),
			Shape::Box(bounds.size),
		)?;

		let mut grabbable = Grabbable::create(
			&state_root,
			Transform::none(),
			&field,
			GrabbableSettings {
				pointer_mode: PointerMode::Align,
				magnet: true,
				..Default::default()
			},
		)?;
		grabbable.set_debug(Some(DebugSettings::default()));
		model.set_spatial_parent(grabbable.content_parent())?;
		field.set_spatial_parent(grabbable.content_parent())?;

		if let Some(content_parent_reference) = client
			.get_state()
			.spatial_anchors(client)
			.get("content_parent")
		{
			grabbable
				.content_parent()
				.set_relative_transform(content_parent_reference, Transform::identity())
				.unwrap();
		}

		Ok(GrabbableDemo {
			root: state_root,
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
	fn save_state(&mut self) -> Result<ClientState> {
		self.root
			.set_relative_transform(
				self.grabbable.content_parent(),
				Transform::from_translation([0.0; 3]),
			)
			.unwrap();
		Ok(ClientState {
			data: None,
			root: self.root.node().get_id().unwrap(),
			spatial_anchors: [(
				"content_parent".to_string(),
				self.grabbable.content_parent().node().get_id().unwrap(),
			)]
			.into_iter()
			.collect(),
		})
	}
}
