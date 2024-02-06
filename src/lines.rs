use color::{color_space::LinearRgb, Color, Rgba};
use glam::{vec3, Vec3};
use lerp::Lerp;
use mint::Vector3;
use stardust_xr_fusion::{drawable::LinePoint, spatial::BoundingBox};
use std::f32::consts::{PI, TAU};

pub fn rectangle(width: f32, height: f32) -> [Vector3<f32>; 4] {
	let half_width = width * 0.5;
	let half_height = height * 0.5;
	let points = [
		[half_width, half_height],
		[-half_width, half_height],
		[-half_width, -half_height],
		[half_width, -half_height],
	];

	let mut result = [[0.0; 3].into(); 4];
	for (i, point) in points.iter().enumerate() {
		result[i] = Vector3 {
			x: point[0],
			y: point[1],
			z: 0.0,
		};
	}
	result
}

pub fn rounded_rectangle(
	width: f32,
	height: f32,
	corner_radius: f32,
	segments: usize,
) -> Vec<Vector3<f32>> {
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
			let point = Vec3::new(
				center.x + corner_radius * angle.cos(),
				center.y + corner_radius * angle.sin(),
				0.0,
			);
			points.push(point.into());
		}
	}

	points
}

pub fn circle(segments: usize, start_angle: f32, radius: f32) -> Vec<Vector3<f32>> {
	arc(segments, start_angle, start_angle + TAU, radius)
}

pub fn arc(segments: usize, start_angle: f32, end_angle: f32, radius: f32) -> Vec<Vector3<f32>> {
	let angle = end_angle - start_angle;
	(0..segments)
		.map(|s| ((s as f32) / (segments as f32) * angle) + start_angle)
		.map(|angle| {
			let (x, y) = angle.sin_cos();
			Vector3 {
				x: x * radius,
				y: y * radius,
				z: 0.0,
			}
		})
		.collect()
}

pub fn bounding_box(bounding_box: BoundingBox) -> Vec<Vec<Vector3<f32>>> {
	let center = Vec3::from(bounding_box.center);
	let size_half = Vec3::from(bounding_box.size) / 2.0;

	vec![
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
	]
}

pub fn make_line_points(
	vec3s: Vec<Vector3<f32>>,
	thickness: f32,
	color: Rgba<f32, LinearRgb>,
) -> Vec<LinePoint> {
	vec3s
		.into_iter()
		.map(|point| LinePoint {
			point,
			thickness,
			color,
		})
		.collect()
}

pub fn trace(t: f32, mut points: Vec<LinePoint>, cyclic: bool) -> Vec<LinePoint> {
	if t <= 0.0 {
		return vec![];
	}
	if t >= 1.0 || points.len() < 2 {
		return points;
	}
	let first_point = points.first().unwrap().clone();
	if cyclic {
		points.push(first_point.clone());
	}
	let mut segment_start_t = 0.0;
	let mut segment_start_point = first_point.clone();
	let mut segment_end_t = 0.0;
	let mut segment_end_point = first_point.clone();

	let mut new_length: usize = 0;
	{
		let mut current_t = 0.0;
		let mut previous_point = &first_point;
		for (points_len, point) in points.iter().enumerate() {
			let previous_position: Vec3 = previous_point.point.into();
			let previous_t = current_t;
			current_t += previous_position.distance(point.point.into());
			if current_t > t {
				new_length = points_len;
				segment_start_t = previous_t;
				segment_end_t = current_t;
				segment_start_point = previous_point.clone();
				segment_end_point = point.clone();
				break;
			}
			previous_point = point;
		}
	}

	points.truncate(new_length);
	let last = points.last_mut().unwrap();

	let segment_t = (t - segment_start_t) / (segment_end_t - segment_start_t);
	last.color = segment_start_point
		.color
		.mix(segment_end_point.color, segment_t);
	last.thickness = (segment_start_point.thickness * segment_t)
		+ (segment_end_point.thickness * (1.0 - segment_t));
	last.point = Vector3::from([
		(segment_start_point.point.x * segment_t) + (segment_end_point.point.x * (1.0 - segment_t)),
		(segment_start_point.point.y * segment_t) + (segment_end_point.point.y * (1.0 - segment_t)),
		(segment_start_point.point.z * segment_t) + (segment_end_point.point.z * (1.0 - segment_t)),
	]);

	points
}

pub fn lerp(from: &[LinePoint], to: &[LinePoint], amount: f32) -> Option<Vec<LinePoint>> {
	if from.len() != to.len() {
		return None;
	}

	Some(
		from.into_iter()
			.zip(to)
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
	)
}
