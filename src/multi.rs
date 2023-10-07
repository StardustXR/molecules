use std::future::Future;

use stardust_xr_fusion::node::NodeError;
use tokio::task::JoinSet;

pub fn multi_node_call<
	I,
	O: Send + 'static,
	F: Future<Output = Result<O, NodeError>> + Send + 'static,
>(
	inputs: impl Iterator<Item = I>,
	mut method: impl FnMut(I) -> Result<F, NodeError>,
) -> impl Future<Output = Vec<Result<O, NodeError>>> {
	let mut join_set = JoinSet::new();
	for input in inputs {
		let future = method(input);
		join_set.spawn(async move { future?.await });
	}
	async move {
		let mut results = Vec::new();
		while let Some(result) = join_set.join_next().await {
			results.push(result.unwrap());
		}
		results
	}
}
