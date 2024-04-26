use stardust_xr_fusion::{
	core::values::Datamap,
	data::{PulseReceiver, PulseReceiverHandler, PulseSenderHandler},
	fields::Field,
	input::{InputData, InputHandlerHandler, InputMethod},
	items::{
		panel::{ChildInfo, Geometry, PanelItem, PanelItemHandler, PanelItemInitData},
		ItemAcceptorHandler,
	},
	spatial::{Spatial, ZoneHandler},
};

pub struct DummyHandler;

// Input
impl InputHandlerHandler for DummyHandler {
	fn input(&mut self, _input: InputMethod, _data: InputData) {}
}

// Data
impl PulseSenderHandler for DummyHandler {
	fn new_receiver(&mut self, _uid: String, _receiver: PulseReceiver, _field: Field) {}
	fn drop_receiver(&mut self, _uid: String) {}
}
impl PulseReceiverHandler for DummyHandler {
	fn data(&mut self, _uid: String, _data: Datamap) {}
}

// Items
impl ItemAcceptorHandler<PanelItem> for DummyHandler {
	fn captured(&mut self, _uid: String, _item: PanelItem, _init_data: PanelItemInitData) {}
	fn released(&mut self, _uid: String) {}
}
impl PanelItemHandler for DummyHandler {
	fn toplevel_size_changed(&mut self, _size: mint::Vector2<u32>) {}
	fn new_child(&mut self, _uid: &str, _info: ChildInfo) {}
	fn reposition_child(&mut self, _uid: &str, _geometry: Geometry) {}
	fn drop_child(&mut self, _uid: &str) {}
}

// Spatial
impl ZoneHandler for DummyHandler {
	fn enter(&mut self, _uid: String, _spatial: Spatial) {}
	fn capture(&mut self, _uid: String) {}
	fn release(&mut self, _uid: String) {}
	fn leave(&mut self, _uid: String) {}
}
