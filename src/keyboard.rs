use std::sync::Arc;

use crate::data::SimplePulseReceiver;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	core::values::Datamap,
	data::{PulseReceiver, PulseReceiverAspect, PulseSender},
	fields::FieldAspect,
	items::panel::{PanelItem, PanelItemAspect, SurfaceId},
	node::NodeError,
	spatial::{SpatialRefAspect, Transform},
};

lazy_static::lazy_static! {
	pub static ref KEYBOARD_MASK: Datamap = Datamap::from_typed(KeyboardEvent::default()).unwrap();
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct KeyboardEvent {
	pub keyboard: (),
	pub xkbv1: (),
	pub keymap_id: u64,
	pub keys: FxHashSet<i32>,
}
impl KeyboardEvent {
	pub fn send_event(&self, sender: &PulseSender, receivers: &[&PulseReceiver]) {
		let data = Datamap::from_typed(self).unwrap();
		for receiver in receivers.iter() {
			let _ = receiver.send_data(sender, &data);
		}
	}

	// pub fn update_xkb_state(&self, receiver_key_state: &mut State) {
	// 	if let Some(state) = self.keymap_id.as_ref().and_then(|k| {
	// 		let ctx = Context::new(CONTEXT_NO_FLAGS);
	// 		let keymap = Keymap::new_from_string(
	// 			&ctx,
	// 			k.clone(),
	// 			KEYMAP_FORMAT_TEXT_V1,
	// 			KEYMAP_COMPILE_NO_FLAGS,
	// 		)?;
	// 		Some(State::new(&keymap))
	// 	}) {
	// 		*receiver_key_state = state;
	// 	};
	// 	if let Some(keys_up) = &self.keys {
	// 		for key_up in keys_up {
	// 			receiver_key_state.update_key((*key_up).into(), KeyDirection::Up);
	// 		}
	// 	}
	// 	if let Some(keys_down) = &self.keys_down {
	// 		for key_down in keys_down {
	// 			receiver_key_state.update_key((*key_down).into(), KeyDirection::Down);
	// 		}
	// 	}
	// }

	pub fn send_to_panel(self, panel: &PanelItem, surface: SurfaceId) -> Result<(), NodeError> {
		let keys = self.keys.iter().cloned().collect::<Vec<_>>();
		panel.keyboard_keys(surface, self.keymap_id, &keys)
	}
}

pub type KeyboardPanelHandler = SimplePulseReceiver<KeyboardEvent>;
pub fn create_keyboard_panel_handler(
	parent: &impl SpatialRefAspect,
	transform: Transform,
	field: &impl FieldAspect,
	panel: &Arc<PanelItem>,
	focus: SurfaceId,
) -> Result<KeyboardPanelHandler, NodeError> {
	let panel = panel.clone();
	SimplePulseReceiver::create(
		parent,
		transform,
		field,
		move |_uid, data: KeyboardEvent| {
			let _ = data.send_to_panel(&panel, focus.clone());
		},
	)
}

#[tokio::test]
async fn keyboard_events() {
	use crate::UIElement;
	use stardust_xr_fusion::data::PulseSenderAspect;
	use stardust_xr_fusion::data::PulseSenderEvent;
	use stardust_xr_fusion::node::NodeType;
	let mut client = stardust_xr_fusion::Client::connect().await.unwrap();

	let field = Arc::new(
		stardust_xr_fusion::fields::Field::create(
			client.get_root(),
			Transform::identity(),
			stardust_xr_fusion::fields::Shape::Sphere(0.1),
		)
		.unwrap(),
	);

	let keyboard_event = KeyboardEvent {
		keyboard: (),
		xkbv1: (),
		keymap_id: 0,
		keys: [1, -1].into_iter().collect(),
	};
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::none(), &KEYBOARD_MASK).unwrap();

	let mut pulse_receiver = None;
	let data = Datamap::from_typed(keyboard_event.clone()).unwrap();

	let event_loop = client.event_loop(move |client, _flow| {
		let pulse_receiver = pulse_receiver.get_or_insert_with({
			let client = client.clone();
			let field = field.clone();
			move || {
				SimplePulseReceiver::create(
					client.get_root(),
					Transform::none(),
					field.as_ref(),
					move |sender, keyboard_event: KeyboardEvent| {
						println!(
							"Pulse sender {} sent {:#?}",
							sender.node().get_id().unwrap(),
							keyboard_event
						);
					},
				)
				.unwrap()
			}
		});
		pulse_receiver.handle_events();

		match pulse_sender.recv_event() {
			Some(PulseSenderEvent::NewReceiver { receiver, field }) => {
				println!(
					"New pulse receiver {:?} with field {:?}",
					receiver.node().get_id(),
					field.node().get_id(),
				);
				receiver.send_data(&pulse_sender, &data).unwrap();
			}
			Some(PulseSenderEvent::DropReceiver { id }) => {
				println!("Pulse receiver {} dropped", id);
			}
			_ => (),
		}
	});

	tokio::time::timeout(core::time::Duration::from_secs(60), event_loop)
		.await
		.unwrap()
		.unwrap()
}
