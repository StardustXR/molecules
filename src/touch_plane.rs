use crate::{
	input_action::{BaseInputAction, InputActionHandler},
	lines::{self, make_line_points},
	DebugSettings, VisualDebug,
};
use glam::{vec3, Vec3};
use map_range::MapRange;
use mint::{Vector2, Vector3};
use rustc_hash::FxHashSet;
use stardust_xr_fusion::{
	drawable::{Line, Lines},
	fields::{BoxField, BoxFieldAspect, UnknownField},
	input::{InputData, InputDataType, InputHandler},
	node::{NodeError, NodeType},
	spatial::{Spatial, SpatialAspect, Transform},
	HandlerWrapper,
};
use std::{ops::Range, sync::Arc};

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
	touch_capture_action: BaseInputAction<State>,
	size: Vector2<f32>,
	pub x_range: Range<f32>,
	pub y_range: Range<f32>,
	thickness: f32,
	currently_hovering: FxHashSet<Arc<InputData>>,
	started_touching: FxHashSet<Arc<InputData>>,
	currently_interacting: FxHashSet<Arc<InputData>>,
	stopped_interacting: FxHashSet<Arc<InputData>>,
	debug_lines: Option<Lines>,
}
impl TouchPlane {
	pub fn create(
		parent: &impl SpatialAspect,
		transform: Transform,
		size: impl Into<Vector2<f32>>,
		thickness: f32,
		x_range: Range<f32>,
		y_range: Range<f32>,
	) -> Result<Self, NodeError> {
		let size = size.into();
		let root = Spatial::create(parent, transform, false)?;
		let field = BoxField::create(
			&root,
			Transform::from_translation([0.0, 0.0, thickness * -0.5]),
			[size.x, size.y, thickness],
		)?;
		let input = InputActionHandler::wrap(
			InputHandler::create(&root, Transform::none(), &field)?,
			State { size },
		)?;

		let hover_action = BaseInputAction::new(false, Self::hover_action);
		let touch_action = BaseInputAction::new(false, Self::touch_action);
		let touch_capture_action = BaseInputAction::new(true, Self::touch_action);
		Ok(TouchPlane {
			root,
			input,
			field,
			hover_action,
			touch_action,
			touch_capture_action,
			size,
			x_range,
			y_range,
			thickness,
			currently_hovering: FxHashSet::default(),
			started_touching: FxHashSet::default(),
			currently_interacting: FxHashSet::default(),
			stopped_interacting: FxHashSet::default(),
			debug_lines: None,
		})
	}

