use crate::data::SimplePulseReceiver;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	core::values::Datamap,
	data::{PulseReceiver, PulseReceiverAspect, PulseSender},
	fields::FieldAspect,
	items::panel::{PanelItem, PanelItemAspect, SurfaceId},
	node::{NodeError, NodeType},
	spatial::{SpatialAspect, Transform},
};

lazy_static::lazy_static! {
	pub static ref KEYBOARD_MASK: Datamap = Datamap::from_typed(KeyboardEvent::default()).unwrap();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyboardEvent {
	pub keyboard: (),
	pub xkbv1: (),
	pub keymap_id: String,
	pub keys: FxHashSet<i32>,
}
impl Default for KeyboardEvent {
	fn default() -> Self {
		Self {
			keyboard: (),
			xkbv1: (),
			keymap_id: Default::default(),
			keys: Default::default(),
		}
	}
}
impl KeyboardEvent {
	pub fn send_event(&self, sender: &PulseSender, receivers: &[&PulseReceiver]) {
		let data = Datamap::from_typed(self).unwrap();
		for receiver in receivers.into_iter() {
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
		panel.keyboard_keys(surface, &self.keymap_id, &keys)
	}
}

pub type KeyboardPanelHandler = SimplePulseReceiver<KeyboardEvent>;
pub fn create_keyboard_panel_handler(
	parent: &impl SpatialAspect,
	transform: Transform,
	field: &impl FieldAspect,
	panel: &PanelItem,
	focus: SurfaceId,
) -> Result<KeyboardPanelHandler, NodeError> {
	let panel = panel.alias();
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
	use stardust_xr_fusion::data::PulseSenderAspect;
	let (client, event_loop) = stardust_xr_fusion::client::Client::connect_with_async_loop()
		.await
		.unwrap();

	struct PulseSenderTest {
		data: Datamap,
		node: PulseSender,
	}
	impl stardust_xr_fusion::data::PulseSenderHandler for PulseSenderTest {
		fn new_receiver(
			&mut self,
			uid: String,
			receiver: PulseReceiver,
			field: stardust_xr_fusion::fields::Field,
		) {
			println!(
				"New pulse receiver {:?} with field {:?} and uid {uid}",
				receiver.node().get_path(),
				field.node().get_path(),
			);
			receiver.send_data(&self.node, &self.data).unwrap();
		}
		fn drop_receiver(&mut self, uid: String) {
			println!("Pulse receiver {} dropped", uid);
		}
	}

	let field =
		stardust_xr_fusion::fields::SphereField::create(client.get_root(), [0.0; 3], 0.1).unwrap();

	let keyboard_event = KeyboardEvent {
		keyboard: (),
		xkbv1: (),
		keymap_id: "".to_string(),
		keys: [1, -1].into_iter().collect(),
	};
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::none(), &KEYBOARD_MASK).unwrap();
	let pulse_sender_test = PulseSenderTest {
		data: Datamap::from_typed(keyboard_event).unwrap(),
		node: pulse_sender.alias(),
	};
	let _pulse_sender = pulse_sender.wrap(pulse_sender_test).unwrap();
	let _pulse_receiver = SimplePulseReceiver::create(
		client.get_root(),
		Transform::none(),
		&field,
		move |uid, keyboard_event: KeyboardEvent| {
			println!("Pulse sender {} sent {:#?}", uid, keyboard_event);
		},
	)
	.unwrap();

	tokio::select! {
		_ = tokio::time::sleep(core::time::Duration::from_secs(60)) => panic!("Timed Out"),
		e = event_loop => e.unwrap().unwrap(),
	}
}
