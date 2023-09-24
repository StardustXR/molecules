use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	core::{schemas::flex::flexbuffers, values::Transform},
	data::{PulseReceiver, PulseSender},
	fields::Field,
	items::panel::{PanelItem, SurfaceID},
	node::{NodeError, NodeType},
	spatial::Spatial,
};
pub use xkbcommon::xkb;

use crate::{data::SimplePulseReceiver, datamap::Datamap};

lazy_static::lazy_static! {
	pub static ref KEYBOARD_MASK: Vec<u8> = Datamap::create(KeyboardEvent::default()).serialize();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyboardEvent {
	pub keyboard: (),
	pub xkbv1: (),
	pub keymap_id: String,
	pub keys: Vec<i32>,
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
	pub fn serialize_pulse_data(&self) -> Vec<u8> {
		let mut serializer = flexbuffers::FlexbufferSerializer::new();
		let _ = self.serialize(&mut serializer);
		serializer.take_buffer()
	}

	pub fn send_event(&self, sender: &PulseSender, receivers: &[&PulseReceiver]) {
		let mut serializer = flexbuffers::FlexbufferSerializer::new();
		if self.serialize(&mut serializer).is_ok() {
			let data = serializer.take_buffer();
			for receiver in receivers.into_iter() {
				let _ = sender.send_data(receiver, &data);
			}
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

	pub fn send_to_panel(self, panel: &PanelItem, surface: &SurfaceID) -> Result<(), NodeError> {
		panel.keyboard_keys(surface, &self.keymap_id, self.keys)
	}
}

pub type KeyboardPanelHandler = SimplePulseReceiver<KeyboardEvent>;
pub fn create_keyboard_panel_handler<Fi: Field>(
	parent: &Spatial,
	transform: Transform,
	field: &Fi,
	panel: &PanelItem,
	focus: SurfaceID,
) -> Result<KeyboardPanelHandler, NodeError> {
	let panel = panel.alias();
	SimplePulseReceiver::create(
		parent,
		transform,
		field,
		move |_uid, data: KeyboardEvent| {
			let _ = data.send_to_panel(&panel, &focus);
		},
	)
}

#[tokio::test]
async fn keyboard_events() {
	let (client, event_loop) = stardust_xr_fusion::client::Client::connect_with_async_loop()
		.await
		.unwrap();
	use stardust_xr_fusion::{core::values::Transform, node::NodeType};

	struct PulseSenderTest {
		data: Vec<u8>,
		node: PulseSender,
	}
	impl stardust_xr_fusion::data::PulseSenderHandler for PulseSenderTest {
		fn new_receiver(
			&mut self,
			info: stardust_xr_fusion::data::NewReceiverInfo,
			receiver: PulseReceiver,
			field: stardust_xr_fusion::fields::UnknownField,
		) {
			println!(
				"New pulse receiver {:?} with field {:?} and info {:?}",
				receiver.node().get_path(),
				field.node().get_path(),
				info
			);
			self.node.send_data(&receiver, &self.data).unwrap();
		}
		fn drop_receiver(&mut self, uid: &str) {
			println!("Pulse receiver {} dropped", uid);
		}
	}

	let field = stardust_xr_fusion::fields::SphereField::create(
		client.get_root(),
		mint::Vector3::from([0.0; 3]),
		0.1,
	)
	.unwrap();

	let mut keyboard_event_serializer = flexbuffers::FlexbufferSerializer::new();
	let keyboard_event = KeyboardEvent {
		keyboard: (),
		xkbv1: (),
		keymap_id: "".to_string(),
		keys: vec![1, -1],
	};
	keyboard_event
		.serialize(&mut keyboard_event_serializer)
		.unwrap();
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::none(), &KEYBOARD_MASK).unwrap();
	let pulse_sender_test = PulseSenderTest {
		data: keyboard_event_serializer.take_buffer(),
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
