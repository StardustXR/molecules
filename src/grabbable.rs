use crate::single_actor_action::SingleActorAction;
use glam::Vec3;
use mint::Vector3;
use stardust_xr_fusion::{
	fields::Field,
	input::{
		action::{BaseInputAction, InputAction, InputActionHandler},
		InputDataType, InputHandler,
	},
	node::NodeError,
	spatial::Spatial,
	HandlerWrapper,
};

pub struct Grabbable {
	root: Spatial,
	content_parent: Spatial,
	condition_action: BaseInputAction<()>,
	grab_action: SingleActorAction<()>,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<()>>,
}
impl Grabbable {
	pub fn new(parent: &Spatial, field: &Field) -> Result<Self, NodeError> {
		let input_handler =
			InputHandler::create(
				parent,
				None,
				None,
				field,
				|_, _| InputActionHandler::new(()),
			)?;
		let root = Spatial::builder()
			.spatial_parent(input_handler.node())
			.zoneable(false)
			.build()?;
		let content_parent = Spatial::builder()
			.spatial_parent(&input_handler.node())
			.zoneable(false)
			.build()?;

		Ok(Grabbable {
			root,
			content_parent,
			condition_action: BaseInputAction::new(false, |data, _| data.distance < 0.05),
			grab_action: SingleActorAction::new(
				true,
				|data, _| {
					data.datamap.with_data(|datamap| match &data.input {
						InputDataType::Pointer(_) => datamap.idx("grab").as_bool(),
						InputDataType::Hand(_) => (datamap.idx("pinchStrength").as_f32() > 0.99),
					})
				},
				false,
			),
			input_handler,
		})
	}
	pub fn update(&mut self) {
		self.input_handler.lock_inner().update_actions([
			self.condition_action.type_erase(),
			self.grab_action.type_erase(),
		]);
		self.grab_action.update(&mut self.condition_action);
		// dbg!(&self.condition_action.actively_acting.len());
		// dbg!(&self.grab_action.actor().is_some());

		if self.grab_action.actor_acting() {
			match &self.grab_action.actor().unwrap().input {
				InputDataType::Hand(h) => {
					let thumb_tip_pos: Vector3<f32> = h.thumb.tip.position.clone().into();
					let thumb_tip_pos: Vec3 = thumb_tip_pos.into();
					let index_tip_pos: Vector3<f32> = h.index.tip.position.clone().into();
					let index_tip_pos: Vec3 = index_tip_pos.into();
					let pinch_pos = (thumb_tip_pos + index_tip_pos) * 0.5;
					self.root
						.set_transform(
							Some(self.input_handler.node()),
							Some(pinch_pos.into()),
							Some(h.palm.rotation.clone().into()),
							None,
						)
						.unwrap();
				}
				InputDataType::Pointer(_p) => (),
			}
		}
		if self.grab_action.actor_started() {
			self.content_parent
				.set_spatial_parent_in_place(&self.root)
				.unwrap();
		}
		if self.grab_action.actor_stopped() {
			self.content_parent
				.set_spatial_parent_in_place(self.input_handler.node())
				.unwrap();
		}
	}
	pub fn grab_action(&self) -> &SingleActorAction<()> {
		&self.grab_action
	}
	pub fn content_parent(&self) -> &Spatial {
		&self.content_parent
	}
}
