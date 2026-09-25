//! look at and poke the molecules queryables in the running stardust scene
//!
//! `cargo run --example scan -- --help`

use clap::{Parser, Subcommand, error::ErrorKind};
use gbm::{BufferObjectFlags, Format, Modifier};
use glam::{EulerRot, Quat, Vec3, camera::rh::proj::directx};
use gluon_ipc::{Context, Handler, Interface, Node, Ref, RefExt};
use stardust_xr_fusion::{
	camera::{CameraInterface, View},
	client::{Client, DefaultHandler},
	dmatex::{
		AlphaMode, DmatexFormat, DmatexPlane, DmatexPlanes, DmatexSize, DmatexSubmitRelease,
		DmatexSubmitReleaseHandler,
	},
	fields::{FieldRef, FieldSample},
	query::{InterfaceDependency, QueriedInterface, QueryableId},
	spatial::{BoundingBox, PartialTransform, SpatialRef, Transform},
	spatial_query::{
		Point, PointsQuery, PointsQueryHandle, PointsQueryHandler, PointsQueryHandlerHandler,
	},
	types::{Posef, Vec3F},
};
use stardust_xr_molecules_protocols::{
	container, derezzable, environment, keyboard_handler, legible,
	mouse_handler::{self, ScrollSource},
	transformable as tf,
};
use stardust_xr_protocol::dir::find_ref_file;
use std::{
	collections::BTreeMap,
	f32::consts::{PI, TAU},
	path::{Path, PathBuf},
	process::ExitCode,
	time::Duration,
};
use timeline_syncobj::{render_node::DrmRenderNode, timeline_syncobj::TimelineSyncObj};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	sync::mpsc,
};
use tracing_subscriber::EnvFilter;

/// look at and poke the molecules queryables in the running stardust scene
///
/// positions are relative to this client's root, the same space the listing reports in
#[derive(Parser)]
struct Cli {
	/// print the listing as json
	#[arg(global = true, long)]
	json: bool,
	/// how long to collect queryables before acting, in ms
	#[arg(global = true, long, default_value_t = 500)]
	wait: u64,
	/// how far from the root to look, in meters
	#[arg(global = true, long, default_value_t = 1000.0)]
	radius: f32,
	/// only look for these interfaces, by short name like derezzable or poseable
	///
	/// every queried object sends its refs over as fds, so fewer queries means less fd traffic,
	/// commands already only look for the interface they need
	#[arg(global = true, long = "interface", short, value_name = "NAME")]
	interfaces: Vec<String>,
	/// also get each object's bounding box relative to the root, covering its children too
	#[arg(global = true, long)]
	bounds: bool,
	/// also get each object's transform relative to the root, rotation as xyz euler degrees
	#[arg(global = true, long)]
	transforms: bool,
	/// also read the text of everything legible
	#[arg(global = true, long)]
	text: bool,
	#[command(subcommand)]
	command: Option<Command>,
}

#[derive(Subcommand, Clone)]
enum Command {
	/// list everything in reach (the default)
	List,
	/// stay connected and run commands from stdin, one per line, same syntax minus `scan`
	///
	/// `watch on|off` streams query events, `quit` leaves, every command ends with a
	/// `--- ok` or `--- error: ...` line so whatever's driving this knows it finished
	Shell,
	/// print the text of something legible
	Read { id: u64 },
	/// ask an object to derez itself, for most apps this closes it
	Derez { id: u64 },
	/// set or offset an object's position
	#[command(allow_negative_numbers = true)]
	Translate {
		id: u64,
		x: f32,
		y: f32,
		z: f32,
		#[arg(long)]
		offset: bool,
	},
	/// set or offset an object's rotation, as xyz euler degrees
	#[command(allow_negative_numbers = true)]
	Rotate {
		id: u64,
		x: f32,
		y: f32,
		z: f32,
		#[arg(long)]
		offset: bool,
	},
	/// set or offset an object's scale
	#[command(allow_negative_numbers = true)]
	Scale {
		id: u64,
		x: f32,
		y: f32,
		z: f32,
		#[arg(long)]
		offset: bool,
	},
	/// set or offset an object's whole pose
	#[command(allow_negative_numbers = true)]
	Pose {
		id: u64,
		x: f32,
		y: f32,
		z: f32,
		/// xyz euler degrees
		#[arg(long, num_args = 3, value_names = ["X", "Y", "Z"])]
		rot: Option<Vec<f32>>,
		#[arg(long)]
		offset: bool,
	},
	/// set or offset any mix of position, rotation and scale
	#[command(allow_negative_numbers = true)]
	Transform {
		id: u64,
		#[arg(long, num_args = 3, value_names = ["X", "Y", "Z"])]
		pos: Option<Vec<f32>>,
		/// xyz euler degrees
		#[arg(long, num_args = 3, value_names = ["X", "Y", "Z"])]
		rot: Option<Vec<f32>>,
		#[arg(long, num_args = 3, value_names = ["X", "Y", "Z"])]
		scale: Option<Vec<f32>>,
		#[arg(long)]
		offset: bool,
	},
	/// swing, spin and bob an object around, then put it back exactly where it was
	Animate {
		id: u64,
		#[arg(long, default_value_t = 8.0)]
		seconds: f32,
	},
	/// send raw mouse input
	Mouse {
		id: u64,
		#[command(subcommand)]
		action: MouseAction,
	},
	/// have the server render a camera and save what it saw as a png
	#[command(allow_negative_numbers = true)]
	Photo {
		#[arg(default_value = "photo.png")]
		out: PathBuf,
		/// relative to the root
		#[arg(long, num_args = 3, value_names = ["X", "Y", "Z"])]
		pos: Option<Vec<f32>>,
		/// xyz euler degrees, the camera looks down -Z
		#[arg(long, num_args = 3, value_names = ["X", "Y", "Z"])]
		rot: Option<Vec<f32>>,
		/// vertical, in degrees
		#[arg(long, default_value_t = 90.0)]
		fov: f32,
		#[arg(long, default_value_t = 1280)]
		width: u32,
		#[arg(long, default_value_t = 720)]
		height: u32,
	},
}

