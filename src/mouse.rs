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
		for receiver in receivers.into_iter() {
			let _ = receiver.send_data(sender, &data);
		}
	}
}

#[tokio::test]
async fn mouse_events() {
	let (client, event_loop) = stardust_xr_fusion::client::Client::connect_with_async_loop()
		.await
		.unwrap();
	use stardust_xr_fusion::{data::PulseSenderAspect, node::NodeType, spatial::Transform};
	struct PulseSenderTest {
		data: Datamap,
		node: PulseSender,
	}
	impl stardust_xr_fusion::data::PulseSenderHandler for PulseSenderTest {
		fn new_receiver(
			&mut self,
			receiver: PulseReceiver,
			field: stardust_xr_fusion::fields::Field,
		) {
			println!(
				"New pulse receiver {:?} with field {:?}",
				receiver.node().get_id(),
				field.node().get_id(),
			);
			receiver.send_data(&self.node, &self.data).unwrap();
		}
		fn drop_receiver(&mut self, id: u64) {
			println!("Pulse receiver {} dropped", id);
		}
	}

	let field = stardust_xr_fusion::fields::Field::create(
		client.get_root(),
		Transform::identity(),
		stardust_xr_fusion::fields::Shape::Sphere(0.1),
	)
	.unwrap();
	let pulse_sender =
		PulseSender::create(client.get_root(), Transform::none(), &MOUSE_MASK).unwrap();
	let pulse_sender_test = PulseSenderTest {
		data: MOUSE_MASK.clone(),
		node: pulse_sender.alias(),
	};
	let _pulse_sender = pulse_sender.wrap(pulse_sender_test).unwrap();
	let _pulse_receiver = crate::data::SimplePulseReceiver::create(
		client.get_root(),
		Transform::none(),
		&field,
		|sender, mouse_event: MouseEvent| {
			println!(
				"Pulse sender {} sent {:#?}",
				sender.node().get_id().unwrap(),
				mouse_event
			)
		},
	);

	tokio::select! {
		_ = tokio::time::sleep(core::time::Duration::from_secs(60)) => panic!("Timed Out"),
		e = event_loop => e.unwrap().unwrap(),
	}
}
