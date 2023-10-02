use mint::Vector2;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	core::schemas::flex::flexbuffers,
	data::{PulseReceiver, PulseSender},
	items::panel::{PanelItem, SurfaceID},
	node::NodeError,
};

use crate::datamap::Datamap;

lazy_static::lazy_static! {
	pub static ref MOUSE_MASK: Vec<u8> = Datamap::create(MouseEvent::default()).serialize();
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct MouseEvent {
	pub mouse: (),
	pub v1: (),
	pub delta: Option<Vector2<f32>>,
	pub scroll_continuous: Option<Vector2<f32>>,
	pub scroll_discrete: Option<Vector2<f32>>,
	pub buttons_up: Option<Vec<u32>>,
	pub buttons_down: Option<Vec<u32>>,
}
impl MouseEvent {
	pub fn new(
		delta: Option<Vector2<f32>>,
		scroll_continuous: Option<Vector2<f32>>,
		scroll_discrete: Option<Vector2<f32>>,
		buttons_up: Option<Vec<u32>>,
		buttons_down: Option<Vec<u32>>,
	) -> Self {
		MouseEvent {
			mouse: (),
			v1: (),
			delta,
			scroll_continuous,
			scroll_discrete,
			buttons_up,
			buttons_down,
		}
	}

	pub fn from_pulse_data(data: &[u8]) -> Option<Self> {
		flexbuffers::Reader::get_root(data)
			.ok()
			.and_then(|r| MouseEvent::deserialize(r).ok())
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
	pub fn send_to_panel(&self, panel: &PanelItem, surface: &SurfaceID) -> Result<(), NodeError> {
		if let Some(scroll_distance) = &self.scroll_continuous {
			panel.pointer_scroll(surface, Some(*scroll_distance), None)?;
		}
		if let Some(scroll_steps) = &self.scroll_discrete {
			panel.pointer_scroll(surface, None, Some(*scroll_steps))?;
		}
		if let Some(buttons_up) = &self.buttons_up {
			for button in buttons_up {
				panel.pointer_button(surface, *button, false)?;
			}
		}
		if let Some(buttons_down) = &self.buttons_down {
			for button in buttons_down {
				panel.pointer_button(surface, *button, true)?;
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
		mouse: (),
		v1: (),
		delta: None,
		scroll_continuous: None,
		scroll_discrete: None,
		buttons_up: None,
		buttons_down: Some(vec![]),
	};
	mouse_event.serialize(&mut mouse_event_serializer).unwrap();
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::none(), &MOUSE_MASK).unwrap();
	let pulse_sender_test = PulseSenderTest {
		data: mouse_event_serializer.take_buffer(),
		node: pulse_sender.alias(),
	};
	let _pulse_sender = pulse_sender.wrap(pulse_sender_test).unwrap();
	let _pulse_receiver = crate::data::SimplePulseReceiver::create(
		client.get_root(),
		Transform::none(),
		&field,
		|uid, mouse_event: MouseEvent| println!("Pulse sender {} sent {:#?}", uid, mouse_event),
	);

	tokio::select! {
		_ = tokio::time::sleep(core::time::Duration::from_secs(60)) => panic!("Timed Out"),
		e = event_loop => e.unwrap().unwrap(),
	}
}
