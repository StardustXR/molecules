pub trait State<D> {
	fn on_enter(&mut self, _data: &mut D) {}
	/// return a new state if you want to change the current state
	fn update(&mut self, data: &mut D) -> Option<Box<dyn State<D>>>;
	fn on_exit(&mut self, _data: &mut D) {}
}

pub struct StateMachine<D>(Box<dyn State<D>>);
impl<D> StateMachine<D> {
	pub fn new<S: State<D> + 'static>(mut initial_state: S, data: &mut D) -> Self {
		initial_state.on_enter(data);
		StateMachine(Box::new(initial_state))
	}
	pub fn update(&mut self, data: &mut D) {
		if let Some(new_state) = self.0.update(data) {
			self.0.on_exit(data);
			self.0 = new_state;
			self.0.on_enter(data);
		}
	}
}

pub struct OwnedStateMachine<D> {
	state: StateMachine<D>,
	pub data: D,
}
impl<D> OwnedStateMachine<D> {
	pub fn new<S: State<D> + 'static>(initial_state: S, mut data: D) -> Self {
		OwnedStateMachine {
			state: StateMachine::new(initial_state, &mut data),
			data,
		}
	}
	pub fn update(&mut self) {
		self.state.update(&mut self.data);
	}
}
