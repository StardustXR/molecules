use std::path::PathBuf;

use stardust_xr_fusion::{
	client::{ClientState, FrameInfo, RootHandler},
	core::schemas::flex::flexbuffers::MapReader,
	data::{NewReceiverInfo, PulseReceiver, PulseReceiverHandler, PulseSenderHandler},
	fields::UnknownField,
	input::{InputData, InputHandlerHandler, UnknownInputMethod},
	items::{
		panel::{ChildInfo, Geometry, PanelItem, PanelItemHandler, PanelItemInitData},
		EnvironmentItem, ItemAcceptorHandler,
	},
	spatial::{Spatial, ZoneHandler},
};

pub struct DummyHandler;
impl RootHandler for DummyHandler {
	fn frame(&mut self, _info: FrameInfo) {}
	fn save_state(&mut self) -> ClientState {
		ClientState::default()
	}
}

// Input
impl InputHandlerHandler for DummyHandler {
	fn input(&mut self, _input: UnknownInputMethod, _data: InputData) {}
}

// Data
impl PulseSenderHandler for DummyHandler {
	fn new_receiver(
		&mut self,
		_info: NewReceiverInfo,
		_receiver: PulseReceiver,
		_field: UnknownField,
	) {
	}
	fn drop_receiver(&mut self, _uid: &str) {}
}
impl PulseReceiverHandler for DummyHandler {
	fn data(&mut self, _uid: &str, _data: &[u8], _data_reader: MapReader<&[u8]>) {}
}

// Items
impl ItemAcceptorHandler<EnvironmentItem> for DummyHandler {
	fn captured(&mut self, _uid: &str, _item: EnvironmentItem, _init_data: PathBuf) {}
	fn released(&mut self, _uid: &str) {}
}
impl ItemAcceptorHandler<PanelItem> for DummyHandler {
	fn captured(&mut self, _uid: &str, _item: PanelItem, _init_data: PanelItemInitData) {}
	fn released(&mut self, _uid: &str) {}
}
impl PanelItemHandler for DummyHandler {
	fn toplevel_size_changed(&mut self, _size: mint::Vector2<u32>) {}
	fn new_child(&mut self, _uid: &str, _info: ChildInfo) {}
	fn reposition_child(&mut self, _uid: &str, _geometry: Geometry) {}
	fn drop_child(&mut self, _uid: &str) {}
}

// Spatial
impl ZoneHandler for DummyHandler {
	fn enter(&mut self, _uid: &str, _spatial: Spatial) {}
	fn capture(&mut self, _uid: &str, _spatial: Spatial) {}
	fn release(&mut self, _uid: &str) {}
	fn leave(&mut self, _uid: &str) {}
}
