use tokio::sync::mpsc;

pub enum Event<T> {
	Closure(Box<dyn Fn(T) + Send + Sync + 'static>),
	Mpsc(mpsc::Sender<T>),
	UnboundedMpsc(mpsc::UnboundedSender<T>),
}

impl<T> Event<T> {
	pub async fn send(&self, value: T) {
		match self {
			Event::Closure(f) => {
				f(value);
				Ok(())
			}
			Event::Mpsc(sender) => sender.send(value).await,
			Event::UnboundedMpsc(sender) => sender.send(value),
		};
	}
	pub fn try_send(&self, value: T) -> bool {
		match self {
			Event::Closure(f) => {
				f(value);
				true
			}
			Event::Mpsc(sender) => sender.try_send(value).is_ok(),
			Event::UnboundedMpsc(sender) => sender.send(value).is_ok(),
		}
	}
}

impl<T> From<mpsc::Sender<T>> for Event<T> {
	fn from(value: mpsc::Sender<T>) -> Self {
		Self::Mpsc(value)
	}
}
impl<T> From<mpsc::UnboundedSender<T>> for Event<T> {
	fn from(value: mpsc::UnboundedSender<T>) -> Self {
		Self::UnboundedMpsc(value)
	}
}
impl<T, F: Fn(T) + Send + Sync + 'static> From<F> for Event<T> {
	fn from(value: F) -> Self {
		Self::Closure(Box::new(value))
	}
}