#[derive(Subcommand, Clone)]
enum MouseAction {
	/// +y is up, +x is right
	#[command(allow_negative_numbers = true)]
	Motion { dx: f32, dy: f32 },
	/// press or release a button code from input-event-codes.h
	Button {
		code: u32,
		#[arg(action = clap::ArgAction::Set)]
		pressed: bool,
	},
	/// press and release, left click by default
	Click {
		#[arg(default_value_t = 0x110)]
		code: u32,
	},
	/// +y is up, +x is right
	#[command(allow_negative_numbers = true)]
	Scroll {
		dx: f32,
		dy: f32,
		/// wheel clicks instead of smooth scrolling
		#[arg(long)]
		discrete: bool,
	},
}
impl Command {
	fn needs(&self) -> Option<&'static str> {
		Some(match self {
			Command::List | Command::Shell | Command::Photo { .. } => return None,
			Command::Read { .. } => legible::Legible::ID,
			Command::Derez { .. } => derezzable::Derezzable::ID,
			Command::Translate { .. } => tf::Translatable::ID,
			Command::Rotate { .. } => tf::Rotatable::ID,
			Command::Scale { .. } => tf::Scalable::ID,
			Command::Pose { .. } | Command::Animate { .. } => tf::Poseable::ID,
			Command::Transform { .. } => tf::Transformable::ID,
			Command::Mouse { .. } => mouse_handler::MouseHandler::ID,
		})
	}
}

const INTERFACES: [&str; 11] = [
	<container::Container as Interface>::ID,
	<derezzable::Derezzable as Interface>::ID,
	<environment::Environment as Interface>::ID,
	<keyboard_handler::KeyboardHandler as Interface>::ID,
	<legible::Legible as Interface>::ID,
	<mouse_handler::MouseHandler as Interface>::ID,
	<tf::Transformable as Interface>::ID,
	<tf::Translatable as Interface>::ID,
	<tf::Rotatable as Interface>::ID,
	<tf::Scalable as Interface>::ID,
	<tf::Poseable as Interface>::ID,
];

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
	let cli = Cli::parse();
	for name in &cli.interfaces {
		if !INTERFACES
			.iter()
			.any(|i| name.eq_ignore_ascii_case(short(i)))
		{
			let known: Vec<_> = INTERFACES.iter().map(|i| short(i)).collect();
			eprintln!("unknown interface {name}, try one of {}", known.join(", "));
			return ExitCode::FAILURE;
		}
	}
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.with_writer(std::io::stderr)
		.init();
	match run(cli).await {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("{e}");
			ExitCode::FAILURE
		}
	}
}

async fn run(cli: Cli) -> Result<(), String> {
	let (client, root) = Client::connect(&[]).await.map_err(|e| e.to_string())?;
	if let Some(Command::Photo { .. }) = cli.command {
		return execute(&client, &root, &mut BTreeMap::new(), &cli).await;
	}
	let (_queries, events, mut found) = collect(&client, &root, &cli).await?;
	match cli.command {
		Some(Command::Shell) => shell(&client, &root, found, events).await,
		_ => execute(&client, &root, &mut found, &cli).await,
	}
}

