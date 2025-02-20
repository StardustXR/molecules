#[zbus::proxy(interface = "org.stardustxr.Tracked")]
pub trait TrackedSpatial {
	#[zbus(property)]
	fn is_tracked(&self) -> zbus::Result<bool>;
}
