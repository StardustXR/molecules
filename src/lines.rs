use color::{Color, Rgba, ToRgba};
use glam::Vec3;
use mint::Vector3;
use stardust_xr_fusion::drawable::LinePoint;
use std::f32::consts::TAU;

pub fn square(width: f32, height: f32, thickness: f32, color: Rgba<u8>) -> Vec<LinePoint> {
	let half_width = width * 0.5;
	let half_height = height * 0.5;

	vec![
		LinePoint {
			point: Vector3 {
				x: half_width,
				y: half_height,
				z: 0.0,
			},
			thickness,
			color,
		},
		LinePoint {
			point: Vector3 {
				x: -half_width,
				y: half_height,
				z: 0.0,
			},
			thickness,
			color,
		},
		LinePoint {
			point: Vector3 {
				x: -half_width,
				y: -half_height,
				z: 0.0,
			},
			thickness,
			color,
		},
		LinePoint {
			point: Vector3 {
				x: half_width,
				y: -half_height,
				z: 0.0,
			},
			thickness,
			color,
		},
	]
}

pub fn circle(segments: usize, radius: f32, thickness: f32, color: Rgba<u8>) -> Vec<LinePoint> {
	arc(segments, TAU, radius, thickness, color)
}

pub fn arc(
	segments: usize,
	angle: f32,
	radius: f32,
	thickness: f32,
	color: Rgba<u8>,
) -> Vec<LinePoint> {
	(0..segments)
		.map(|s| (s as f32) / (segments as f32) * angle)
		.map(|angle| {
			let (x, y) = angle.sin_cos();
			LinePoint {
				point: Vector3 {
					x: x * radius,
					y: y * radius,
					z: 0.0,
				},
				thickness,
				color,
			}
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
	let first_point = *points.first().unwrap();
	if cyclic {
		points.push(first_point);
	}
	let mut segment_start_t = 0.0;
	let mut segment_start_point = first_point;
	let mut segment_end_t = 0.0;
	let mut segment_end_point = first_point;

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
				segment_start_point = *previous_point;
				segment_end_point = *point;
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
		.to_rgba::<f32>()
		.mix(segment_end_point.color.to_rgba::<f32>(), segment_t)
		.to_rgba::<u8>();
	last.thickness = (segment_start_point.thickness * segment_t)
		+ (segment_end_point.thickness * (1.0 - segment_t));
	last.point = Vector3::from([
		(segment_start_point.point.x * segment_t) + (segment_end_point.point.x * (1.0 - segment_t)),
		(segment_start_point.point.y * segment_t) + (segment_end_point.point.y * (1.0 - segment_t)),
		(segment_start_point.point.z * segment_t) + (segment_end_point.point.z * (1.0 - segment_t)),
	]);

	points
}
