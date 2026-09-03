use glam::Vec3;
use gluon::{Interface, RefExt};
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	client::Client,
	drawable::{Line, Lines, LinesExt},
	fields::{Field, FieldExt, FieldRef, FieldSample, Shape},
	query::{InterfaceDependency, QueriedInterface, QueryableId},
	spatial::{Spatial, SpatialExt, SpatialRef, Transform},
	spatial_query::{ZoneQuery, ZoneQueryHandler, ZoneQueryHandlerHandler},
	types::{Color, Vec3F, rgba_linear},
};
use stardust_xr_molecules::{
	lines::{LineExt, circle},
	transformable::protocol::Translatable,
};
use std::{f32::consts::TAU, time::Instant};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

const RADIUS: f32 = 1.0;
const LENGTH: f32 = 2.0;

/// rough peak horizontal displacement, the process is gaussian so this is about 3 rms
const AMPLITUDE: f32 = 0.05;
const VERTICAL: f32 = 0.66;
const DURATION: f32 = 1.0;
const ATTACK: f32 = 0.15;
const DECAY: f32 = 5.0;

/// soil column resonance the bedrock noise gets filtered through, firm ground
const SOIL_FREQUENCY: f32 = 2.5;
const SOIL_DAMPING: f32 = 0.6;

const BAND: (f32, f32) = (0.5, 12.0);
const COMPONENTS: usize = 16;

/// wildly slower than real rock, real waves would cross this zone in under a microsecond
/// and the sweep is the whole point of looking at it
const SPEED: f32 = 3.0;
const COHERENCE: f32 = 0.8;
const TRANSVERSE: f32 = 0.6;
const QUAKE_SEED: u32 = 0x1eaf_c0de;

const RING_INTERVAL: f32 = 0.1;
const RING_SEGMENTS: usize = 64;
const RING_THICKNESS: f32 = 0.012;

const SETTLE_FRAMES: usize = 3;

struct Movable {
	translatable: Translatable,
	base: Vec3,
	seed: u32,
}

/// fast attack and a long tail, the asymmetry is what reads as ground shaking
/// instead of something ringing
fn envelope(t: f32) -> f32 {
	(t / ATTACK).min(1.0) * (-DECAY * (t - ATTACK).max(0.0)).exp()
}

/// Kanai-Tajimi: bedrock white noise through the soil column as a damped resonator,
/// heavily damped so it's a whole band of energy rather than a tone
fn kanai_tajimi(f: f32) -> f32 {
	let r = f / SOIL_FREQUENCY;
	let damped = 2.0 * SOIL_DAMPING * r;
	(1.0 + damped * damped) / ((1.0 - r * r).powi(2) + damped * damped)
}

/// one axis of unit-rms ground motion by spectral representation, a closed form in `t`
/// so any frame time samples the same wave and the restore stays exact
fn ground_motion(t: f32, seed: u32) -> f32 {
	let df = (BAND.1 - BAND.0) / COMPONENTS as f32;
	let (sum, power) = (0..COMPONENTS).fold((0.0, 0.0), |(sum, power), k| {
		let f = BAND.0 + (k as f32 + 0.5) * df;
		let s = kanai_tajimi(f) * df;
		let component = (2.0 * s).sqrt() * (TAU * f * t + phase(seed, k)).cos();
		(sum + component, power + s)
	});
	sum / power.sqrt()
}

/// a ring per wavefront, riding out at the same speed the motion does so you can watch
/// each object start moving as one crosses it
fn rings(t: f32) -> Vec<Line> {
	let mut emit = 0.0;
	let mut rings = Vec::new();
	while emit < DURATION {
		let radius = (t - emit) * SPEED;
		emit += RING_INTERVAL;
		if radius <= 0.0 || radius >= RADIUS {
			continue;
		}
		let p = radius / RADIUS;
		let strength = envelope((emit - RING_INTERVAL).max(ATTACK));
		rings.push(
			circle(RING_SEGMENTS, 0.0, radius)
				.color(ring_color(p, strength))
				.thickness(RING_THICKNESS * (1.0 - p) + 0.002),
		);
	}
	rings
}

/// white at the front, cooling through orange to red as it spreads out and thins
fn ring_color(p: f32, strength: f32) -> Color {
	let (g, b) = if p < 0.5 {
		let k = p * 2.0;
		(1.0 - 0.5 * k, 1.0 - k)
	} else {
		(0.5 * (1.0 - (p - 0.5) * 2.0), 0.0)
	};
	rgba_linear!(1.0, g, b, (1.0 - p).powi(2) * strength)
}

fn phase(seed: u32, k: usize) -> f32 {
	let mut h = seed.wrapping_mul(0x9e37_79b9) ^ (k as u32).wrapping_mul(0x85eb_ca6b);
	h ^= h >> 15;
	h = h.wrapping_mul(0x2545_f491);
	h ^= h >> 13;
	(h as f32 / u32::MAX as f32) * TAU
}
impl Movable {
	fn distance(&self) -> f32 {
		self.base.with_y(0.0).length()
	}

	fn falloff(&self) -> f32 {
		1.0 - (self.distance() / RADIUS).clamp(0.0, 1.0)
	}