	fn hover(size: Vector2<f32>, point: Vector3<f32>, front: bool) -> bool {
		point.x.abs() * 2.0 < size.x
			&& point.y.abs() * 2.0 < size.y
			&& point.z.is_sign_positive() == front
	}
	fn hover_action(input: &InputData, state: &State) -> bool {
		match &input.input {
			InputDataType::Pointer(_) => input.distance < 0.0,
			InputDataType::Hand(h) => {
				// Self::hover(state.size, h.thumb.tip.position) ||
				Self::hover(state.size, h.index.tip.position, true)
					|| Self::hover(state.size, h.index.tip.position, false)
			}
			InputDataType::Tip(t) => {
				Self::hover(state.size, t.origin, true) || Self::hover(state.size, t.origin, false)
			}
		}
	}
	fn touch_action(input: &InputData, state: &State) -> bool {
		match &input.input {
			InputDataType::Pointer(_) => {
				input.datamap.with_data(|d| d.idx("select").as_f32() > 0.5)
			}
			InputDataType::Hand(h) => {
				// Self::hover(state.size, h.thumb.tip.position) ||
				Self::hover(state.size, h.index.tip.position, false)
			}
			InputDataType::Tip(t) => Self::hover(state.size, t.origin, false),
		}
	}
	pub fn interact_point(&self, input: &InputData) -> (Vector2<f32>, f32) {
		let interact_point = match &input.input {
			InputDataType::Pointer(p) => {
				let normal = vec3(0.0, 0.0, 1.0);
				let denom = normal.dot(p.direction().into());
				let t = -Vec3::from(p.origin).dot(normal) / denom;
				(Vec3::from(p.origin) + Vec3::from(p.direction()) * t).into()
			}
			InputDataType::Hand(h) => h.index.tip.position,
			InputDataType::Tip(t) => t.origin,
		};

		let x = interact_point
			.x
			.clamp(-self.size.x / 2.0, self.size.x / 2.0)
			.map_range(-self.size.x / 2.0..self.size.x / 2.0, self.x_range.clone());
		let y = interact_point
			.y
			.clamp(-self.size.y / 2.0, self.size.y / 2.0)
			.map_range(self.size.y / 2.0..-self.size.y / 2.0, self.y_range.clone());

		([x, y].into(), interact_point.z)
	}
	pub fn input_to_points<'a>(
		&self,
		inputs: impl Iterator<Item = &'a Arc<InputData>>,
	) -> Vec<Vector2<f32>> {
		inputs.map(|i| self.interact_point(i).0).collect()
	}

	pub fn root(&self) -> &Spatial {
		&self.root
	}
	pub fn input_handler(&self) -> &InputHandler {
		self.input.node()
	}
	pub fn field(&self) -> UnknownField {
		UnknownField::alias_field(&self.field)
	}

	pub fn set_size(&mut self, size: impl Into<Vector2<f32>>) -> Result<(), NodeError> {
		let size = size.into();
		self.size = size;
		self.input.lock_wrapped().update_state(State { size });
		self.field.set_size([size.x, size.y, self.thickness])?;
		Ok(())
	}
	pub fn set_thickness(&mut self, thickness: f32) -> Result<(), NodeError> {
		self.thickness = thickness;
		self.field
			.set_local_transform(Transform::from_translation([0.0, 0.0, thickness * -0.5]))?;
		self.field.set_size([self.size.x, self.size.y, thickness])?;
		Ok(())
	}

	/// Get all the raw inputs that are touching
	pub fn touching_inputs(&self) -> &FxHashSet<Arc<InputData>> {
		&self.touch_action.currently_acting
	}
	/// Is the surface getting its first touch?
	pub fn touch_started(&self) -> bool {
		!self.started_touching.is_empty()
			&& self
				.started_touching
				.is_superset(&self.currently_interacting)
	}
	/// Is something touching the surface?
	pub fn touching(&self) -> bool {
		!self.currently_interacting.is_empty()
	}
	/// Did everything just stop touching the surface?
	pub fn touch_stopped(&self) -> bool {
		self.currently_interacting.is_empty() && !self.stopped_interacting.is_empty()
	}

	/// Get all the raw inputs that are hovering
	pub fn hovering_inputs(&self) -> FxHashSet<Arc<InputData>> {
		self.currently_hovering
			.difference(&self.currently_interacting)
			.cloned()
			.collect()
	}
	/// Get all the points hovering over the surface, in x_range and y_range
	pub fn hover_points(&self) -> Vec<Vector2<f32>> {
		self.input_to_points(self.hover_action.currently_acting.iter())
	}

	/// Get all the raw inputs that just started touching
	pub fn started_inputs(&self) -> &FxHashSet<Arc<InputData>> {
		&self.started_touching
	}
	/// Get all the raw inputs that are currently touching
	pub fn interacting_inputs(&self) -> &FxHashSet<Arc<InputData>> {
		&self.currently_interacting
	}
	/// Get all the raw inputs that just stopped touching
	pub fn stopped_inputs(&self) -> &FxHashSet<Arc<InputData>> {
		&self.stopped_interacting
	}

	/// Get all the 2D points in the x and y range that just started touching
	pub fn touch_down_points(&self) -> Vec<Vector2<f32>> {
		self.input_to_points(self.started_touching.iter())
	}
	/// Get all the 2D points in the x and y range that are currently touching
	pub fn touching_points(&self) -> Vec<Vector2<f32>> {
		self.input_to_points(self.currently_interacting.iter())
	}
	/// Get all the 2D points in the x and y range that just stopped touching
	pub fn touch_up_points(&self) -> Vec<Vector2<f32>> {
		self.input_to_points(self.stopped_interacting.iter())
	}

	/// Set whether this will receive input or not
	pub fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.input.node().set_enabled(enabled)
	}

	/// Update the state of this touch plane. Run once every frame.
	pub fn update(&mut self) {
		self.input.lock_wrapped().update_actions([
			&mut self.hover_action,
			&mut self.touch_action,
			&mut self.touch_capture_action,
		]);

		// Update the currently hovering stuff
		self.currently_hovering = self.hover_action.currently_acting.clone();

		// When we move from hovering in front of the button to intersecting it, that's the start of a touch!self
		let hovered: FxHashSet<Arc<InputData>> = self
			.hover_action
			.currently_acting
			.iter()
			.filter(|i| !self.hover_action.started_acting.contains(*i))
			.chain(self.hover_action.stopped_acting.iter())
			.cloned()
			.collect();
		self.started_touching = hovered
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
			.chain(self.started_touching.iter())
			// Update all the items that are still valid
			.filter_map(|i| self.touch_action.currently_acting.get(i))
			.cloned()
			.collect();
	}
}
impl VisualDebug for TouchPlane {
	fn set_debug(&mut self, settings: Option<DebugSettings>) {
		self.debug_lines = settings.and_then(|settings| {
			let square = lines::rounded_rectangle(
				self.size.x,
				self.size.y,
				settings.line_thickness * 0.5,
				4,
			);
			let line_points =
				make_line_points(&square, settings.line_thickness, settings.line_color);
			let line_back = Line {
				points: line_points
					.iter()
					.cloned()
					.map(|mut l| {
						l.color.a = 0.5;
						l.point.z = -self.thickness;
						l
					})
					.collect::<Vec<_>>(),
				cyclic: true,
			};
			let line_front = Line {
				points: line_points,
				cyclic: true,
			};
			let lines = Lines::create(
				&self.root,
				Transform::from_translation([0.0, 0.0, 0.0]),
				&[line_front, line_back],
			)
			.ok()?;
			Some(lines)
		})
	}
}
