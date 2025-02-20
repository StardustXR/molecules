#[zbus::proxy(interface = "org.stardustxr.Tracked")]
trait TrackedSpatial {
	#[zbus(property)]
	fn is_tracked(&self) -> zbus::Result<bool>;
}
