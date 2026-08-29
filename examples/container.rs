use glam::{Quat, Vec3, vec3};
use gluon::{Interface, RefExt};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	drawable::{Line, Lines, LinesExt},
	fields::{Field, FieldExt, Shape},
	query::{QueryableExt, QueryableObject},
	spatial::{Spatial, SpatialExt, SpatialRef, Transform},
	suis::InputDataType,
	types::{Color, rgba_linear},
};
use stardust_xr_molecules::{
	FrameSensitive, UIElement,
	container::{Containable, Container},
	grabbable::{Grabbable, GrabbableSettings, PointerMode},
	input_action::InputSnapshot,
	lines::{LineExt, shape},
};
use stardust_xr_molecules_protocols::container;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

const SHIMMER_RADIUS: f32 = 0.1;

struct GrabBox {
	grabbable: Grabbable,
	content: Spatial,
	field: Field,
	lines: Lines,
	outline: Vec<Line>,
	idle: Vec<Line>,
}
impl GrabBox {
	async fn new<H: ClientHandler>(
		client: &Client<H>,
		parent: SpatialRef,
		position: Vec3,
		size: [f32; 3],
		color: Color,
	) -> Result<Self> {
		let (content, _) = Spatial::new(client, &parent, Transform::IDENTITY).await?;
		let (field, _) = Field::new(client, &content, Shape::Box { size: size.into() }).await?;

		let mut grabbable = Grabbable::new(
			client,
			parent,
			Transform::from_translation(position),
			field.clone(),
			GrabbableSettings {
				pointer_mode: PointerMode::Move,
				..Default::default()
			},
		)
		.await?;
		grabbable.set_pose(position, Quat::IDENTITY);
		content.set_parent(grabbable.content_parent().spatial_ref().await?)?;

		let outline: Vec<Line> = shape(Shape::Box { size: size.into() })
			.into_iter()
			.map(|l| l.simple_subdivide(16).color(color).thickness(0.002))
			.collect();
		let idle: Vec<Line> = outline
			.iter()
			.cloned()
			.map(|l| {
				l.color(rgba_linear!(
					color.c.r * 0.2,
					color.c.g * 0.2,
					color.c.b * 0.2,
					color.a
				))
			})
			.collect();
		let lines = Lines::new(client, &content, idle.clone()).await?;

		Ok(GrabBox {
			grabbable,
			content,
			field,
			lines,
			outline,
			idle,
		})
	}

	fn update(&mut self) {
		self.grabbable.handle_events();
		let _ = self.lines.set_lines(self.visual());
	}

	fn visual(&self) -> Vec<Line> {
		let grab = self.grabbable.grab_action();
		if grab.actor_acting() {
			return self.outline.clone();
		}

		grab.hovering().current().iter().fold(
			self.idle.clone(),
			|lines: Vec<Line>, snap: &Arc<InputSnapshot>| {
				let strength = grab_strength(snap) / (snap.semantic.order + 1) as f32;
				let points = shimmer_points(snap);
				lines
					.into_iter()
					.map(|l| {
						l.shimmer(
							&points,
							SHIMMER_RADIUS,
							0.0,
							rgba_linear!(2.0 * strength, 2.0 * strength, 2.0 * strength, 1.0),
							1.0 + 4.0 * strength,
						)
					})
					.collect()
			},
		)
	}
}

fn grab_strength(snap: &InputSnapshot) -> f32 {
	match snap.input() {
		InputDataType::Hand { .. } => snap.datamap_f32("pinch_strength"),
		_ => snap.datamap_f32("grab"),
	}
}

fn shimmer_points(snap: &InputSnapshot) -> Vec<[f32; 3]> {
	match snap.input() {
		InputDataType::Pointer { data } => {
			let origin = Vec3::from(data.pose.position);
			let direction = Vec3::from(data.direction());
			vec![(origin + direction * data.deepest_point).into()]
		}
		InputDataType::Hand { data } => vec![
			Vec3::from(data.thumb.tip.pose.position).into(),
			Vec3::from(data.index.tip.pose.position).into(),
		],
		InputDataType::Tip { data } => vec![Vec3::from(data.pose.position).into()],
	}
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let (client, root) = Client::connect(&[]).await.unwrap();
	let (_root_spatial, root_ref) = Spatial::new(&client, &root, Transform::IDENTITY)
		.await
		.unwrap();

	let mut container_box = GrabBox::new(
		&client,
		root_ref.clone(),
		vec3(0.0, 0.0, -0.5),
		[0.3; 3],
		rgba_linear!(0.0, 0.75, 1.0, 1.0),
	)
	.await
	.unwrap();
	let (_container_node, container_ref) = container::Container::new_node(Container).unwrap();
	let container_queryable = QueryableObject::new(
		&client,
		container_box.content.clone(),
		container_box.field.clone(),
	)
	.await
	.unwrap();
	let _container_guard = container_queryable
		.add_interface(&container_ref, container::Container::ID)
		.await
		.unwrap();

	let (containable_root, containable_root_ref) =
		Spatial::new(&client, &root_ref, Transform::IDENTITY)
			.await
			.unwrap();
	let mut containable_box = GrabBox::new(
		&client,
		containable_root_ref,
		vec3(0.0, 0.25, -0.5),
		[0.05; 3],
		rgba_linear!(1.0, 0.5, 0.0, 1.0),
	)
	.await
	.unwrap();
	let _containable = Containable::new(
		&client,
		containable_root,
		root_ref.clone(),
		containable_box
			.grabbable
			.content_parent()
			.spatial_ref()
			.await
			.unwrap(),
		|containers| {
			containers
				.values()
				.filter(|(sample, _)| sample.distance < 0.0)
				.max_by(|(a, _), (b, _)| a.distance.total_cmp(&b.distance))
				.map(|(_, spatial)| spatial.clone())
		},
	)
	.await
	.unwrap();

	let mut frame_receiver = client.frame_receiver();
	loop {
		let info = frame_receiver.recv().await.unwrap();

		container_box.update();
		container_box.grabbable.frame(&info);
		containable_box.update();
		containable_box.grabbable.frame(&info);
	}
}
