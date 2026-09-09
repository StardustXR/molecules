//! A tip input method orbiting the client's root, pressing select every other second.
//!
//! Run it alongside `cargo run --example button` or `--example grabbable` to drive them
//! with no real hardware.

use stardust_xr_fusion::{
	client::Client,
	drawable::{Line, Lines, LinesExt},
	fields::FieldSample,
	query::QueryableId,
	spatial::{PartialTransform, Spatial, SpatialExt, Transform},
	spatial_query::Point,
	suis::{DatamapData, InputDataType, InputHandler as InputHandlerProxy, Tip},
	types::{Color, Posef, Timestamp, rgba_linear},
};
use stardust_xr_molecules::{
	input_method::{CachedHandler, DatamapBuilder, InputMethod, InputMethodHelper},
	lines,
};
use std::{
	collections::{HashMap, HashSet},
	sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

const RADIUS: f32 = 0.05;
const PERIOD: f32 = 4.0;

const IDLE: Color = rgba_linear!(0.0, 0.2, 1.0, 1.0);
const CAPTURED: Color = rgba_linear!(0.0, 1.0, 0.75, 1.0);

fn tip_lines(color: Color) -> Vec<Line> {
	vec![lines::tip(0.05, 0.004, color)]
}

struct CirclingTip {
	select: AtomicBool,
}

impl InputMethodHelper for CirclingTip {
	type QueryValue = FieldSample;

	async fn order_handlers_and_captures(
		&self,
		handlers: &HashMap<QueryableId, CachedHandler<FieldSample>>,
		capture_requests: &HashSet<InputHandlerProxy>,
	) -> (Vec<InputHandlerProxy>, Option<InputHandlerProxy>) {
		let mut ordered: Vec<(f32, InputHandlerProxy)> = handlers
			.values()
			.filter(|e| e.spatial.is_some())
			.map(|e| (e.value.distance, e.handler.clone()))
			.collect();
		ordered.sort_by(|(a, _), (b, _)| a.total_cmp(b));

		// a capture is exclusive: everyone else gets input_left until it releases
		match ordered
			.iter()
			.find(|(_, handler)| capture_requests.contains(handler))
		{
			Some((_, handler)) => (vec![handler.clone()], Some(handler.clone())),
			None => (ordered.into_iter().map(|(_, h)| h).collect(), None),
		}
	}

	async fn input_data(&self, _time: Timestamp) -> Option<InputDataType> {
		Some(InputDataType::Tip {
			data: Tip {
				pose: Posef::default(),
				chirality: None,
				grip_pose: None,
				grip_surface_pose: None,
				simulated_hand: None,
			},
		})
	}

	async fn datamap(&self) -> HashMap<String, DatamapData> {
		let pressed = self.select.load(Ordering::Relaxed) as u8 as f32;
		DatamapBuilder::default()
			.f32("select", pressed)
			.f32("grab", pressed)
			.build()
	}
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt::init();
	let (client, root) = Client::connect(&[]).await.unwrap();

	let (spatial, spatial_ref) = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();

	let tip_lines_obj = Lines::new(&client, &spatial, tip_lines(IDLE))
		.await
		.unwrap();

	let tip = CirclingTip {
		select: AtomicBool::new(false),
	};

	let (method, _proxy, query) = InputMethod::new_points(
		&client,
		tip,
		spatial_ref,
		vec![Point {
			point: [0.0; 3].into(),
			margin: 0.5,
		}],
	)
	.await
	.unwrap();

	let mut frames = client.frame_receiver();
	let mut elapsed = 0.0;
	let mut captured = false;
	loop {
		let info = match frames.recv().await {
			Ok(info) => info,
			Err(RecvError::Lagged(n)) => {
				warn!("lost {n} frame events");
				continue;
			}
			Err(RecvError::Closed) => break,
		};
		elapsed += info.delta;

		let angle = elapsed / PERIOD * std::f32::consts::TAU;
		let position = [angle.cos() * RADIUS, 0.0, angle.sin() * RADIUS];
		let _ = spatial.set_local_transform(PartialTransform {
			translation: Some(position.into()),
			rotation: None,
			scale: None,
		});
		let _ = query.update(vec![Point {
			point: position.into(),
			margin: 0.5,
		}]);
		method
			.select
			.store(elapsed as u32 % 2 == 1, Ordering::Relaxed);

		method.send(info.predicted_display_time).await;

		let now_captured = method.active_capture().await.is_some();
		if now_captured != captured {
			captured = now_captured;
			let _ = tip_lines_obj.set_lines(tip_lines(if captured { CAPTURED } else { IDLE }));
		}
	}
}
