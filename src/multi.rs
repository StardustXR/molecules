use std::future::Future;

use tokio::task::JoinSet;

pub fn multi_call<
	I,
	O: Send + 'static,
	E: Send + 'static,
	F: Future<Output = Result<O, E>> + Send + 'static,
>(
	inputs: impl Iterator<Item = I>,
	mut method: impl FnMut(I) -> F,
) -> impl Future<Output = Vec<Result<O, E>>> {
	let mut join_set = JoinSet::new();
	for input in inputs {
		join_set.spawn(method(input));
	}
	async move {
		let mut results = Vec::new();
		while let Some(result) = join_set.join_next().await {
			results.push(result.unwrap());
		}
		results
	}
}
