use serde::{de::DeserializeOwned, Serialize};
use stardust_xr_fusion::{
	core::schemas::flex::flexbuffers::{DeserializationError, FlexbufferSerializer, Reader},
	input::InputMethod,
	node::NodeError,
};

pub struct Datamap<D: Serialize + DeserializeOwned> {
	serializer: FlexbufferSerializer,
	data: D,
}
impl<D: Serialize + DeserializeOwned> Datamap<D> {
	pub fn create(data: D) -> Self {
		Datamap {
			serializer: FlexbufferSerializer::new(),
			data,
		}
	}
	pub fn from_data(data: &[u8]) -> Result<Self, DeserializationError> {
		let data = D::deserialize(Reader::get_root(data)?)?;
		Ok(Datamap {
			serializer: FlexbufferSerializer::new(),
			data,
		})
	}
	pub fn update_input_method(&mut self, input_method: &InputMethod) -> Result<(), NodeError> {
		input_method.set_datamap(&self.serialize())
	}

	pub fn serialize(&mut self) -> Vec<u8> {
		self.data.serialize(&mut self.serializer).unwrap();
		self.serializer.take_buffer()
	}
}
