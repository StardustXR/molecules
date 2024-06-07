#![allow(unused)]

use stardust_xr_fusion::{
	core::values::{Datamap, Vector2},
	data::{PulseReceiver, PulseReceiverHandler, PulseSenderHandler},
	fields::Field,
	input::{
		InputData, InputHandler, InputHandlerHandler, InputMethod, InputMethodHandler,
		InputMethodRef,
	},
	items::{
		camera::{CameraItem, CameraItemAcceptor, CameraItemAcceptorHandler, CameraItemUiHandler},
		panel::{
			ChildInfo, Geometry, PanelItem, PanelItemAcceptorHandler, PanelItemHandler,
			PanelItemInitData,
		},
		ItemAcceptorHandler, ItemUiHandler,
	},
	spatial::{Spatial, SpatialRef, ZoneHandler},
};

pub struct DummyHandler;

// Input
impl InputHandlerHandler for DummyHandler {
	fn input(&mut self, _input: Vec<InputMethodRef>, _data: Vec<InputData>) {}
}
impl InputMethodHandler for DummyHandler {
	fn create_handler(&mut self, handler: InputHandler, field: Field) {}
	fn request_capture_handler(&mut self, id: u64) {}
	fn destroy_handler(&mut self, id: u64) {}
}

// Data
impl PulseSenderHandler for DummyHandler {
	fn new_receiver(&mut self, _receiver: PulseReceiver, _field: Field) {}
	fn drop_receiver(&mut self, _id: u64) {}
}
impl PulseReceiverHandler for DummyHandler {
	fn data(&mut self, sender: SpatialRef, _data: Datamap) {}
}

// Items
impl ItemUiHandler for DummyHandler {
	fn capture_item(&mut self, item_id: u64, acceptor_id: u64) {}
	fn release_item(&mut self, item_id: u64, acceptor_id: u64) {}
	fn destroy_item(&mut self, id: u64) {}
	fn destroy_acceptor(&mut self, id: u64) {}
}
impl ItemAcceptorHandler for DummyHandler {
	fn release_item(&mut self, id: u64) {}
}

impl CameraItemUiHandler for DummyHandler {
	fn create_item(&mut self, item: CameraItem) {}
	fn create_acceptor(&mut self, acceptor: CameraItemAcceptor, acceptor_field: Field) {}
}
impl CameraItemAcceptorHandler for DummyHandler {
	fn capture_item(&mut self, item: CameraItem) {}
}

impl PanelItemAcceptorHandler for DummyHandler {
	fn capture_item(&mut self, item: PanelItem, initial_data: PanelItemInitData) {}
}
impl PanelItemHandler for DummyHandler {
	fn toplevel_parent_changed(&mut self, parent_id: u64) {}
	fn toplevel_title_changed(&mut self, title: String) {}
	fn toplevel_app_id_changed(&mut self, app_id: String) {}
	fn toplevel_fullscreen_active(&mut self, active: bool) {}
	fn toplevel_move_request(&mut self) {}
	fn toplevel_resize_request(&mut self, up: bool, down: bool, left: bool, right: bool) {}
	fn toplevel_size_changed(&mut self, size: Vector2<u32>) {}
	fn set_cursor(&mut self, geometry: Geometry) {}
	fn hide_cursor(&mut self) {}
	fn create_child(&mut self, id: u64, info: ChildInfo) {}
	fn reposition_child(&mut self, id: u64, geometry: Geometry) {}
	fn destroy_child(&mut self, id: u64) {}
}

// Spatial
impl ZoneHandler for DummyHandler {
	fn enter(&mut self, _spatial: SpatialRef) {}
	fn capture(&mut self, _spatial: Spatial) {}
	fn release(&mut self, _id: u64) {}
	fn leave(&mut self, _id: u64) {}
}
