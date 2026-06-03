use std::path::Path;

use gluon_codegen::{Derives, ModuleExternalProtocol};

fn main() {
	gluon_codegen::helpers::gen_multiple_modules(
		&[
			("keyboard", Path::new("./gluon/org.stardustxr.XKBv1.gluon")),
			("mouse", Path::new("./gluon/org.stardustxr.Mousev1.gluon")),
			(
				"reparentable",
				Path::new("./gluon/org.stardustxr.Reparentable.gluon"),
			),
			(
				"derezzable",
				Path::new("./gluon/org.stardustxr.Derezzable.gluon"),
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
		],
		Derives::all(),
		&[],
		true,
		"./src/protocol",
	);
}