async fn execute(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	found: &mut BTreeMap<u64, Found>,
	cli: &Cli,
) -> Result<(), String> {
	match cli.command.clone().unwrap_or(Command::List) {
		Command::Shell => return Err("already in a shell".to_string()),
		Command::List => {
			for f in found.values_mut() {
				f.bounds = None;
				f.transform = None;
				f.text = None;
			}
			let shown = |f: &Found| {
				cli.interfaces.is_empty()
					|| f.refs.keys().any(|id| {
						cli.interfaces
							.iter()
							.any(|i| i.eq_ignore_ascii_case(short(id)))
					})
			};
			let (mut listed, rest): (BTreeMap<_, _>, BTreeMap<_, _>) = std::mem::take(found)
				.into_iter()
				.partition(|(_, f)| shown(f));
			if cli.bounds {
				fetch_bounds(client, root, &mut listed).await;
			}
			if cli.transforms {
				fetch_transforms(client, root, &mut listed).await;
			}
			if cli.text {
				fetch_text(&mut listed).await;
			}
			if cli.json {
				print_json(&listed);
			} else {
				print_table(&listed, cli);
			}
			*found = rest;
			found.append(&mut listed);
		}
		Command::Read { id } => {
			let text = proxy::<legible::Legible>(found, id)?
				.text()
				.await
				.map_err(|e| e.to_string())?;
			println!("{text}");
		}
		Command::Animate { id, seconds } => {
			animate(client, root, found, id, seconds).await?;
			println!("animated {id} for {seconds}s, back where it started");
		}
		Command::Photo {
			out,
			pos,
			rot,
			fov,
			width,
			height,
		} => {
			let transform = Transform {
				translation: pos.map_or(Vec3::ZERO, |p| Vec3::from_slice(&p)).into(),
				rotation: rot.map_or(Quat::IDENTITY, |r| euler(&r)).into(),
				scale: Vec3::ONE.into(),
			};
			photo(client, root, transform, fov, width, height, &out).await?;
			println!("saved {}", out.display());
		}
		command => println!("{}", call(root, found, command)?),
	}
	Ok(())
}

async fn shell(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	mut found: BTreeMap<u64, Found>,
	mut events: mpsc::UnboundedReceiver<Event>,
) -> Result<(), String> {
	let mut lines = BufReader::new(tokio::io::stdin()).lines();
	let mut watching = false;
	loop {
		tokio::select! {
			Some(event) = events.recv() => {
				if watching {
					print_event(&event);
				}
				apply(&mut found, event);
			}
			line = lines.next_line() => {
				let Some(line) = line.map_err(|e| e.to_string())? else {
					break;
				};
				let words: Vec<&str> = line.split_whitespace().collect();
				let result = match words.as_slice() {
					[] => continue,
					[w, ..] if w.starts_with('#') => continue,
					["quit" | "exit"] => break,
					["watch", "on"] => {
						watching = true;
						Ok(())
					}
					["watch", "off"] => {
						watching = false;
						Ok(())
					}
					words => match Cli::try_parse_from(std::iter::once("scan").chain(words.iter().copied())) {
						Ok(cli) => {
							while let Ok(event) = events.try_recv() {
								if watching {
									print_event(&event);
								}
								apply(&mut found, event);
							}
							execute(client, root, &mut found, &cli).await
						}
						Err(e) if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) => {
							print!("{}", e.render());
							Ok(())
						}
						Err(e) => Err(e.render().to_string()),
					},
				};
				match result {
					Ok(()) => println!("--- ok"),
					Err(e) => println!("--- error: {}", e.trim().replace('\n', " ")),
				}
			}
		}
	}
	Ok(())
}

fn print_event(event: &Event) {
	match event {
		Event::Entered(id, _, interfaces, sample) => {
			let names: Vec<_> = interfaces.iter().map(|i| short(&i.interface_id)).collect();
			println!(
				"event entered {} {:.3} m {}",
				id.id,
				sample.distance,
				names.join(",")
			);
		}
		Event::InterfacesChanged(id, interfaces) => {
			let names: Vec<_> = interfaces.iter().map(|i| short(&i.interface_id)).collect();
			println!("event interfaces {} {}", id.id, names.join(","));
		}
		Event::Moved(id, sample) => println!("event moved {} {:.3} m", id.id, sample.distance),
		Event::Left(id, interface) => println!("event left {} {}", id.id, short(interface)),
	}
}

