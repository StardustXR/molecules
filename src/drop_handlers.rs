use tokio::task::{AbortHandle, JoinHandle};
pub struct AbortOnDrop(pub AbortHandle);
impl Drop for AbortOnDrop {
	fn drop(&mut self) {
		self.0.abort();
	}
}

impl<T> From<JoinHandle<T>> for AbortOnDrop {
	fn from(value: JoinHandle<T>) -> Self {
		Self(value.abort_handle())
	}
}

impl From<AbortHandle> for AbortOnDrop {
	fn from(value: AbortHandle) -> Self {
		Self(value)
	}
}

pub struct OnDrop<T, F: FnMut(&mut T)>(pub T, pub F);
impl<T, F: FnMut(&mut T)> Drop for OnDrop<T, F> {
	fn drop(&mut self) {
		self.1(&mut self.0)
	}
}
