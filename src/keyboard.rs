use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	data::{PulseReceiver, PulseSender},
	items::panel::PanelItem,
	node::NodeError,
};
pub use xkbcommon::xkb;
use xkbcommon::xkb::{
	Context, KeyDirection, Keymap, State, CONTEXT_NO_FLAGS, KEYMAP_COMPILE_NO_FLAGS,
	KEYMAP_FORMAT_TEXT_V1,
};

lazy_static::lazy_static! {
	pub static ref KEYBOARD_MASK: Vec<u8> = {
		let mut fbb = flexbuffers::Builder::default();
		let mut map = fbb.start_map();
		map.push("keyboard", "xkbv1");
		map.end_map();
		fbb.take_buffer()
	};
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyboardEvent {
	keyboard: String,
	keymap: Option<String>,
	keys_up: Option<Vec<u32>>,
	keys_down: Option<Vec<u32>>,
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
				receiver_key_state.update_key(*key_up, KeyDirection::Up);
			}
		}
		if let Some(keys_down) = &self.keys_down {
			for key_down in keys_down {
				receiver_key_state.update_key(*key_down, KeyDirection::Down);
			}
		}
	}

	pub fn send_to_panel(&self, panel: &PanelItem) -> Result<(), NodeError> {
		if let Some(keymap) = &self.keymap {
			let ctx = Context::new(CONTEXT_NO_FLAGS);
			let xkb_keymap = Keymap::new_from_string(
				&ctx,
				keymap.clone(),
				KEYMAP_FORMAT_TEXT_V1,
				KEYMAP_COMPILE_NO_FLAGS,
			);
			if xkb_keymap.is_some() {
				panel.keyboard_deactivate()?;
				panel.keyboard_activate(&keymap)?;
			}
		}
		if let Some(keys_up) = &self.keys_up {
			for key in keys_up {
				panel.keyboard_key_state(*key, false)?;
			}
		}
		if let Some(keys_down) = &self.keys_down {
			for key in keys_down {
				panel.keyboard_key_state(*key, true)?;
			}
		}
		Ok(())
	}
}

#[tokio::test]
async fn keyboard_events() {
	let (client, event_loop) = stardust_xr_fusion::client::Client::connect_with_async_loop()
		.await
		.unwrap();
	use stardust_xr_fusion::node::NodeType;
	use std::sync::Arc;

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
		node: Arc<PulseSender>,
	}
	impl stardust_xr_fusion::data::PulseSenderHandler for PulseSenderTest {
		fn new_receiver(
			&mut self,
			receiver: &PulseReceiver,
			field: &stardust_xr_fusion::fields::UnknownField,
			info: stardust_xr_fusion::data::NewReceiverInfo,
		) {
			println!(
				"New pulse receiver {:?} with field {:?} and info {:?}",
				receiver.node().get_path(),
				field.node().get_path(),
				info
			);
			self.node.send_data(receiver, &self.data).unwrap();
		}
		fn drop_receiver(&mut self, uid: &str) {
			println!("Pulse receiver {} dropped", uid);
		}
	}

	let field = stardust_xr_fusion::fields::SphereField::builder()
		.spatial_parent(client.get_root())
		.radius(0.1)
		.build()
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
	let _pulse_sender = PulseSender::create(
		client.get_root(),
		None,
		None,
		KEYBOARD_MASK.clone(),
		|node| PulseSenderTest {
			data: keyboard_event_serializer.take_buffer(),
			node: node.clone(),
		},
	)
	.unwrap();
	let _pulse_receiver = PulseReceiver::create(
		client.get_root(),
		None,
		None,
		&field,
		KEYBOARD_MASK.clone(),
		|_| PulseReceiverTest {
			_client: client.clone(),
			state: State::new(&keymap),
		},
	)
	.unwrap();

	tokio::select! {
		_ = tokio::time::sleep(core::time::Duration::from_secs(60)) => panic!("Timed Out"),
		e = event_loop => e.unwrap().unwrap(),
	}
}
