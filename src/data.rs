use stardust_xr_fusion::{
	core::{schemas::flex::flexbuffers::MapReader, values::Transform},
	data::{PulseReceiver, PulseReceiverHandler},
	fields::Field,
	node::NodeError,
	spatial::Spatial,
	HandlerWrapper,
};

pub struct InlinePulseReceiver(HandlerWrapper<PulseReceiver, InlineHandler>);
impl InlinePulseReceiver {
	pub fn create<Fi: Field, F: FnMut(&str, &[u8], MapReader<&[u8]>) + Send + Sync + 'static>(
		spatial_parent: &Spatial,
		transform: Transform,
		field: &Fi,
		mask: &[u8],
		on_data: F,
	) -> Result<Self, NodeError> {
		Ok(InlinePulseReceiver(
			PulseReceiver::create(spatial_parent, transform, field, mask)?
				.wrap(InlineHandler(Box::new(on_data)))?,
		))
	}
}
impl std::ops::Deref for InlinePulseReceiver {
	type Target = PulseReceiver;

	fn deref(&self) -> &Self::Target {
		self.0.node()
	}
}

struct InlineHandler(Box<dyn FnMut(&str, &[u8], MapReader<&[u8]>) + Send + Sync + 'static>);
impl PulseReceiverHandler for InlineHandler {
	fn data(&mut self, uid: &str, data: &[u8], data_reader: MapReader<&[u8]>) {
		(self.0)(uid, data, data_reader)
	}
}
