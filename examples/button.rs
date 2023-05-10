#![allow(dead_code)]

use color_eyre::eyre::Result;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::{
	client::{Client, FrameInfo, RootHandler},
	core::values::Transform,
	drawable::{MaterialParameter, Model, ModelPart, ResourceID},
	node::NodeError,
};
use stardust_xr_molecules::{touch_plane::TouchPlane, DebugSettings, VisualDebug};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _wrapped_root = client.wrap_root(ButtonDemo::new(&client)?)?;

	tokio::select! {
		_ = tokio::signal::ctrl_c() => (),
		e = event_loop => e??,
	}
	Ok(())
}

struct ButtonDemo {
	touch_plane: TouchPlane,
	model: Model,
	button_part: ModelPart,
}
impl ButtonDemo {
	fn new(client: &Client) -> Result<Self, NodeError> {
		let mut touch_plane = TouchPlane::create(
			client.get_root(),
			Transform::default(),
			[0.1; 2],
			0.03,
			0.0..1.0,
			0.0..1.0,
		)?;
		touch_plane.set_debug(Some(DebugSettings::default()));
		let model = Model::create(
			client.get_root(),
			Transform::default(),
			&ResourceID::new_namespaced("molecules", "button"),
		)?;
		let button_part = model.model_part("Button/Model")?;

		Ok(ButtonDemo {
			touch_plane,
			model,
			button_part,
		})
	}
}
impl RootHandler for ButtonDemo {
	fn frame(&mut self, _info: FrameInfo) {
		self.touch_plane.update();

		// let touch_points = self.touch_plane.touching_points();

		if self.touch_plane.touch_started() {
			println!("Touch started");
			let color = [0.0, 1.0, 0.0, 1.0];
			self.button_part
				.set_material_parameter("color", MaterialParameter::Color(color))
				.unwrap();
			self.button_part
				.set_material_parameter(
					"emission_factor",
					MaterialParameter::Color(color.map(|c| c * 0.75)),
				)
				.unwrap();
		}
		if self.touch_plane.touch_stopped() {
			println!("Touch ended");
			let color = [1.0, 0.0, 0.0, 1.0];
			self.button_part
				.set_material_parameter("color", MaterialParameter::Color(color))
				.unwrap();
			self.button_part
				.set_material_parameter(
					"emission_factor",
					MaterialParameter::Color(color.map(|c| c * 0.5)),
				)
				.unwrap();
		}
	}
}
