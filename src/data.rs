use crate::dummy::DummyHandler;
use serde::{de::DeserializeOwned, Serialize};
use stardust_xr_fusion::{
	core::{
		schemas::flex::flexbuffers::{FlexbufferSerializer, MapReader, Reader},
		values::Transform,
	},
	data::{PulseReceiver, PulseReceiverHandler},
	fields::Field,
	node::NodeError,
	spatial::Spatial,
	HandlerWrapper,
};

/// A simple pulse receiver that runs a closure whenever it gets data, type for schema convenience.
pub struct SimplePulseReceiver<T: Serialize + DeserializeOwned + Default + 'static>(
	HandlerWrapper<PulseReceiver, InlineHandler<T>>,
);
impl<T: Serialize + DeserializeOwned + Default + 'static> SimplePulseReceiver<T> {
	pub fn create<Fi: Field, F: FnMut(&str, T) + Send + Sync + 'static>(
		spatial_parent: &Spatial,
		transform: Transform,
		field: &Fi,
		on_data: F,
	) -> Result<Self, NodeError> {
		let mut mask_serializer = FlexbufferSerializer::new();
		T::default()
			.serialize(&mut mask_serializer)
			.map_err(|_| NodeError::Serialization)?;
		Ok(SimplePulseReceiver(
			PulseReceiver::create(spatial_parent, transform, field, mask_serializer.view())?
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
	Box<dyn FnMut(&str, T) + Send + Sync + 'static>,
);
impl<T: Serialize + DeserializeOwned + Default + 'static> PulseReceiverHandler
	for InlineHandler<T>
{
	fn data(&mut self, uid: &str, data: &[u8], _data_reader: MapReader<&[u8]>) {
		let Ok(root) = Reader::get_root(data) else {return};
		let Ok(data) = T::deserialize(root) else {return};
		(self.0)(uid, data)
	}
}

/// Pulse receiver that only acts as a tag, doesn't
pub struct NodeTag(HandlerWrapper<PulseReceiver, DummyHandler>);
impl NodeTag {
	pub fn create<T: Serialize + Default, Fi: Field>(
		spatial_parent: &Spatial,
		transform: Transform,
		field: &Fi,
	) -> Result<Self, NodeError> {
		let mut mask_serializer = FlexbufferSerializer::new();
		T::default()
			.serialize(&mut mask_serializer)
			.map_err(|_| NodeError::Serialization)?;

		// check if the mask is a map or not
		{
			let flex_root =
				Reader::get_root(mask_serializer.view()).map_err(|_| NodeError::Serialization)?;
			let _map_reader = flex_root.get_map().map_err(|_| NodeError::ReturnedError {
				e: "Mask is not a map".to_string(),
			})?;
		}

		Ok(NodeTag(
			PulseReceiver::create(spatial_parent, transform, field, mask_serializer.view())?
				.wrap(DummyHandler)?,
		))
	}
}
