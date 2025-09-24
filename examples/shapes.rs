use glam::{vec3, Mat4, Vec3};
use stardust_xr_fusion::{
	client::Client,
	drawable::Lines,
	fields::{CylinderShape, Shape, TorusShape},
	root::RootAspect,
	spatial::{Spatial, Transform},
};
use stardust_xr_molecules::lines::{shape, LineExt};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let mut client = Client::connect().await.unwrap();

	let root = Spatial::create(client.get_root(), Transform::identity(), true).unwrap();

	let mut x_offset = -0.375;

	let shapes = vec![
		Shape::Box([0.1, 0.1, 0.1].into()),
		Shape::Cylinder(CylinderShape {
			length: 0.2,
			radius: 0.1,
		}),
		Shape::Sphere(0.1),
		Shape::Torus(TorusShape {
			radius_a: 0.1,
			radius_b: 0.03,
		}),
	]
	.into_iter()
	.flat_map(|l| {
		let l = shape(l)
			.into_iter()
			.map(|l| l.transform(Mat4::from_translation(vec3(x_offset, 0.0, 0.0))))
			.collect::<Vec<_>>();
		x_offset += 0.25;
		l
	})
	.map(|l| l.thickness(0.005))
	.collect::<Vec<_>>();

	let _lines = Lines::create(
		&root,
		Transform::from_translation(Vec3::new(x_offset, 0.0, 0.0)),
		&shapes,
	)
	.unwrap();

	client
		.sync_event_loop(|client, _flow| {
			while let Some(root_event) = client.get_root().recv_root_event() {
				if let stardust_xr_fusion::root::RootEvent::Ping { response } = root_event {
					response.send_ok(());
				}
			}
		})
		.await
		.unwrap()
}
