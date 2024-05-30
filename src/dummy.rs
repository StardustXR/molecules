#![allow(unused)]

use stardust_xr_fusion::{
	core::values::{Datamap, Vector2},
	data::{PulseReceiver, PulseReceiverHandler, PulseSenderHandler},
	fields::Field,
	input::{InputData, InputHandler, InputHandlerHandler, InputMethod, InputMethodHandler},
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
	fn input(&mut self, _input: InputMethod, _data: InputData) {}
}
impl InputMethodHandler for DummyHandler {
	fn create_handler(&mut self, uid: String, handler: InputHandler, field: Field) {}
	fn request_capture_handler(&mut self, uid: String) {}
	fn destroy_handler(&mut self, uid: String) {}
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
impl ItemUiHandler for DummyHandler {
	fn capture_item(&mut self, item_uid: String, acceptor_uid: String) {}
	fn release_item(&mut self, item_uid: String, acceptor_uid: String) {}
	fn destroy_item(&mut self, uid: String) {}
	fn destroy_acceptor(&mut self, uid: String) {}
}
impl ItemAcceptorHandler for DummyHandler {
	fn release_item(&mut self, uid: String) {}
}

impl CameraItemUiHandler for DummyHandler {
	fn create_item(&mut self, uid: String, item: CameraItem) {}
	fn create_acceptor(
		&mut self,
		uid: String,
		acceptor: CameraItemAcceptor,
		acceptor_field: Field,
	) {
	}
}
impl CameraItemAcceptorHandler for DummyHandler {
	fn capture_item(&mut self, uid: String, item: CameraItem) {}
}

impl PanelItemAcceptorHandler for DummyHandler {
	fn capture_item(&mut self, uid: String, item: PanelItem, initial_data: PanelItemInitData) {}
}
impl PanelItemHandler for DummyHandler {
	fn toplevel_parent_changed(&mut self, parent_uid: String) {}
	fn toplevel_title_changed(&mut self, title: String) {}
	fn toplevel_app_id_changed(&mut self, app_id: String) {}
	fn toplevel_fullscreen_active(&mut self, active: bool) {}
	fn toplevel_move_request(&mut self) {}
	fn toplevel_resize_request(&mut self, up: bool, down: bool, left: bool, right: bool) {}
	fn toplevel_size_changed(&mut self, size: Vector2<u32>) {}
	fn set_cursor(&mut self, geometry: Geometry) {}
	fn hide_cursor(&mut self) {}
	fn create_child(&mut self, uid: String, info: ChildInfo) {}
	fn reposition_child(&mut self, uid: String, geometry: Geometry) {}
	fn destroy_child(&mut self, uid: String) {}
}

// Spatial
impl ZoneHandler for DummyHandler {
	fn enter(&mut self, _uid: String, _spatial: Spatial) {}
	fn capture(&mut self, _uid: String) {}
	fn release(&mut self, _uid: String) {}
	fn leave(&mut self, _uid: String) {}
}
