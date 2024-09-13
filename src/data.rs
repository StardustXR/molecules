use serde::{de::DeserializeOwned, Serialize};
use stardust_xr_fusion::{
	core::values::Datamap,
	data::{PulseReceiver, PulseReceiverAspect, PulseReceiverEvent},
	fields::FieldAspect,
	node::NodeError,
	spatial::{SpatialRef, SpatialRefAspect, Transform},
};

use crate::UIElement;

/// A simple pulse receiver that runs a closure whenever it gets data, type for schema convenience.
pub struct SimplePulseReceiver<T: Serialize + DeserializeOwned + Default + 'static> {
	receiver: PulseReceiver,
	handler: Box<dyn FnMut(SpatialRef, T) + Send + Sync + 'static>,
}
impl<T: Serialize + DeserializeOwned + Default + 'static> SimplePulseReceiver<T> {
	pub fn create<F: FnMut(SpatialRef, T) + Send + Sync + 'static>(
		spatial_parent: &impl SpatialRefAspect,
		transform: Transform,
		field: &impl FieldAspect,
		handler: F,
	) -> Result<Self, NodeError> {
		Ok(SimplePulseReceiver {
			receiver: PulseReceiver::create(
				spatial_parent,
				transform,
				field,
				&Datamap::from_typed(T::default()).map_err(|_| NodeError::Serialization)?,
			)?,
			handler: Box::new(handler),
		})
	}
}
impl<T: Serialize + DeserializeOwned + Default + 'static> UIElement for SimplePulseReceiver<T> {
	fn handle_events(&mut self) -> bool {
		let mut handled = false;
		while let Some(PulseReceiverEvent::Data { sender, data }) = self.receiver.recv_event() {
			handled = true;
			let Ok(data) = data.deserialize() else {
				return true;
			};
			(self.handler)(sender, data)
		}
		handled
	}
}
impl<T: Serialize + DeserializeOwned + Default + 'static> std::ops::Deref
	for SimplePulseReceiver<T>
{
	type Target = PulseReceiver;

	fn deref(&self) -> &Self::Target {
		&self.receiver
	}
}

/// Pulse receiver that only acts as a tag, doesn't
pub type NodeTag = PulseReceiver;
pub fn create_node_tag<T: Serialize + Default>(
	spatial_parent: &impl SpatialRefAspect,
	transform: Transform,
	field: &impl FieldAspect,
) -> Result<NodeTag, NodeError> {
	let mask = Datamap::from_typed(T::default()).map_err(|_| NodeError::Serialization)?;
	PulseReceiver::create(spatial_parent, transform, field, &mask)
}
