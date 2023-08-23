use std::path::PathBuf;

use stardust_xr_fusion::{
	client::{FrameInfo, RootHandler},
	core::schemas::flex::flexbuffers::MapReader,
	data::{NewReceiverInfo, PulseReceiver, PulseReceiverHandler, PulseSenderHandler},
	fields::UnknownField,
	input::{InputData, InputHandlerHandler, UnknownInputMethod},
	items::{
		panel::{PanelItem, PanelItemHandler, PanelItemInitData, ToplevelInfo},
		EnvironmentItem, ItemAcceptorHandler,
	},
	spatial::{Spatial, ZoneHandler},
};

pub struct DummyHandler;
impl RootHandler for DummyHandler {
	fn frame(&mut self, _info: FrameInfo) {}
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
	fn commit_toplevel(&mut self, _state: Option<ToplevelInfo>) {}
}

// Spatial
impl ZoneHandler for DummyHandler {
	fn enter(&mut self, _uid: &str, _spatial: Spatial) {}
	fn capture(&mut self, _uid: &str, _spatial: Spatial) {}
	fn release(&mut self, _uid: &str) {}
	fn leave(&mut self, _uid: &str) {}
}
