use std::{
	f32::consts::{PI, TAU},
	ops::Mul,
};

use color::Rgba;
use mint::Vector3;
use stardust_xr_fusion::drawable::{self, LinePoint};

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
