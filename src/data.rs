use crate::dummy::DummyHandler;
use serde::{de::DeserializeOwned, Serialize};
use stardust_xr_fusion::{
	core::values::Datamap,
	data::{PulseReceiver, PulseReceiverAspect, PulseReceiverHandler},
	fields::FieldAspect,
	node::NodeError,
	spatial::{SpatialRef, SpatialRefAspect, Transform},
	HandlerWrapper,
};

/// A simple pulse receiver that runs a closure whenever it gets data, type for schema convenience.
pub struct SimplePulseReceiver<T: Serialize + DeserializeOwned + Default + 'static>(
	HandlerWrapper<PulseReceiver, InlineHandler<T>>,
);
impl<T: Serialize + DeserializeOwned + Default + 'static> SimplePulseReceiver<T> {
	pub fn create<F: FnMut(SpatialRef, T) + Send + Sync + 'static>(
		spatial_parent: &impl SpatialRefAspect,
		transform: Transform,
		field: &impl FieldAspect,
		on_data: F,
	) -> Result<Self, NodeError> {
		Ok(SimplePulseReceiver(
			PulseReceiver::create(
				spatial_parent,
				transform,
				field,
				&Datamap::from_typed(T::default()).map_err(|_| NodeError::Serialization)?,
			)?
			.wrap(InlineHandler(Box::new(on_data)))?,
		))
	}
}
impl<T: Serialize + DeserializeOwned + Default + 'static> std::ops::Deref
	for SimplePulseReceiver<T>
{
	type Target = PulseReceiver;

	fn deref(&self) -> &Self::Target {
		self.0.node()
	}
}

struct InlineHandler<T: Serialize + DeserializeOwned + Default + 'static>(
	Box<dyn FnMut(SpatialRef, T) + Send + Sync + 'static>,
);
impl<T: Serialize + DeserializeOwned + Default + 'static> PulseReceiverHandler
	for InlineHandler<T>
{
	fn data(&mut self, sender: SpatialRef, data: Datamap) {
		let Ok(data) = data.deserialize() else { return };
		(self.0)(sender, data)
	}
}

/// Pulse receiver that only acts as a tag, doesn't
pub type NodeTag = HandlerWrapper<PulseReceiver, DummyHandler>;
pub fn create_node_tag<T: Serialize + Default>(
	spatial_parent: &impl SpatialRefAspect,
	transform: Transform,
	field: &impl FieldAspect,
) -> Result<NodeTag, NodeError> {
	let mask = Datamap::from_typed(T::default()).map_err(|_| NodeError::Serialization)?;
	PulseReceiver::create(spatial_parent, transform, field, &mask)?.wrap(DummyHandler)
}
