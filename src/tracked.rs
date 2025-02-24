#[zbus::proxy(interface = "org.stardustxr.Tracked")]
pub trait Tracked {
	#[zbus(property)]
	fn is_tracked(&self) -> zbus::Result<bool>;
}
