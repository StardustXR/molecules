use mint::Vector2;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	data::{PulseReceiver, PulseSender},
	items::panel::PanelItem,
	node::NodeError,
};

lazy_static::lazy_static! {
	pub static ref MOUSE_MASK: Vec<u8> = {
		let mut fbb = flexbuffers::Builder::default();
		let mut map = fbb.start_map();
		map.push("mouse", "v1");
		map.end_map();
		fbb.take_buffer()
	};
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MouseEvent {
	pub mouse: String,
	pub delta: Option<Vector2<f32>>,
	pub scroll_distance: Option<Vector2<f32>>,
	pub scroll_steps: Option<Vector2<f32>>,
	pub buttons_up: Option<Vec<u32>>,
	pub buttons_down: Option<Vec<u32>>,
}
impl MouseEvent {
	pub fn new(
		delta: Option<Vector2<f32>>,
		scroll_distance: Option<Vector2<f32>>,
		scroll_steps: Option<Vector2<f32>>,
		buttons_up: Option<Vec<u32>>,
		buttons_down: Option<Vec<u32>>,
	) -> Self {
		MouseEvent {
			mouse: "v1".to_string(),
			delta,
			scroll_distance,
			scroll_steps,
			buttons_up,
			buttons_down,
		}
	}

	pub fn from_pulse_data(data: &[u8]) -> Option<Self> {
		flexbuffers::Reader::get_root(data).ok().and_then(|r| {
			MouseEvent::deserialize(r).ok().and_then(|ev| {
				if &ev.mouse == "v1" {
					Some(ev)
				} else {
					None
				}
			})
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

	/// Does not handle delta
	pub fn send_to_panel(&self, panel: &PanelItem) -> Result<(), NodeError> {
		if let Some(scroll_distance) = &self.scroll_distance {
			panel.pointer_scroll(*scroll_distance, Vector2::from([0.0; 2]))?;
		}
		if let Some(scroll_steps) = &self.scroll_steps {
			panel.pointer_scroll(Vector2::from([0.0; 2]), *scroll_steps)?;
		}
		if let Some(buttons_up) = &self.buttons_up {
			for button in buttons_up {
				panel.pointer_button(*button, false)?;
			}
		}
		if let Some(buttons_down) = &self.buttons_down {
			for button in buttons_down {
				panel.pointer_button(*button, true)?;
			}
		}
		Ok(())
	}
}

#[tokio::test]
async fn mouse_events() {
	let (client, event_loop) = stardust_xr_fusion::client::Client::connect_with_async_loop()
		.await
		.unwrap();
	use stardust_xr_fusion::{core::values::Transform, node::NodeType};

	struct PulseReceiverTest {
		_client: std::sync::Arc<stardust_xr_fusion::client::Client>,
	}
	unsafe impl Send for PulseReceiverTest {}
	unsafe impl Sync for PulseReceiverTest {}
	impl stardust_xr_fusion::data::PulseReceiverHandler for PulseReceiverTest {
		fn data(&mut self, uid: &str, data: &[u8], _data_reader: flexbuffers::MapReader<&[u8]>) {
			let mouse_event = MouseEvent::from_pulse_data(data).unwrap();
			println!("Pulse sender {} sent {:#?}", uid, mouse_event);
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

	let mut mouse_event_serializer = flexbuffers::FlexbufferSerializer::new();
	let mouse_event = MouseEvent {
		mouse: "v1".to_string(),
		delta: None,
		scroll_distance: None,
		scroll_steps: None,
		buttons_up: None,
		buttons_down: Some(vec![]),
	};
	mouse_event.serialize(&mut mouse_event_serializer).unwrap();
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::default(), &MOUSE_MASK).unwrap();
	let pulse_sender_test = PulseSenderTest {
		data: mouse_event_serializer.take_buffer(),
		node: pulse_sender.alias(),
	};
	let _pulse_sender = pulse_sender.wrap(pulse_sender_test).unwrap();
	let _pulse_receiver =
		PulseReceiver::create(client.get_root(), Transform::default(), &field, &MOUSE_MASK)
			.unwrap()
			.wrap(PulseReceiverTest {
				_client: client.clone(),
			});

	tokio::select! {
		_ = tokio::time::sleep(core::time::Duration::from_secs(60)) => panic!("Timed Out"),
		e = event_loop => e.unwrap().unwrap(),
	}
}
