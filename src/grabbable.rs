use crate::single_actor_action::SingleActorAction;
use glam::Vec3;
use stardust_xr_fusion::{
	core::values::Transform,
	fields::Field,
	input::{
		action::{BaseInputAction, InputAction, InputActionHandler},
		InputDataType, InputHandler,
	},
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};
use tracing::debug;

#[derive(Debug, Clone, Copy)]
pub struct GrabData {
	pub max_distance: f32,
}

pub struct Grabbable {
	root: Spatial,
	content_parent: Spatial,
	global_action: BaseInputAction<GrabData>,
	condition_action: BaseInputAction<GrabData>,
	grab_action: SingleActorAction<GrabData>,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<GrabData>>,
	min_distance: f32,
}
impl Grabbable {
	pub fn new<Fi: Field>(
		content_space: &Spatial,
		content_transform: Transform,
		field: &Fi,
		settings: GrabData,
	) -> Result<Self, NodeError> {
		let global_action = BaseInputAction::new(false, |_, _| true);
		let condition_action = BaseInputAction::new(false, |input, data: &GrabData| {
			input.distance < data.max_distance
		});
		let grab_action = SingleActorAction::new(
			true,
			|data, _| {
				data.datamap.with_data(|datamap| match &data.input {
					InputDataType::Hand(_) => datamap.idx("pinch_strength").as_f32() > 0.90,
					_ => datamap.idx("grab").as_f32() > 0.90,
				})
			},
			false,
		);
		let input_handler = InputHandler::create(
			content_space.client()?.get_root(),
			Transform::default(),
			field,
		)?
		.wrap(InputActionHandler::new(settings))?;
		let root = Spatial::create(input_handler.node(), Transform::default(), false)?;
		let content_parent = Spatial::create(input_handler.node(), Transform::default(), true)?;
		content_parent.set_transform(Some(content_space), content_transform)?;

		Ok(Grabbable {
			root,
			content_parent,
			global_action,
			condition_action,
			grab_action,
			input_handler,
			min_distance: f32::MAX,
		})
	}
	pub fn update(&mut self) {
		self.input_handler.lock_wrapped().update_actions([
			self.global_action.type_erase(),
			self.condition_action.type_erase(),
			self.grab_action.type_erase(),
		]);
		self.grab_action.update(&mut self.condition_action);

		if let Some(actor) = self.grab_action.actor() {
			let transform = match &actor.input {
				InputDataType::Hand(h) => Transform::from_position_rotation(
					Vec3::from(h.thumb.tip.position).lerp(Vec3::from(h.index.tip.position), 0.5),
					h.palm.rotation,
				),
				InputDataType::Pointer(p) => {
					Transform::from_position_rotation(p.origin, p.orientation)
				}
				InputDataType::Tip(t) => Transform::from_position_rotation(t.origin, t.orientation),
			};

			debug!(?transform, uid = actor.uid, "Currently grabbing");

			self.root
				.set_transform(Some(self.input_handler.node()), transform)
				.unwrap();
		}
		if self.grab_action.actor_started() {
			debug!(
				uid = self.grab_action.actor().as_ref().unwrap().uid,
				"Started grabbing"
			);
			self.content_parent.set_zoneable(false).unwrap();
			self.content_parent
				.set_spatial_parent_in_place(&self.root)
				.unwrap();
		}
		if self.grab_action.actor_stopped() {
			debug!("Stopped grabbing");
			self.content_parent
				.set_spatial_parent_in_place(self.input_handler.node())
				.unwrap();
			self.content_parent.set_zoneable(true).unwrap();
		}

		self.min_distance = self
			.global_action
			.actively_acting
			.iter()
			.map(|data| data.distance)
			.reduce(|a, b| a.min(b))
			.unwrap_or(f32::MAX);
	}
	pub fn grab_action(&self) -> &SingleActorAction<GrabData> {
		&self.grab_action
	}
	pub fn content_parent(&self) -> &Spatial {
		&self.content_parent
	}
	pub fn min_distance(&self) -> f32 {
		self.min_distance
	}
}
