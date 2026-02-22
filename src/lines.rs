use glam::{FloatExt, Mat4, Vec3, Vec3A, vec3};
use lerp::Lerp;
use stardust_xr_fusion::{
	drawable::{Line, LinePoint},
	fields::{CylinderShape, Shape, TorusShape},
	spatial::BoundingBox,
	values::color::rgba_linear,
	values::{
		Mat4 as Matrix4, Vector3,
		color::{Rgba, color_space::LinearRgb},
	},
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

fn gcd(mut a: usize, mut b: usize) -> usize {
	while b != 0 {
		(a, b) = (b, a % b);
	}
	a
}

fn lcm(a: usize, b: usize) -> usize {
	a * b / gcd(a, b)
}

pub trait LineExt: Sized {
	fn thickness(self, thickness: f32) -> Self;
	fn color(self, color: Rgba<f32, LinearRgb>) -> Self;
	fn shimmer<P: Into<Vector3<f32>> + Copy>(
		self,
		points: &[P],
		max_distance: f32,
		min_distance: f32,
		to_color: Rgba<f32, LinearRgb>,
		thickness_multiplier: f32,
	) -> Self;
	fn trace(self, t: f32) -> Self;
	fn simple_subdivide(&self, n: usize) -> Self;
	fn lerp(self, other: &Self, amount: f32) -> Option<Self>;
	fn transform(self, transform: impl Into<Matrix4>) -> Self;
}

impl LineExt for Line {
	fn thickness(self, thickness: f32) -> Self {
		Line {
			points: self
				.points
				.into_iter()
				.map(|p| LinePoint {
					point: p.point,
					thickness,
					color: p.color,
				})
				.collect(),
			cyclic: self.cyclic,
		}
	}
	fn color(self, color: Rgba<f32, LinearRgb>) -> Self {
		Line {
			points: self
				.points
				.into_iter()
				.map(|p| LinePoint {
					point: p.point,
					thickness: p.thickness,
					color,
				})
				.collect(),
			cyclic: self.cyclic,
		}
	}

	fn shimmer<P: Into<Vector3<f32>> + Copy>(
		mut self,
		points: &[P],
		max_distance: f32,
		min_distance: f32,
		to_color: Rgba<f32, LinearRgb>,
		thickness_multiplier: f32,
	) -> Self {
		for point in &mut self.points {
			let Some(shimmer_distance) = points
				.iter()
				.map(|p| Vec3::from(Into::<Vector3<f32>>::into(*p)).distance(point.point.into()))
				.reduce(|a, b| a.min(b))
			else {
				return self;
			};

			let mapped = shimmer_distance
				.remap(max_distance, min_distance, 0.0, 1.0)
				.clamp(0.0, 1.0);

			point.color.lerp_bounded_to(to_color, mapped);
			point.thickness *= mapped.remap(0.0, 1.0, 1.0, thickness_multiplier);
		}
		self
	}

	fn trace(self, t: f32) -> Self {
		// Edge cases
		if self.points.len() < 2 || t >= 1.0 {
			return self;
		}
		if t <= 0.0 {
			return Line {
				points: vec![],
				cyclic: false,
			};
		}

		// Build working points list (close loop if cyclic)
		let mut points = self.points;
		if self.cyclic {
			let first = *points.first().unwrap();
			points.push(first);
		}

		// Calculate total length
		let total_length: f32 = points
			.windows(2)
			.map(|w| Vec3::from(w[0].point).distance(Vec3::from(w[1].point)))
			.sum();

		if total_length == 0.0 {
			return Line {
				points,
				cyclic: false,
			};
		}

		let target_distance = t * total_length;

		// Find the segment containing target_distance
		let mut accumulated = 0.0;
		let mut result_points = Vec::new();

		for window in points.windows(2) {
			let start_point = &window[0];
			let end_point = &window[1];
			let segment_length =
				Vec3::from(start_point.point).distance(Vec3::from(end_point.point));

			if accumulated + segment_length >= target_distance {
				// This segment contains our target
				result_points.push(*start_point);

				let segment_t = if segment_length > 0.0 {
					(target_distance - accumulated) / segment_length
				} else {
					0.0
				};

				// Interpolate the final point
				let start_pos = Vec3::from(start_point.point);
				let end_pos = Vec3::from(end_point.point);

				let interpolated = LinePoint {
					point: start_pos.lerp(end_pos, segment_t).into(),
					thickness: Lerp::lerp(start_point.thickness, end_point.thickness, segment_t),
					color: start_point.color.lerp_bounded(end_point.color, segment_t),
				};
				result_points.push(interpolated);
				break;
			}

			result_points.push(*start_point);
			accumulated += segment_length;
		}

		Line {
			points: result_points,
			cyclic: false,
		}
	}

	fn simple_subdivide(&self, n: usize) -> Self {
		if n == 0 || self.points.len() < 2 {
			return self.clone();
		}
		let mut new_points = Vec::new();
		for window in self.points.windows(2) {
			let start = &window[0];
			let end = &window[1];
			new_points.push(*start);
			for i in 1..=n {
				let t = i as f32 / (n + 1) as f32;
				let start_pos = Vec3::from(start.point);
				let end_pos = Vec3::from(end.point);
				new_points.push(LinePoint {
					point: start_pos.lerp(end_pos, t).into(),
					thickness: Lerp::lerp(start.thickness, end.thickness, t),
					color: start.color.lerp_bounded(end.color, t),
				});
			}
		}
		new_points.push(*self.points.last().unwrap());
		Line {
			points: new_points,
			cyclic: self.cyclic,
		}
	}

	fn lerp(self, to: &Self, amount: f32) -> Option<Self> {
		let len_a = self.points.len();
		let len_b = to.points.len();

		// Fast path: same point counts
		if len_a == len_b {
			return Some(Line {
				points: self
					.points
					.into_iter()
					.zip(to.points.iter())
					.map(|(from, to)| {
						let from_point = Vec3::from(from.point);
						let to_point = Vec3::from(to.point);

						LinePoint {
							point: from_point.lerp_bounded(to_point, amount).into(),
							thickness: from.thickness.lerp_bounded(to.thickness, amount),
							color: from.color.lerp_bounded(to.color, amount),
						}
					})
					.collect(),
				cyclic: if amount > 0.5 { to.cyclic } else { self.cyclic },
			});
		}

		// Need at least 2 points in each line to have segments
		if len_a < 2 || len_b < 2 {
			return None;
		}

		// Different point counts: use LCM of segment counts
		let seg_a = len_a - 1;
		let seg_b = len_b - 1;
		let target = lcm(seg_a, seg_b);

		let subdivided_a = self.simple_subdivide(target / seg_a - 1);
		let subdivided_b = to.simple_subdivide(target / seg_b - 1);

		// Now both have target + 1 points, lerp normally
		subdivided_a.lerp(&subdivided_b, amount)
	}

	fn transform(self, transform: impl Into<Matrix4>) -> Self {
		let transform: Mat4 = transform.into().into();
		Line {
			points: self
				.points
				.into_iter()
				.map(|p| LinePoint {
					point: transform.transform_point3a(Vec3A::from(p.point)).into(),
					thickness: p.thickness,
					color: p.color,
				})
				.collect(),
			cyclic: self.cyclic,
		}
	}
}

pub fn rounded_rectangle(width: f32, height: f32, corner_radius: f32, segments: usize) -> Line {
	let mut points = Vec::new();

	let half_width = width / 2.0;
	let half_height = height / 2.0;

	let angle_step = PI / 2.0 / (segments as f32);

	for i in 0..4 {
		let start_angle = match i {
			0 => 0.0,
			1 => PI * 0.5,
			2 => PI,
			3 => PI * 1.5,
			_ => unreachable!(),
		};

		let center = match i {
			0 => Vec3::new(half_width - corner_radius, half_height - corner_radius, 0.0),
			1 => Vec3::new(
				-half_width + corner_radius,
				half_height - corner_radius,
				0.0,
			),
			2 => Vec3::new(
				-half_width + corner_radius,
				-half_height + corner_radius,
				0.0,
			),
			3 => Vec3::new(
				half_width - corner_radius,
				-half_height + corner_radius,
				0.0,
			),
			_ => unreachable!(),
		};

		for j in 0..=segments {
			let angle = start_angle + (angle_step * j as f32);
			points.push(LinePoint {
				point: [
					center.x + corner_radius * angle.cos(),
					center.y + corner_radius * angle.sin(),
					0.0,
				]
				.into(),
				..Default::default()
			});
		}
	}

	Line {
		points,
		cyclic: true,
	}
}

pub fn shape(shape: Shape) -> Vec<Line> {
	fn y_offset_circle(segments: usize, radius: f32, offset: f32) -> Line {
		let mut line = circle(segments, 0.0, radius);
		line.points.iter_mut().for_each(|p| p.point.y += offset);
		line
	}
	match shape {
		Shape::Box(size) => bounding_box(BoundingBox {
			center: Vec3::ZERO.into(),
			size,
		}),
		Shape::Cylinder(CylinderShape { length, radius }) => {
			let top = y_offset_circle(32, radius, length * 0.5);
			let bottom = y_offset_circle(32, radius, -length * 0.5);

			let connector_1 =
				simple_line([radius, length * 0.5, 0.0], [radius, length * -0.5, 0.0]);
			let connector_2 =
				simple_line([-radius, length * 0.5, 0.0], [-radius, length * -0.5, 0.0]);
			let connector_3 =
				simple_line([0.0, length * 0.5, radius], [0.0, length * -0.5, radius]);
			let connector_4 =
				simple_line([0.0, length * 0.5, -radius], [0.0, length * -0.5, -radius]);

			vec![
				top,
				bottom,
				connector_1,
				connector_2,
				connector_3,
				connector_4,
			]
		}
		Shape::Sphere(radius) => {
			let y = circle(32, 0.0, radius);
			let x = y.clone().transform(Mat4::from_rotation_x(FRAC_PI_2));
			let z = y.clone().transform(Mat4::from_rotation_z(FRAC_PI_2));

			vec![x, y, z]
		}
		Shape::Spline(spline) => {
			vec![spline.to_lines(8)]
		}
		Shape::Torus(TorusShape { radius_a, radius_b }) => {
			let radius_a_outer = circle(32, 0.0, radius_a - radius_b);
			let radius_a_inner = circle(32, 0.0, radius_a + radius_b);
			let radius_a_top = y_offset_circle(32, radius_a, radius_b);
			let radius_a_bottom = y_offset_circle(32, radius_a, -radius_b);

			let radius_b_1 = circle(16, 0.0, radius_b).transform(
				Mat4::from_translation(vec3(radius_a, 0.0, 0.0)) * Mat4::from_rotation_x(FRAC_PI_2),
			);
			let radius_b_2 = circle(16, 0.0, radius_b).transform(
				Mat4::from_translation(vec3(-radius_a, 0.0, 0.0))
					* Mat4::from_rotation_x(FRAC_PI_2),
			);
			let radius_b_3 = circle(16, 0.0, radius_b).transform(
				Mat4::from_translation(vec3(0.0, 0.0, radius_a))
					* Mat4::from_rotation_y(FRAC_PI_2)
					* Mat4::from_rotation_x(FRAC_PI_2),
			);
			let radius_b_4 = circle(16, 0.0, radius_b).transform(
				Mat4::from_translation(vec3(0.0, 0.0, -radius_a))
					* Mat4::from_rotation_y(FRAC_PI_2)
					* Mat4::from_rotation_x(FRAC_PI_2),
			);
			vec![
				radius_a_outer,
				radius_a_inner,
				radius_a_top,
				radius_a_bottom,
				radius_b_1,
				radius_b_2,
				radius_b_3,
				radius_b_4,
			]
		}
	}
}

/// on the XZ plane
pub fn circle(segments: usize, start_angle: f32, radius: f32) -> Line {
	let line = arc(segments, start_angle, start_angle + TAU, radius);
	Line {
		points: line.points,
		cyclic: true,
	}
}

/// on the XZ plane
pub fn arc(segments: usize, start_angle: f32, end_angle: f32, radius: f32) -> Line {
	let angle = end_angle - start_angle;
	let points = (0..segments)
		.map(|s| ((s as f32) / (segments as f32) * angle) + start_angle)
		.map(|angle| {
			let (x, y) = angle.sin_cos();
			LinePoint {
				point: Vector3 {
					x: x * radius,
					y: 0.0,
					z: y * radius,
				},
				..Default::default()
			}
		})
		.collect();
	Line {
		points,
		cyclic: false,
	}
}

pub fn line_from_points(points: Vec<impl Into<Vector3<f32>>>) -> Line {
	Line {
		points: points
			.into_iter()
			.map(|p| LinePoint {
				point: p.into(),
				..Default::default()
			})
			.collect(),
		cyclic: false,
	}
}

pub fn axes(length: f32, thickness: f32) -> Vec<Line> {
	let r = rgba_linear!(1.0, 0.0, 0.0, 1.0);
	let g = rgba_linear!(0.0, 1.0, 0.0, 1.0);
	let b = rgba_linear!(0.0, 0.0, 1.0, 1.0);
	vec![
		line_from_points(vec![Vec3::ZERO, Vec3::X * length])
			.color(r)
			.thickness(thickness),
		line_from_points(vec![Vec3::ZERO, Vec3::Y * length])
			.color(g)
			.thickness(thickness),
		line_from_points(vec![Vec3::ZERO, Vec3::Z * length])
			.color(b)
			.thickness(thickness),
	]
}

fn simple_line(start: impl Into<Vector3<f32>>, end: impl Into<Vector3<f32>>) -> Line {
	Line {
		points: vec![
			LinePoint {
				point: start.into(),
				..Default::default()
			},
			LinePoint {
				point: end.into(),
				..Default::default()
			},
		],
		cyclic: false,
	}
}

pub fn bounding_box(bounding_box: BoundingBox) -> Vec<Line> {
	let center = Vec3::from(bounding_box.center);
	let size_half = Vec3::from(bounding_box.size) / 2.0;

	vec![
		simple_line(
			center + vec3(-size_half.x, size_half.y, size_half.z),
			center + vec3(-size_half.x, size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, size_half.y, size_half.z),
			center + vec3(size_half.x, size_half.y, size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, size_half.y, -size_half.z),
			center + vec3(size_half.x, size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, -size_half.y, size_half.z),
			center + vec3(-size_half.x, -size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, -size_half.y, size_half.z),
			center + vec3(size_half.x, -size_half.y, size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, -size_half.y, -size_half.z),
			center + vec3(size_half.x, -size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(size_half.x, size_half.y, size_half.z),
			center + vec3(size_half.x, size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(size_half.x, size_half.y, size_half.z),
			center + vec3(size_half.x, -size_half.y, size_half.z),
		),
		simple_line(
			center + vec3(size_half.x, size_half.y, -size_half.z),
			center + vec3(size_half.x, -size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, size_half.y, size_half.z),
			center + vec3(-size_half.x, -size_half.y, size_half.z),
		),
		simple_line(
			center + vec3(-size_half.x, size_half.y, -size_half.z),
			center + vec3(-size_half.x, -size_half.y, -size_half.z),
		),
		simple_line(
			center + vec3(size_half.x, -size_half.y, size_half.z),
			center + vec3(size_half.x, -size_half.y, -size_half.z),
		),
	]
}