	fn arrival(&self) -> f32 {
		self.distance() / SPEED
	}

	fn shake(&self, root: &SpatialRef, t: f32) {
		let td = t - self.arrival();
		if td <= 0.0 {
			return;
		}
		let radial = self.base.with_y(0.0).normalize_or(Vec3::X);
		let transverse = Vec3::Y.cross(radial);
		let a = AMPLITUDE / 3.0 * self.falloff() * envelope(td);
		let offset = radial * (a * self.motion(td, 0))
			+ transverse * (a * TRANSVERSE * self.motion(td, 1))
			+ Vec3::Y * (a * VERTICAL * self.motion(td, 2));
		self.set(root, self.base + offset);
	}

	/// one realization shared by the whole quake with a per object one mixed in, so close
	/// things move together and far ones drift out of step the way real ground does
	fn motion(&self, td: f32, axis: u32) -> f32 {
		let incoherence = 1.0 - (-(self.distance() / COHERENCE).powi(2)).exp();
		let shared = ground_motion(td, QUAKE_SEED ^ axis);
		let local = ground_motion(td, self.seed ^ axis);
		(1.0 - incoherence * incoherence).sqrt() * shared + incoherence * local
	}

	fn set(&self, root: &SpatialRef, to: Vec3) {
		let _ = self
			.translatable
			.set_relative_translation(root.clone(), to.into());
	}
}

#[derive(gluon::Handler)]
struct Epicenter(Mutex<FxHashMap<QueryableId, Movable>>);
impl ZoneQueryHandlerHandler for Epicenter {
	async fn entered(
		&self,
		_ctx: gluon::Context,
		obj: QueryableId,
		_field: FieldRef,
		_spatial: SpatialRef,
		interfaces: Vec<QueriedInterface>,
		relative_position: Vec3F,
		_spatial_info: FieldSample,
	) {
		let Some(interface) = interfaces
			.into_iter()
			.find(|i| i.interface_id == Translatable::ID)
		else {
			return;
		};
		// a queryable registers one spatial and the zone reports its origin, so this is
		// only the movable's real resting place if that's the spatial its Translatable moves
		let movable = Movable {
			translatable: Translatable::from_ref(interface.interface),
			base: relative_position.into(),
			seed: obj.id as u32,
		};
		self.0.lock().await.insert(obj, movable);
	}

	async fn interfaces_changed(
		&self,
		_ctx: gluon::Context,
		_obj: QueryableId,
		_interfaces: Vec<QueriedInterface>,
	) {
	}

	async fn moved(
		&self,
		_ctx: gluon::Context,
		_obj: QueryableId,
		_relative_position: Vec3F,
		_spatial_info: FieldSample,
	) {
	}

	async fn left(&self, _ctx: gluon::Context, _obj: QueryableId) {}
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::connect(&[]).await.unwrap();
	let (root_spatial, root_ref) = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();

	let (_field, field_ref) = Field::new(
		&client,
		&root_spatial,
		Shape::Cylinder {
			length: LENGTH,
			radius: RADIUS,
		},
	)
	.await
	.unwrap();

	let (epicenter, epicenter_ref) =
		ZoneQueryHandler::new_node(Epicenter(Mutex::default())).unwrap();
	let query_handle = client
		.spatial_query_interface()
		.zone_query(ZoneQuery {
			handler: epicenter_ref.into_proxy(),
			interfaces: vec![InterfaceDependency {
				id: Translatable::ID.to_string(),
				optional: false,
			}],
			zone_field: field_ref,
			margin: 0.0,
		})
		.await
		.unwrap()
		.unwrap();

	let ring_lines = Lines::new(&client, &root_spatial, vec![]).await.unwrap();

	let mut frame_receiver = client.frame_receiver();
	for _ in 0..SETTLE_FRAMES {
		frame_receiver.recv().await.unwrap();
	}

	let mut movables: Vec<Movable> = std::mem::take(&mut *epicenter.0.lock().await)
		.into_values()
		.collect();
	movables.sort_by(|a, b| a.distance().total_cmp(&b.distance()));
	// the handler node is the query's lifetime, and the list is locked in now
	drop(query_handle);
	drop(epicenter);

	info!("shaking {} movables", movables.len());
	for movable in &movables {
		let p = movable.base;
		info!(
			"  [{:.3}, {:.3}, {:.3}] {:.3}m out, {:.0}% amplitude, arrives at {:.0}ms",
			p.x,
			p.y,
			p.z,
			movable.distance(),
			movable.falloff() * 100.0,
			movable.arrival() * 1000.0
		);
	}

	let start = Instant::now();
	loop {
		frame_receiver.recv().await.unwrap();

		let t = start.elapsed().as_secs_f32();
		if t >= DURATION + RADIUS / SPEED {
			break;
		}
		let _ = ring_lines.set_lines(rings(t));
		for movable in &movables {
			movable.shake(&root_ref, t);
		}
	}

	let _ = ring_lines.set_lines(vec![]);
	for movable in &movables {
		movable.set(&root_ref, movable.base);
	}
}
