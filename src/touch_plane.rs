use std::sync::Arc;

use mint::{Vector2, Vector3};
use rustc_hash::FxHashSet;
use stardust_xr_fusion::{
	core::values::Transform,
	drawable::Lines,
	fields::BoxField,
	input::{
		action::{BaseInputAction, InputAction, InputActionHandler},
		InputData, InputDataType, InputHandler,
	},
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};

use crate::{lines, DebugSettings, VisualDebug};

#[derive(Debug, Clone, Copy)]
struct State {
	size: Vector2<f32>,
}

pub struct TouchPlane {
	root: Spatial,
	input: HandlerWrapper<InputHandler, InputActionHandler<State>>,
	field: BoxField,
	hover_action: BaseInputAction<State>,
	touch_action: BaseInputAction<State>,
	size: Vector2<f32>,
	thickness: f32,
	started_interacting: FxHashSet<Arc<InputData>>,
	currently_interacting: FxHashSet<Arc<InputData>>,
	stopped_interacting: FxHashSet<Arc<InputData>>,
	debug_lines: Option<Lines>,
}
impl TouchPlane {
	pub fn new(
		parent: &Spatial,
		transform: Transform,
		size: impl Into<Vector2<f32>>,
		thickness: f32,
	) -> Result<Self, NodeError> {
		let size = size.into();
		let root = Spatial::create(parent, transform, false)?;
		let field = BoxField::create(
			&root,
			Transform::from_position([0.0, 0.0, thickness * -0.5]),
			[size.x, size.y, thickness],
		)?;
		let input = InputHandler::create(&root, Transform::default(), &field)?
			.wrap(InputActionHandler::new(State { size }))?;

		let hover_action = BaseInputAction::new(false, Self::hover_action);
		let touch_action = BaseInputAction::new(true, Self::touch_action);
		Ok(TouchPlane {
			root,
			input,
			field,
			hover_action,
			touch_action,
			size,
			thickness,
			started_interacting: FxHashSet::default(),
			currently_interacting: FxHashSet::default(),
			stopped_interacting: FxHashSet::default(),
			debug_lines: None,
		})
	}

	fn hover(size: Vector2<f32>, point: Vector3<f32>) -> bool {
		point.x.abs() * 2.0 < size.x && point.y.abs() * 2.0 < size.y && point.z > 0.0
	}
	fn hover_action(input: &InputData, state: &State) -> bool {
		match &input.input {
			InputDataType::Pointer(_) => {
				input.datamap.with_data(|d| d.idx("select").as_f32() < 0.5) && input.distance < 0.0
			}
			InputDataType::Hand(h) => {
				Self::hover(state.size, h.thumb.tip.position)
					|| Self::hover(state.size, h.index.tip.position)
			}
			InputDataType::Tip(t) => Self::hover(state.size, t.origin),
		}
	}
	fn touch_action(input: &InputData, _state: &State) -> bool {
		match &input.input {
			InputDataType::Pointer(_) => {
				input.datamap.with_data(|d| d.idx("select").as_f32() > 0.5) && input.distance < 0.0
			}
			_ => input.distance < 0.0,
		}
	}

	pub fn root(&self) -> &Spatial {
		&self.root
	}

	pub fn set_size(&mut self, size: impl Into<Vector2<f32>>) -> Result<(), NodeError> {
		let size = size.into();
		self.size = size;
		self.field.set_size([size.x, size.y, self.thickness])?;
		Ok(())
	}
	pub fn set_thickness(&mut self, thickness: f32) -> Result<(), NodeError> {
		self.thickness = thickness;
		self.field.set_size([self.size.x, self.size.y, thickness])?;
		Ok(())
	}

	pub fn touch_started(&self) -> bool {
		!self.started_interacting.is_empty()
			&& self
				.started_interacting
				.is_superset(&self.currently_interacting)
	}
	pub fn touching(&self) -> bool {
		!self.currently_interacting.is_empty()
	}
	pub fn touch_stopped(&self) -> bool {
		self.currently_interacting.is_empty() && !self.stopped_interacting.is_empty()
	}

	pub fn started_inputs(&self) -> Vec<Arc<InputData>> {
		self.started_interacting.iter().cloned().collect()
	}
	pub fn interacting_inputs(&self) -> Vec<Arc<InputData>> {
		self.currently_interacting.iter().cloned().collect()
	}
	pub fn stopped_inputs(&self) -> Vec<Arc<InputData>> {
		self.stopped_interacting.iter().cloned().collect()
	}

	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.node().set_enabled(enabled)
	}

	pub fn update(&mut self) {
		self.input.lock_wrapped().update_actions([
			self.hover_action.type_erase(),
			self.touch_action.type_erase(),
		]);
		// When we move from hovering in front of the button to intersecting it, that's the start of a touch!self
		// let hovered: FxHashSet<Arc<InputData>> = self
		// 	.hover_action
		// 	.actively_acting
		// 	.iter()
		// 	.chain(self.hover_action.stopped_acting.iter())
		// 	.cloned()
		// 	.collect();
		self.started_interacting = self
			.hover_action
			.actively_acting
			.intersection(&self.touch_action.started_acting)
			.cloned()
			.collect();
		// When we have a successful touch stop intersecting the field, it's stopped interacting
		self.stopped_interacting = self
			.touch_action
			.stopped_acting
			.intersection(&self.currently_interacting)
			.cloned()
			.collect();
		// Update the currently interacting stuff
		self.currently_interacting = self
			.currently_interacting
			.iter()
			// Add all the items that just started touching after hovering
			.chain(self.started_interacting.iter())
			// Update all the items that are still valid
			.filter_map(|i| self.touch_action.actively_acting.get(i))
			.cloned()
			.collect();
	}
}
impl VisualDebug for TouchPlane {
	fn set_debug(&mut self, settings: Option<DebugSettings>) {
		self.debug_lines = settings.and_then(|settings| {
			Lines::create(
				&self.root,
				Transform::none(),
				&lines::square(self.size.x, self.size.y, settings.thickness, settings.color),
				true,
			)
			.ok()
		})
	}
}
