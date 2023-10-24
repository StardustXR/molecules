mod single_actor_action;
pub use single_actor_action::*;

use rustc_hash::FxHashSet;
use stardust_xr_fusion::input::{InputData, InputHandlerHandler, UnknownInputMethod};
use std::{fmt::Debug, mem::swap, sync::Arc};

pub trait InputActionState: Sized + Clone + Send + Sync + 'static {}
impl<T: Sized + Clone + Send + Sync + 'static> InputActionState for T {}

pub type ActiveCondition<S> = fn(&InputData, state: &S) -> bool;

pub trait InputAction<S: InputActionState> {
	fn base(&self) -> &BaseInputAction<S>;
	fn base_mut(&mut self) -> &mut BaseInputAction<S>;
	fn type_erase(&mut self) -> &mut dyn InputAction<S>
	where
		Self: Sized,
	{
		self as &mut dyn InputAction<S>
	}
}

#[derive(Clone)]
pub struct BaseInputAction<S: InputActionState> {
	pub capture_on_trigger: bool,
	pub active_condition: ActiveCondition<S>,

	pub started_acting: FxHashSet<Arc<InputData>>,
	pub currently_acting: FxHashSet<Arc<InputData>>,
	pub stopped_acting: FxHashSet<Arc<InputData>>,
	queued_inputs: FxHashSet<Arc<InputData>>,
}
impl<S: InputActionState> BaseInputAction<S> {
	pub fn new(capture_on_trigger: bool, active_condition: ActiveCondition<S>) -> Self {
		Self {
			capture_on_trigger,
			active_condition,

			started_acting: FxHashSet::default(),
			currently_acting: FxHashSet::default(),
			stopped_acting: FxHashSet::default(),
			queued_inputs: FxHashSet::default(),
		}
	}

	fn update(&mut self, external: &mut BaseInputAction<S>) {
		self.started_acting = FxHashSet::from_iter(
			self.queued_inputs
				.difference(&self.currently_acting)
				.cloned(),
		);
		self.stopped_acting = FxHashSet::from_iter(
			self.currently_acting
				.difference(&self.queued_inputs)
				.cloned(),
		);
		swap(&mut self.currently_acting, &mut self.queued_inputs);
		self.queued_inputs.clear();

		external.started_acting = self.started_acting.clone();
		external.currently_acting = self.currently_acting.clone();
		external.stopped_acting = self.stopped_acting.clone();
		external.started_acting = self.started_acting.clone();

		self.capture_on_trigger = external.capture_on_trigger;
		self.active_condition = external.active_condition;
	}

	fn input_event(&mut self, data: &Arc<InputData>, state: &S) -> bool {
		if (self.active_condition)(data, state) {
			// if we want to capture this on trigger, then we shouldn't count it as triggered until it successfully captures
			if !(self.capture_on_trigger && !data.captured) {
				self.queued_inputs.insert(data.clone());
			}
			true
		} else {
			false
		}
	}
}

impl<S: InputActionState> InputAction<S> for BaseInputAction<S> {
	fn base(&self) -> &BaseInputAction<S> {
		self
	}
	fn base_mut(&mut self) -> &mut BaseInputAction<S> {
		self
	}
}
impl<S: InputActionState> PartialEq for BaseInputAction<S> {
	fn eq(&self, other: &Self) -> bool {
		self.capture_on_trigger == other.capture_on_trigger
			&& self.active_condition as usize == other.active_condition as usize
	}
}
impl<S: InputActionState> Debug for BaseInputAction<S> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("InputAction")
			.field("capture_on_trigger", &self.capture_on_trigger)
			.field("started_acting", &self.started_acting)
			.field("actively_acting", &self.currently_acting)
			.field("stopped_acting", &self.stopped_acting)
			.field("queued_inputs", &self.queued_inputs)
			.finish()
	}
}

#[derive(Debug, Default)]
pub struct InputActionHandler<S: InputActionState> {
	actions: Vec<BaseInputAction<S>>,
	state: S,
	back_state: S,
}
impl<S: InputActionState> InputActionHandler<S> {
	pub fn new(state: S) -> Self {
		Self {
			actions: Vec::new(),
			back_state: state.clone(),
			state,
		}
	}

	pub fn update_actions<'a>(
		&mut self,
		actions: impl IntoIterator<Item = &'a mut (dyn InputAction<S> + 'a)>,
	) {
		self.back_state = self.state.clone();

		self.actions = actions
			.into_iter()
			.map(|action| {
				if let Some(internal_action) = self
					.actions
					.iter_mut()
					.find(|internal_action| **internal_action == *action.base())
				{
					internal_action.update(action.base_mut());
				}
				action.base().clone()
			})
			.collect();
	}
	pub fn update_state(&mut self, state: S) {
		self.state = state;
	}
}
impl<S: InputActionState> InputHandlerHandler for InputActionHandler<S> {
	fn input(&mut self, input: UnknownInputMethod, data: InputData) {
		let data = Arc::new(data);
		let capture = self
			.actions
			.iter_mut()
			.map(|action| action.input_event(&data, &self.state) && action.capture_on_trigger)
			.reduce(|a, b| a || b)
			.unwrap_or_default();
		if capture {
			let _ = input.capture();
		}
	}
}