struct Found {
	refs: BTreeMap<String, Ref>,
	spatial: SpatialRef,
	sample: FieldSample,
	bounds: Option<Result<BoundingBox, String>>,
	transform: Option<Result<Transform, String>>,
	text: Option<Result<String, String>>,
}

enum Event {
	Entered(QueryableId, SpatialRef, Vec<QueriedInterface>, FieldSample),
	InterfacesChanged(QueryableId, Vec<QueriedInterface>),
	Moved(QueryableId, FieldSample),
	Left(QueryableId, &'static str),
}

#[derive(Debug, Handler)]
struct Probe {
	interface: &'static str,
	tx: mpsc::UnboundedSender<Event>,
}
impl PointsQueryHandlerHandler for Probe {
	async fn entered(
		&self,
		_ctx: Context,
		obj: QueryableId,
		_field: FieldRef,
		spatial: SpatialRef,
		interfaces: Vec<QueriedInterface>,
		spatial_info: FieldSample,
	) {
		let _ = self
			.tx
			.send(Event::Entered(obj, spatial, interfaces, spatial_info));
	}
	async fn interfaces_changed(
		&self,
		_ctx: Context,
		obj: QueryableId,
		interfaces: Vec<QueriedInterface>,
	) {
		let _ = self.tx.send(Event::InterfacesChanged(obj, interfaces));
	}
	async fn moved(&self, _ctx: Context, obj: QueryableId, spatial_info: FieldSample) {
		let _ = self.tx.send(Event::Moved(obj, spatial_info));
	}
	async fn left(&self, _ctx: Context, obj: QueryableId) {
		let _ = self.tx.send(Event::Left(obj, self.interface));
	}
}

// one query per interface since a query needs a required interface, merged by queryable id
type Queries = Vec<(Node<Probe>, PointsQueryHandle)>;

async fn collect(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	cli: &Cli,
) -> Result<
	(
		Queries,
		mpsc::UnboundedReceiver<Event>,
		BTreeMap<u64, Found>,
	),
	String,
> {
	let needed = cli.command.as_ref().and_then(Command::needs);
	let wanted = |id: &str| match needed {
		Some(needed) => id == needed,
		None if cli.interfaces.is_empty() => true,
		None => cli
			.interfaces
			.iter()
			.any(|i| i.eq_ignore_ascii_case(short(id))),
	};

	let (tx, mut rx) = mpsc::unbounded_channel();
	let mut queries = Vec::new();
	for interface in INTERFACES.into_iter().filter(|i| wanted(i)) {
		let (node, handler) = PointsQueryHandler::new_node(Probe {
			interface,
			tx: tx.clone(),
		})
		.map_err(|e| e.to_string())?;
		let handle = client
			.spatial_query_interface()
			.points_query(PointsQuery {
				handler: handler.into_proxy(),
				interfaces: vec![InterfaceDependency {
					id: interface.to_string(),
					optional: false,
				}],
				reference_spatial: root.clone(),
				points: vec![Point {
					point: [0.0; 3].into(),
					margin: cli.radius,
				}],
			})
			.await
			.map_err(|e| e.to_string())?
			.map_err(|e| e.to_string())?;
		queries.push((node, handle));
	}

	let mut found = BTreeMap::new();
	let deadline = tokio::time::sleep(Duration::from_millis(cli.wait));
	tokio::pin!(deadline);
	loop {
		tokio::select! {
			_ = &mut deadline => break,
			Some(event) = rx.recv() => apply(&mut found, event),
		}
	}
	while let Ok(event) = rx.try_recv() {
		apply(&mut found, event);
	}
	Ok((queries, rx, found))
}

fn apply(found: &mut BTreeMap<u64, Found>, event: Event) {
	let refs = |interfaces: Vec<QueriedInterface>| {
		interfaces
			.into_iter()
			.map(|i| (i.interface_id, i.interface))
	};
	match event {
		Event::Entered(id, spatial, interfaces, sample) => found
			.entry(id.id)
			.or_insert(Found {
				refs: BTreeMap::new(),
				spatial,
				sample,
				bounds: None,
				transform: None,
				text: None,
			})
			.refs
			.extend(refs(interfaces)),
		Event::InterfacesChanged(id, interfaces) => {
			if let Some(f) = found.get_mut(&id.id) {
				f.refs.extend(refs(interfaces));
			}
		}
		Event::Moved(id, sample) => {
			if let Some(f) = found.get_mut(&id.id) {
				f.sample = sample;
			}
		}
		Event::Left(id, interface) => {
			if let Some(f) = found.get_mut(&id.id) {
				f.refs.remove(interface);
				if f.refs.is_empty() {
					found.remove(&id.id);
				}
			}
		}
	}
}

fn proxy<P: RefExt>(found: &BTreeMap<u64, Found>, id: u64) -> Result<P, String> {
	let f = found
		.get(&id)
		.ok_or_else(|| format!("nothing with id {id} in reach"))?;
	let r = f
		.refs
		.get(<P as Interface>::ID)
		.ok_or_else(|| format!("{id} isn't {}", short(<P as Interface>::ID)))?;
	Ok(P::from_ref(r.clone()))
}

fn call(
	root: &SpatialRef,
	found: &BTreeMap<u64, Found>,
	command: Command,
) -> Result<String, String> {
	let root = root.clone();
	let sent = |r: Result<(), gluon_ipc::SendError>| r.map_err(|e| e.to_string());
	match command {
		Command::List
		| Command::Shell
		| Command::Read { .. }
		| Command::Animate { .. }
		| Command::Photo { .. } => unreachable!(),
		Command::Derez { id } => {
			sent(proxy::<derezzable::Derezzable>(found, id)?.derez())?;
			Ok(format!("derezzed {id}"))
		}
		Command::Translate {
			id,
			x,
			y,
			z,
			offset,
		} => {
			let t = proxy::<tf::Translatable>(found, id)?;
			let v = Vec3::new(x, y, z).into();
			sent(if offset {
				t.offset_relative_translation(root, v)
			} else {
				t.set_relative_translation(root, v)
			})?;
			Ok(format!("translated {id}"))
		}
		Command::Rotate {
			id,
			x,
			y,
			z,
			offset,
		} => {
			let t = proxy::<tf::Rotatable>(found, id)?;
			let q = euler(&[x, y, z]).into();
			sent(if offset {
				t.offset_relative_rotation(root, q)
			} else {
				t.set_relative_rotation(root, q)
			})?;
			Ok(format!("rotated {id}"))
		}
		Command::Scale {
			id,
			x,
			y,
			z,
			offset,
		} => {
			let t = proxy::<tf::Scalable>(found, id)?;
			let v = Vec3::new(x, y, z).into();
			sent(if offset {
				t.offset_relative_scale(root, v)
			} else {
				t.set_relative_scale(root, v)
			})?;
			Ok(format!("scaled {id}"))
		}
		Command::Pose {
			id,
			x,
			y,
			z,
			rot,
			offset,
		} => {
			let t = proxy::<tf::Poseable>(found, id)?;
			let pose = Posef {
				position: Vec3::new(x, y, z).into(),
				orientation: rot.map_or(Quat::IDENTITY, |r| euler(&r)).into(),
			};
			sent(if offset {
				t.offset_relative_pse(root, pose)
			} else {
				t.set_relative_pose(root, pose)
			})?;
			Ok(format!("posed {id}"))
		}
		Command::Transform {
			id,
			pos,
			rot,
			scale,
			offset,
		} => {
			let t = proxy::<tf::Transformable>(found, id)?;
			let partial = PartialTransform {
				translation: pos.map(|p| Vec3::from_slice(&p).into()),
				rotation: rot.map(|r| euler(&r).into()),
				scale: scale.map(|s| Vec3::from_slice(&s).into()),
			};
			sent(if offset {
				t.offset_relative_transform(root, partial)
			} else {
				t.set_relative_transform(root, partial)
			})?;
			Ok(format!("transformed {id}"))
		}
		Command::Mouse { id, action } => {
			let m = proxy::<mouse_handler::MouseHandler>(found, id)?;
			match action {
				MouseAction::Motion { dx, dy } => sent(m.motion([dx, dy].into(), None))?,
				MouseAction::Button { code, pressed } => sent(m.button(code, pressed, None))?,
				MouseAction::Click { code } => {
					sent(m.button(code, true, None))?;
					sent(m.button(code, false, None))?;
				}
				MouseAction::Scroll { dx, dy, discrete } if discrete => {
					sent(m.scroll_discrete([dx, dy].into(), ScrollSource::Wheel, None))?
				}
				MouseAction::Scroll { dx, dy, .. } => {
					sent(m.scroll_smooth([dx, dy].into(), ScrollSource::Continuous, None))?
				}
			}
			Ok(format!("sent mouse input to {id}"))
		}
	}
}

async fn animate(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	found: &BTreeMap<u64, Found>,
	id: u64,
	seconds: f32,
) -> Result<(), String> {
	let target = proxy::<tf::Poseable>(found, id)?;
	let start = client
		.spatial_interface()
		.get_relative_transform(root.clone(), found[&id].spatial.clone())
		.await
		.map_err(|e| e.to_string())?
		.map_err(|e| format!("couldn't read its pose: {e:?}"))?;
	let p0 = Vec3::from(start.translation);
	let q0 = Quat::from(start.rotation);
	let seconds = seconds.max(0.1);

	let mut frames = client.frame_receiver();
	let mut t = 0.0;
	loop {
		let Ok(info) = frames.recv().await else {
			break;
		};
		t += info.delta;
		let f = (t / seconds).min(1.0);
		// sin² eases both ends, and every wave finishes a whole number of cycles by f = 1
		let ease = (PI * f).sin().powi(2);
		let w = TAU * f;
		let offset = Vec3::new(
			(2.0 * w).sin(),
			0.4 * (3.0 * w).sin(),
			(2.0 * w).cos() - 1.0,
		) * ease;
		let spin = Quat::from_rotation_y(TAU * f * f * (3.0 - 2.0 * f))
			* Quat::from_rotation_z(0.35 * ease * (4.0 * w).sin());
		let (p, q) = if f >= 1.0 {
			(p0, q0)
		} else {
			(p0 + offset, spin * q0)
		};
		let _ = target.set_relative_pose(
			root.clone(),
			Posef {
				position: p.into(),
				orientation: q.into(),
			},
		);
		if f >= 1.0 {
			break;
		}
	}
	Ok(())
}

#[derive(Debug, Handler)]
struct Release(u64);
impl DmatexSubmitReleaseHandler for Release {
	async fn consume(&self, _ctx: Context) -> u64 {
		self.0
	}
}

async fn photo(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	transform: Transform,
	fov: f32,
	w: u32,
	h: u32,
	out: &Path,
) -> Result<(), String> {
	let cameras = CameraInterface::connect(
		find_ref_file("stardust-camera").ok_or("the server isn't exposing stardust-camera")?,
	)
	.await
	.map_err(|e| e.to_string())?;
	let dmatex = client.dmatex_interface();
	let node_id = dmatex
		.primary_render_node_id()
		.await
		.map_err(|e| e.to_string())?;
	let node = DrmRenderNode::new(node_id).map_err(|e| e.to_string())?;

	let formats: Vec<_> = dmatex
		.enumerate_formats(node_id)
		.await
		.map_err(|e| e.to_string())?
		.ok_or("the server couldn't list its dmatex formats")?
		.into_iter()
		// the server maps rgba8888 onto vulkan's R8G8B8A8, so the bytes come out r g b a
		.filter(|f| f.drm_fourcc == Format::Rgba8888 as u32 && f.supports_rendering)
		.collect();
	let srgb = formats.iter().any(|f| f.supports_srgb);
	// linear keeps the cpu readback a plain copy, anything else gets detiled by the gbm map
	let linear = u64::from(Modifier::Linear);
	let modifiers: Vec<_> = if formats.iter().any(|f| f.drm_modifier == linear) {
		vec![Modifier::Linear]
	} else {
		formats
			.iter()
			.map(|f| Modifier::from(f.drm_modifier))
			.collect()
	};
	if modifiers.is_empty() {
		return Err("the server can't render into rgba8888".to_string());
	}
	let gbm = gbm::Device::new(node.clone()).map_err(|e| format!("couldn't open gbm: {e}"))?;
	let bo = gbm
		// gbm doesn't know rgba8888, but every 32 bit fourcc lays the same bytes out
		.create_buffer_object_with_modifiers2::<()>(
			w,
			h,
			Format::Abgr8888,
			modifiers.into_iter(),
			BufferObjectFlags::RENDERING,
		)
		.map_err(|e| format!("couldn't allocate the photo buffer: {e}"))?;
	let planes = (0..bo.plane_count() as i32)
		.map(|i| DmatexPlane {
			offset: bo.offset(i) as u64,
			row_size: bo.stride_for_plane(i) as u64,
			array_element_size: 0,
			depth_slice_size: 0,
		})
		.collect();
	let timeline = TimelineSyncObj::new(&node).map_err(|e| e.to_string())?;
	let target = dmatex
		.import_dmatex(
			DmatexSize::Size2D {
				size: [w, h].into(),
			},
			DmatexFormat {
				drm_fourcc: Format::Rgba8888 as u32,
				drm_modifier: bo.modifier().into(),
				is_srgb: srgb,
				alpha_mode: AlphaMode::PremultipliedOptical,
				ycbcr_info: None,
			},
			1u32,
			DmatexPlanes::Simple {
				dmabuf_fd: bo.fd().map_err(|e| e.to_string())?,
				planes,
			},
			timeline.export().map_err(|e| e.to_string())?,
		)
		.await
		.map_err(|e| e.to_string())?
		.map_err(|e| format!("the server wouldn't import the photo buffer: {e:?}"))?;

	let spatial = client
		.spatial_interface()
		.create_spatial(root.clone(), transform)
		.await
		.map_err(|e| e.to_string())?
		.map_err(|e| format!("couldn't make the camera's spatial: {e:?}"))?;
	let camera = cameras
		.create_camera(spatial.spatial.clone())
		.await
		.map_err(|e| e.to_string())?
		.map_err(|e| format!("couldn't make a camera: {e:?}"))?;

	// the server drops a draw on the floor if the dmatex hasn't reached bevy yet
	let mut frames = client.frame_receiver();
	for _ in 0..3 {
		let _ = frames.recv().await;
	}
	unsafe { timeline.signal(1) }.map_err(|e| e.to_string())?;
	let release = DmatexSubmitRelease::new_service(Release(2)).map_err(|e| e.to_string())?;
	camera
		.request_draw(
			target,
			1u64,
			release.into_proxy(),
			vec![View {
				projection_matrix: directx::perspective_infinite_reverse(
					fov.to_radians(),
					w as f32 / h as f32,
					0.01,
				)
				.into(),
				camera_relative_transform: Transform {
					translation: Vec3::ZERO.into(),
					rotation: Quat::IDENTITY.into(),
					scale: Vec3::ONE.into(),
				},
			}],
		)
		.map_err(|e| e.to_string())?;
	tokio::time::timeout(
		Duration::from_secs(5),
		timeline.wait_async(2).map_err(|e| e.to_string())?,
	)
	.await
	.map_err(|_| "the server never finished rendering")?;

	let rgb = bo
		.map(0, 0, w, h, |m| {
			let stride = m.stride() as usize;
			(0..h as usize)
				.flat_map(|y| m.buffer()[y * stride..][..w as usize * 4].chunks_exact(4))
				.flat_map(|p| [p[0], p[1], p[2]])
				.collect::<Vec<u8>>()
		})
		.map_err(|e| e.to_string())?;
	let mut png = png::Encoder::new(
		std::io::BufWriter::new(std::fs::File::create(out).map_err(|e| e.to_string())?),
		w,
		h,
	);
	png.set_color(png::ColorType::Rgb);
	png.write_header()
		.and_then(|mut p| p.write_image_data(&rgb))
		.map_err(|e| e.to_string())
}

async fn fetch_text(found: &mut BTreeMap<u64, Found>) {
	for f in found.values_mut() {
		let Some(r) = f.refs.get(<legible::Legible as Interface>::ID) else {
			continue;
		};
		f.text = Some(
			legible::Legible::from_ref(r.clone())
				.text()
				.await
				.map_err(|e| e.to_string()),
		);
	}
}

async fn fetch_transforms(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	found: &mut BTreeMap<u64, Found>,
) {
	for f in found.values_mut() {
		let t = client
			.spatial_interface()
			.get_relative_transform(root.clone(), f.spatial.clone())
			.await;
		f.transform = Some(match t {
			Ok(Ok(t)) => Ok(t),
			Ok(Err(e)) => Err(format!("{e:?}")),
			Err(e) => Err(e.to_string()),
		});
	}
}

// one at a time, every request carries spatial refs as fds
async fn fetch_bounds(
	client: &Client<DefaultHandler>,
	root: &SpatialRef,
	found: &mut BTreeMap<u64, Found>,
) {
	for f in found.values_mut() {
		let b = client
			.spatial_interface()
			.get_relative_bounding_box(root.clone(), f.spatial.clone())
			.await;
		f.bounds = Some(match b {
			Ok(Ok(b)) => Ok(b),
			Ok(Err(e)) => Err(format!("{e:?}")),
			Err(e) => Err(e.to_string()),
		});
	}
}

fn sorted(found: &BTreeMap<u64, Found>) -> Vec<(&u64, &Found)> {
	let mut found: Vec<_> = found.iter().collect();
	found.sort_by(|a, b| a.1.sample.distance.total_cmp(&b.1.sample.distance));
	found
}

fn print_table(found: &BTreeMap<u64, Found>, cli: &Cli) {
	if found.is_empty() {
		println!("no molecules queryables within {} m", cli.radius);
		return;
	}
	let bounds_header = if cli.bounds {
		format!("  {:<28}  {:<28}", "bounds center", "bounds extents")
	} else {
		String::new()
	};
	let transform_header = if cli.transforms {
		format!("  {:<28}  {:<24}", "position", "rotation xyz°")
	} else {
		String::new()
	};
	println!(
		"{:>6}  {:>9}  {:<26}{bounds_header}{transform_header}  interfaces",
		"id", "distance", "closest point"
	);
	for (id, f) in sorted(found) {
		let p = f.sample.closest_point;
		let names: Vec<_> = f.refs.keys().map(|i| short(i)).collect();
		let bounds = match &f.bounds {
			Some(Ok(b)) => format!("  {:<28}  {:<28}", vec3(b.center), vec3(b.extents)),
			Some(Err(e)) => format!("  {:<58}", format!("error: {e}")),
			None if cli.bounds => format!("  {:<58}", "-"),
			None => String::new(),
		};
		let transform = match &f.transform {
			Some(Ok(t)) => format!("  {:<28}  {:<24}", vec3(t.translation), degrees(t)),
			Some(Err(e)) => format!("  {:<54}", format!("error: {e}")),
			None if cli.transforms => format!("  {:<54}", "-"),
			None => String::new(),
		};
		let text = match &f.text {
			Some(Ok(t)) => format!("  {t:?}"),
			Some(Err(e)) => format!("  (couldn't read: {e})"),
			None => String::new(),
		};
		println!(
			"{id:>6}  {:>7.3} m  [{:>6.3}, {:>6.3}, {:>6.3}]{bounds}{transform}  {}{text}",
			f.sample.distance,
			p.x,
			p.y,
			p.z,
			names.join(", ")
		);
	}
}

fn print_json(found: &BTreeMap<u64, Found>) {
	let entries: Vec<String> = sorted(found)
		.into_iter()
		.map(|(id, f)| {
			let p = f.sample.closest_point;
			let interfaces: Vec<String> = f.refs.keys().map(|i| format!("\"{i}\"")).collect();
			let bounds = match &f.bounds {
				Some(Ok(b)) => format!(
					",\"bounds\":{{\"center\":[{},{},{}],\"extents\":[{},{},{}]}}",
					b.center.x, b.center.y, b.center.z, b.extents.x, b.extents.y, b.extents.z
				),
				Some(Err(e)) => format!(",\"bounds\":null,\"bounds_error\":{e:?}"),
				None => String::new(),
			};
			let transform = match &f.transform {
				Some(Ok(t)) => {
					let (q, e) = (t.rotation, euler_of(t));
					format!(
						",\"transform\":{{\"position\":[{},{},{}],\"rotation\":[{},{},{},{}],\"euler_xyz_degrees\":[{},{},{}],\"scale\":[{},{},{}]}}",
						t.translation.x, t.translation.y, t.translation.z, q.v.x, q.v.y, q.v.z, q.s, e.x, e.y, e.z, t.scale.x, t.scale.y, t.scale.z
					)
				}
				Some(Err(e)) => format!(",\"transform\":null,\"transform_error\":{e:?}"),
				None => String::new(),
			};
			let text = match &f.text {
				Some(Ok(t)) => format!(",\"text\":{}", json_string(t)),
				Some(Err(e)) => format!(",\"text\":null,\"text_error\":{}", json_string(e)),
				None => String::new(),
			};
			format!(
				"{{\"id\":{id},\"distance\":{},\"closest_point\":[{},{},{}],\"interfaces\":[{}]{bounds}{transform}{text}}}",
				f.sample.distance,
				p.x,
				p.y,
				p.z,
				interfaces.join(",")
			)
		})
		.collect();
	println!("[{}]", entries.join(","));
}

fn json_string(s: &str) -> String {
	let mut out = String::from("\"");
	for c in s.chars() {
		match c {
			'"' => out.push_str("\\\""),
			'\\' => out.push_str("\\\\"),
			c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
			c => out.push(c),
		}
	}
	out.push('"');
	out
}
fn vec3(v: Vec3F) -> String {
	format!("[{:.3}, {:.3}, {:.3}]", v.x, v.y, v.z)
}
fn euler_of(t: &Transform) -> Vec3 {
	let (x, y, z) = Quat::from(t.rotation).to_euler(EulerRot::XYZ);
	Vec3::new(x, y, z).map(f32::to_degrees)
}
fn degrees(t: &Transform) -> String {
	let e = euler_of(t);
	format!("[{:.1}, {:.1}, {:.1}]", e.x, e.y, e.z)
}
fn short(id: &str) -> &str {
	id.rsplit('.').next().unwrap_or(id)
}
fn euler(degrees: &[f32]) -> Quat {
	let [x, y, z] = [degrees[0], degrees[1], degrees[2]].map(f32::to_radians);
	Quat::from_euler(EulerRot::XYZ, x, y, z)
}
