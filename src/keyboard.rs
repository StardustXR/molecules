use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	core::{schemas::flex::flexbuffers, values::Transform},
	data::{PulseReceiver, PulseReceiverHandler, PulseSender},
	fields::Field,
	items::panel::{PanelItem, SurfaceID},
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};
pub use xkbcommon::xkb;
use xkbcommon::xkb::{
	Context, KeyDirection, Keymap, State, CONTEXT_NO_FLAGS, KEYMAP_COMPILE_NO_FLAGS,
	KEYMAP_FORMAT_TEXT_V1,
};

use crate::datamap::Datamap;

lazy_static::lazy_static! {
	pub static ref KEYBOARD_MASK: Vec<u8> = Datamap::create(KeyboardEvent::default()).serialize();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyboardEvent {
	pub keyboard: String,
	pub keymap: Option<String>,
	pub keys_up: Option<Vec<u32>>,
	pub keys_down: Option<Vec<u32>>,
}
impl Default for KeyboardEvent {
	fn default() -> Self {
		Self {
			keyboard: "xkbv1".to_string(),
			keymap: None,
			keys_up: None,
			keys_down: None,
		}
	}
}
impl KeyboardEvent {
	pub fn new(
		keymap: Option<&Keymap>,
		keys_up: Option<Vec<u32>>,
		keys_down: Option<Vec<u32>>,
	) -> Self {
		KeyboardEvent {
			keyboard: "xkbv1".to_string(),
			keymap: keymap.map(|k| k.get_as_string(KEYMAP_FORMAT_TEXT_V1)),
			keys_up,
			keys_down,
		}
	}

	pub fn from_pulse_data(data: &[u8]) -> Option<Self> {
		flexbuffers::Reader::get_root(data)
			.ok()
			.and_then(|r| KeyboardEvent::deserialize(r).ok())
			.and_then(|ev| {
				if &ev.keyboard == "xkbv1" {
					Some(ev)
				} else {
					None
				}
			})
	}

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

	pub fn update_xkb_state(&self, receiver_key_state: &mut State) {
		if let Some(state) = self.keymap.as_ref().and_then(|k| {
			let ctx = Context::new(CONTEXT_NO_FLAGS);
			let keymap = Keymap::new_from_string(
				&ctx,
				k.clone(),
				KEYMAP_FORMAT_TEXT_V1,
				KEYMAP_COMPILE_NO_FLAGS,
			)?;
			Some(State::new(&keymap))
		}) {
			*receiver_key_state = state;
		};
		if let Some(keys_up) = &self.keys_up {
			for key_up in keys_up {
				receiver_key_state.update_key((*key_up).into(), KeyDirection::Up);
			}
		}
		if let Some(keys_down) = &self.keys_down {
			for key_down in keys_down {
				receiver_key_state.update_key((*key_down).into(), KeyDirection::Down);
			}
		}
	}

	pub fn send_to_panel(&self, panel: &PanelItem, surface: &SurfaceID) -> Result<(), NodeError> {
		if let Some(keymap) = &self.keymap {
			let ctx = Context::new(CONTEXT_NO_FLAGS);
			let xkb_keymap = Keymap::new_from_string(
				&ctx,
				keymap.clone(),
				KEYMAP_FORMAT_TEXT_V1,
				KEYMAP_COMPILE_NO_FLAGS,
			);
			if xkb_keymap.is_some() {
				panel.keyboard_set_keymap(&keymap)?;
			}
		}

		for key in self.keys_down.as_ref().unwrap_or(&Vec::new()) {
			panel.keyboard_key(surface, *key, true)?;
		}
		for key in self.keys_down.as_ref().unwrap_or(&Vec::new()) {
			panel.keyboard_key(surface, *key, false)?;
		}
		Ok(())
	}
}

pub type KeyboardPanelRelay = HandlerWrapper<PulseReceiver, KeyboardPanelHandler>;
pub struct KeyboardPanelHandler {
	panel: PanelItem,
	focus: SurfaceID,
}
impl KeyboardPanelHandler {
	pub fn create<Fi: Field>(
		parent: &Spatial,
		transform: Transform,
		field: &Fi,
		panel: &PanelItem,
		focus: SurfaceID,
	) -> Result<KeyboardPanelRelay, NodeError> {
		let panel = panel.alias();
		PulseReceiver::create(parent, transform, field, &KEYBOARD_MASK)?
			.wrap(KeyboardPanelHandler { panel, focus })
	}
}
impl PulseReceiverHandler for KeyboardPanelHandler {
	fn data(&mut self, _uid: &str, data: &[u8], _data_reader: flexbuffers::MapReader<&[u8]>) {
		let Some(keyboard_event) = KeyboardEvent::from_pulse_data(data) else {return};
		let _ = keyboard_event.send_to_panel(&self.panel, &self.focus);
	}
}

#[tokio::test]
async fn keyboard_events() {
	let (client, event_loop) = stardust_xr_fusion::client::Client::connect_with_async_loop()
		.await
		.unwrap();
	use stardust_xr_fusion::{core::values::Transform, node::NodeType};

	struct PulseReceiverTest {
		_client: std::sync::Arc<stardust_xr_fusion::client::Client>,
		state: xkb::State,
	}
	unsafe impl Send for PulseReceiverTest {}
	unsafe impl Sync for PulseReceiverTest {}
	impl stardust_xr_fusion::data::PulseReceiverHandler for PulseReceiverTest {
		fn data(&mut self, uid: &str, data: &[u8], _data_reader: flexbuffers::MapReader<&[u8]>) {
			let keyboard_event = KeyboardEvent::from_pulse_data(data).unwrap();
			println!("Pulse sender {} sent {:#?}", uid, keyboard_event);
			keyboard_event.update_xkb_state(&mut self.state);
			// self.client.stop_loop();
		}
	}
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

	let keymap = xkb::Keymap::new_from_names(
		&Context::new(0),
		"",
		"",
		"",
		"",
		None,
		xkb::ffi::XKB_KEYMAP_COMPILE_NO_FLAGS,
	)
	.unwrap();
	let mut keyboard_event_serializer = flexbuffers::FlexbufferSerializer::new();
	let keyboard_event = KeyboardEvent {
		keyboard: "xkbv1".to_string(),
		keymap: Some(keymap.get_as_string(xkb::ffi::XKB_KEYMAP_FORMAT_TEXT_V1)),
		keys_up: None,
		keys_down: Some(vec![1]),
	};
	keyboard_event
		.serialize(&mut keyboard_event_serializer)
		.unwrap();
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::default(), &KEYBOARD_MASK).unwrap();
	let pulse_sender_test = PulseSenderTest {
		data: keyboard_event_serializer.take_buffer(),
		node: pulse_sender.alias(),
	};
	let _pulse_sender = pulse_sender.wrap(pulse_sender_test).unwrap();
	let _pulse_receiver = PulseReceiver::create(
		client.get_root(),
		Transform::default(),
		&field,
		&KEYBOARD_MASK,
	)
	.unwrap()
	.wrap(PulseReceiverTest {
		_client: client.clone(),
		state: State::new(&keymap),
	});

	tokio::select! {
		_ = tokio::time::sleep(core::time::Duration::from_secs(60)) => panic!("Timed Out"),
		e = event_loop => e.unwrap().unwrap(),
	}
}
