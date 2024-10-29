use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	core::{
		schemas::flex::flexbuffers,
		values::{Datamap, Vector2},
	},
	data::{PulseReceiver, PulseReceiverAspect, PulseSender},
};

lazy_static::lazy_static! {
	pub static ref MOUSE_MASK: Datamap = Datamap::from_typed(MouseEvent::default()).unwrap();
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct MouseEvent {
	pub mouse: (),
	pub v1: (),
	pub delta: Option<Vector2<f32>>,
	pub scroll_continuous: Option<Vector2<f32>>,
	pub scroll_discrete: Option<Vector2<f32>>,
	pub raw_input_events: Option<FxHashSet<u32>>,
}
impl MouseEvent {
	pub fn new(
		delta: Option<Vector2<f32>>,
		scroll_continuous: Option<Vector2<f32>>,
		scroll_discrete: Option<Vector2<f32>>,
		raw_input_events: Option<FxHashSet<u32>>,
	) -> Self {
		MouseEvent {
			mouse: (),
			v1: (),
			delta,
			scroll_continuous,
			scroll_discrete,
			raw_input_events,
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
		let data = Datamap::from_typed(self).unwrap();
		for receiver in receivers.iter() {
			let _ = receiver.send_data(sender, &data);
		}
	}
}

#[tokio::test]
async fn mouse_events() {
	use crate::data::SimplePulseReceiver;
	use crate::UIElement;
	use stardust_xr_fusion::data::PulseSenderAspect;
	use stardust_xr_fusion::data::PulseSenderEvent;
	use stardust_xr_fusion::fields::{Field, Shape};
	use stardust_xr_fusion::node::NodeType;
	use stardust_xr_fusion::spatial::Transform;
	use std::sync::Arc;

	let mut client = stardust_xr_fusion::Client::connect().await.unwrap();

	let field = Arc::new(
		Field::create(client.get_root(), Transform::identity(), Shape::Sphere(0.1)).unwrap(),
	);

	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::none(), &MOUSE_MASK).unwrap();
	let mut pulse_receiver = None;
	let event_loop = client.sync_event_loop(move |client, _flow| {
		let pulse_receiver = pulse_receiver.get_or_insert_with({
			let client = client.clone();
			let field = field.clone();
			move || {
				SimplePulseReceiver::create(
					client.get_root(),
					Transform::none(),
					field.as_ref(),
					move |sender, mouse_event: MouseEvent| {
						println!(
							"Pulse sender {} sent {:#?}",
							sender.node().id(),
							mouse_event
						);
					},
				)
				.unwrap()
			}
		});
		pulse_receiver.handle_events();

		match pulse_sender.recv_pulse_sender_event() {
			Some(PulseSenderEvent::NewReceiver { receiver, field }) => {
				println!(
					"New pulse receiver {:?} with field {:?}",
					receiver.node().id(),
					field.node().id(),
				);
				receiver.send_data(&pulse_sender, &MOUSE_MASK).unwrap();
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
