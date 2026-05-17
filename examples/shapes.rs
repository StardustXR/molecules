use glam::{Mat4, Vec3, vec3};
use stardust_xr_fusion::{
	client::Client,
	drawable::{Lines, LinesAspect},
	fields::{CylinderShape, Shape, TorusShape},
	root::{RootAspect, RootEvent},
	spatial::{Spatial, Transform},
	values::color::rgba_linear,
};
use stardust_xr_molecules::lines::{LineExt, shape};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let mut client = Client::auto_connect(&[]).await.unwrap();

	let root = Spatial::create(client.get_root(), Transform::identity()).unwrap();

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
			.map(|l| {
				l.transform(Mat4::from_translation(vec3(x_offset, 0.0, 0.0)))
					.color(rgba_linear!(0.0, 1.0, 0.25, 0.5))
			})
			.collect::<Vec<_>>();
		x_offset += 0.25;
		l
	})
	.map(|l| l.thickness(0.005))
	.collect::<Vec<_>>();

	let lines = Lines::create(
		&root,
		Transform::from_translation(Vec3::new(x_offset, 0.0, 0.0)),
		&shapes,
	)
	.unwrap();

	client
		.sync_event_loop(|client, _flow| {
			while let Some(root_event) = client.get_root().recv_root_event() {
				match root_event {
					RootEvent::Ping { response } => response.send_ok(()),
					RootEvent::Frame { info } => {
						let mut shapes = shapes.clone();
						for shape in &mut shapes {
							*shape = shape.clone().trace(info.elapsed).shimmer(
								&[[(info.elapsed * 0.5).sin(), 0.0, 0.0]],
								0.25,
								0.0,
								rgba_linear!(2.0, 2.0, 2.0, 1.0),
								0.5,
							);
						}
						let _ = lines.set_lines(&shapes);
					}
					_ => (),
				}
			}
		})
		.await
		.unwrap()
}
