use glam::{vec3, Mat4, Vec3, Vec3A};
use lerp::Lerp;
use stardust_xr_fusion::{
	core::values::{
		color::{color_space::LinearRgb, Rgba},
		RowMatrix4, Vector3,
	},
	drawable::{Line, LinePoint},
	spatial::BoundingBox,
};
use std::f32::consts::{PI, TAU};

pub trait LineExt: Sized {
	fn thickness(self, thickness: f32) -> Self;
	fn color(self, color: Rgba<f32, LinearRgb>) -> Self;
	// fn trace(self, amount: f32) -> Self;
	fn lerp(self, other: &Self, amount: f32) -> Option<Self>;
	fn transform(self, transform: impl Into<RowMatrix4<f32>>) -> Self;
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

	// fn trace(self, t: f32) -> Self {
	// 	let points =
	// 	if t <= 0.0 {
	// 		return self;
	// 	}
	// 	if t >= 1.0 || points.len() < 2 {
	// 		return self;
	// 	}
	// 	let first_point = points.first().unwrap().clone();
	// 	if cyclic {
	// 		points.push(first_point.clone());
	// 	}
	// 	let mut segment_start_t = 0.0;
	// 	let mut segment_start_point = first_point.clone();
	// 	let mut segment_end_t = 0.0;
	// 	let mut segment_end_point = first_point.clone();

	// 	let mut new_length: usize = 0;
	// 	{
	// 		let mut current_t = 0.0;
	// 		let mut previous_point = &first_point;
	// 		for (points_len, point) in points.iter().enumerate() {
	// 			let previous_position: Vec3 = previous_point.point.into();
	// 			let previous_t = current_t;
	// 			current_t += previous_position.distance(point.point.into());
	// 			if current_t > t {
	// 				new_length = points_len;
	// 				segment_start_t = previous_t;
	// 				segment_end_t = current_t;
	// 				segment_start_point = previous_point.clone();
	// 				segment_end_point = point.clone();
	// 				break;
	// 			}
	// 			previous_point = point;
	// 		}
	// 	}

	// 	points.truncate(new_length);
	// 	let last = points.last_mut().unwrap();

	// 	let segment_t = (t - segment_start_t) / (segment_end_t - segment_start_t);
	// 	last.color = segment_start_point
	// 		.color
	// 		.mix(segment_end_point.color, segment_t);
	// 	last.thickness = (segment_start_point.thickness * segment_t)
	// 		+ (segment_end_point.thickness * (1.0 - segment_t));
	// 	last.point = Vector3::from([
	// 		(segment_start_point.point.x * segment_t)
	// 			+ (segment_end_point.point.x * (1.0 - segment_t)),
	// 		(segment_start_point.point.y * segment_t)
	// 			+ (segment_end_point.point.y * (1.0 - segment_t)),
	// 		(segment_start_point.point.z * segment_t)
	// 			+ (segment_end_point.point.z * (1.0 - segment_t)),
	// 	]);

	// 	points
	// }

	fn lerp(self, to: &Self, amount: f32) -> Option<Self> {
		if self.points.len() != to.points.len() {
			return None;
		}
		Some(Line {
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
		})
	}

	fn transform(self, transform: impl Into<RowMatrix4<f32>>) -> Self {
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

pub fn circle(segments: usize, start_angle: f32, radius: f32) -> Line {
	let line = arc(segments, start_angle, start_angle + TAU, radius);
	Line {
		points: line.points,
		cyclic: true,
	}
}

pub fn arc(segments: usize, start_angle: f32, end_angle: f32, radius: f32) -> Line {
	let angle = end_angle - start_angle;
	let points = (0..segments)
		.map(|s| ((s as f32) / (segments as f32) * angle) + start_angle)
		.map(|angle| {
			let (x, y) = angle.sin_cos();
			LinePoint {
				point: Vector3 {
					x: x * radius,
					y: y * radius,
					z: 0.0,
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

pub fn bounding_box(bounding_box: BoundingBox) -> Vec<Line> {
	let center = Vec3::from(bounding_box.center);
	let size_half = Vec3::from(bounding_box.size) / 2.0;

	let lines_points = vec![
		vec![
			(center + vec3(-size_half.x, size_half.y, size_half.z)).into(),
			(center + vec3(-size_half.x, size_half.y, -size_half.z)).into(),
		],
		vec![
			(center + vec3(-size_half.x, size_half.y, size_half.z)).into(),
			(center + vec3(size_half.x, size_half.y, size_half.z)).into(),
		],
		vec![
			(center + vec3(-size_half.x, size_half.y, -size_half.z)).into(),
			(center + vec3(size_half.x, size_half.y, -size_half.z)).into(),
		],
		vec![
			(center + vec3(-size_half.x, -size_half.y, size_half.z)).into(),
			(center + vec3(-size_half.x, -size_half.y, -size_half.z)).into(),
		],
		vec![
			(center + vec3(-size_half.x, -size_half.y, size_half.z)).into(),
			(center + vec3(size_half.x, -size_half.y, size_half.z)).into(),
		],
		vec![
			(center + vec3(-size_half.x, -size_half.y, -size_half.z)).into(),
			(center + vec3(size_half.x, -size_half.y, -size_half.z)).into(),
		],
		vec![
			(center + vec3(size_half.x, size_half.y, size_half.z)).into(),
			(center + vec3(size_half.x, size_half.y, -size_half.z)).into(),
		],
		vec![
			(center + vec3(size_half.x, size_half.y, size_half.z)).into(),
			(center + vec3(size_half.x, -size_half.y, size_half.z)).into(),
		],
		vec![
			(center + vec3(size_half.x, size_half.y, -size_half.z)).into(),
			(center + vec3(size_half.x, -size_half.y, -size_half.z)).into(),
		],
	];

	lines_points
		.into_iter()
		.map(|l| Line {
			points: l
				.into_iter()
				.map(|point| LinePoint {
					point,
					..Default::default()
				})
				.collect(),
			cyclic: false,
		})
		.collect()
}
