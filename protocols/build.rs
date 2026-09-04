use std::path::Path;

use gluon_codegen::{Derives, ModuleExternalProtocol};

fn main() {
	gluon_codegen::helpers::gen_multiple_modules(
		&[
			(
				"container",
				Path::new("./gluon/org.stardustxr.Container.gluon"),
			),
			(
				"derezzable",
				Path::new("./gluon/org.stardustxr.Derezzable.gluon"),
			),
			(
				"environment",
				Path::new("./gluon/org.stardustxr.Environment.gluon"),
			),
			(
				"keyboard_handler",
				Path::new("./gluon/org.stardustxr.KeyboardHandler.gluon"),
			),
			(
				"mouse_handler",
				Path::new("./gluon/org.stardustxr.MouseHandler.gluon"),
			),
			(
				"transformable",
				Path::new("./gluon/org.stardustxr.Transformable.gluon"),
			),
		],
		&[
			&ModuleExternalProtocol {
				rust_module: "stardust_xr_protocol::types",
				external_protocol: stardust_xr_protocol::types::EXTERNAL_PROTOCOL,
			},
			&ModuleExternalProtocol {
				rust_module: "stardust_xr_protocol::spatial",
				external_protocol: stardust_xr_protocol::spatial::EXTERNAL_PROTOCOL,
			},
			&ModuleExternalProtocol {
				rust_module: "stardust_xr_protocol::keymap",
				external_protocol: stardust_xr_protocol::keymap::EXTERNAL_PROTOCOL,
			},
		],
		Derives::all(),
		&[],
		true,
		"./src/protocol",
	);
}
