use crate::dummy::DummyHandler;
use serde::{de::DeserializeOwned, Serialize};
use stardust_xr_fusion::{
	core::values::Datamap,
	data::{PulseReceiver, PulseReceiverAspect, PulseReceiverHandler},
	fields::FieldAspect,
	node::NodeError,
	spatial::{SpatialAspect, Transform},
	HandlerWrapper,
};

/// A simple pulse receiver that runs a closure whenever it gets data, type for schema convenience.
pub struct SimplePulseReceiver<T: Serialize + DeserializeOwned + Default + 'static>(
	HandlerWrapper<PulseReceiver, InlineHandler<T>>,
);
impl<T: Serialize + DeserializeOwned + Default + 'static> SimplePulseReceiver<T> {
	pub fn create<F: FnMut(String, T) + Send + Sync + 'static>(
		spatial_parent: &impl SpatialAspect,
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
	Box<dyn FnMut(String, T) + Send + Sync + 'static>,
);
impl<T: Serialize + DeserializeOwned + Default + 'static> PulseReceiverHandler
	for InlineHandler<T>
{
	fn data(&mut self, uid: String, data: Datamap) {
		let Ok(data) = data.deserialize() else { return };
		(self.0)(uid, data)
	}
}

/// Pulse receiver that only acts as a tag, doesn't
pub struct NodeTag(HandlerWrapper<PulseReceiver, DummyHandler>);
impl NodeTag {
	pub fn create<T: Serialize + Default>(
		spatial_parent: &impl SpatialAspect,
		transform: Transform,
		field: &impl FieldAspect,
	) -> Result<Self, NodeError> {
		let mask = Datamap::from_typed(T::default()).map_err(|_| NodeError::Serialization)?;
		Ok(NodeTag(
			PulseReceiver::create(spatial_parent, transform, field, &mask)?.wrap(DummyHandler)?,
		))
	}
}
