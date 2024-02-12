#![allow(dead_code)]

use color::rgba_linear;
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::{Client, ClientState, FrameInfo, RootHandler},
	core::values::ResourceID,
	drawable::{Line, Lines, Model},
	fields::BoxField,
	node::{NodeError, NodeType},
	spatial::{SpatialAspect, Transform},
};
use stardust_xr_molecules::{
	lines::{bounding_box, make_line_points},
	Grabbable, GrabbableSettings, PointerMode,
};
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
	bounding_box: Lines,
}
impl GrabbableDemo {
	async fn new(client: &Client) -> Result<Self, NodeError> {
		let model = Model::create(
			client.get_root(),
			Transform::from_scale([0.5; 3]),
			&*GRABBABLE_MODEL,
		)?;
		let bounds = model.get_relative_bounding_box(client.get_root()).await?;
		let bounding_lines: Vec<Line> = bounding_box(bounds.clone())
			.into_iter()
			.map(|l| Line {
				points: make_line_points(l, 0.001, rgba_linear!(1.0, 1.0, 1.0, 1.0)),
				cyclic: true,
			})
			.collect();
		let bounding_box = Lines::create(&model, Transform::identity(), &bounding_lines)?;
		let field = BoxField::create(
			&model,
			Transform::from_translation(bounds.center),
			bounds.size,
		)?;

		let grabbable = Grabbable::create(
			client.get_root(),
			Transform::none(),
			&field,
			GrabbableSettings {
				pointer_mode: PointerMode::Align,
				..Default::default()
			},
		)?;
		model.set_spatial_parent(grabbable.content_parent())?;
		field.set_spatial_parent(grabbable.content_parent())?;
		bounding_box.set_spatial_parent(grabbable.content_parent())?;

		Ok(GrabbableDemo {
			grabbable,
			field,
			model,
			bounding_box,
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
