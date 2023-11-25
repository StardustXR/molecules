use serde::{de::DeserializeOwned, Serialize};
use stardust_xr_fusion::{
	core::schemas::flex::flexbuffers::{self, DeserializationError},
	input::InputMethod,
	node::NodeError,
};

pub struct Datamap<D: Serialize + DeserializeOwned>(D);
impl<D: Serialize + DeserializeOwned> Datamap<D> {
	pub fn create(data: D) -> Self {
		Datamap(data)
	}
	pub fn from_data(data: &[u8]) -> Result<Self, DeserializationError> {
		flexbuffers::from_slice(data).map(|f| Datamap(f))
	}
	pub fn update_input_method(
		&mut self,
		input_method: &impl InputMethod,
	) -> Result<(), NodeError> {
		input_method.set_datamap(&self.serialize())
	}

	pub fn data(&mut self) -> &mut D {
		&mut self.0
	}

	pub fn serialize(&mut self) -> Vec<u8> {
		flexbuffers::to_vec(&mut self.data()).unwrap()
	}
}
