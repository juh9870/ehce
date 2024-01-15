use bevy::log::error;
use bevy::prelude::Image;
use bevy_xpbd_2d::math::Vector;
use bevy_xpbd_2d::prelude::Collider;
use contour::ContourBuilder;
use geo::{Simplify, TriangulateEarcut};
use miette::Diagnostic;
use thiserror::Error;

use crate::image_ext::ImageExt;

mod image_ext;

#[derive(Debug, Error, Diagnostic)]
pub enum ColliderComputationError {
    #[error("Provided width and height values don't match bitmap length. {} * {} != {}", .width, .height, .len)]
    BadDimensions { width: u32, height: u32, len: usize },
}

pub fn compute_collider(
    bitmap: &[bool],
    width: u32,
    height: u32,
    threshold: f32,
) -> Result<Collider, ColliderComputationError> {
    let dots = bitmap
        .iter()
        .map(|e| if *e { 1.0 } else { 0.0 })
        .collect::<Vec<f32>>();
    let area = width * height;

    if area != bitmap.len() as u32 {
        return Err(ColliderComputationError::BadDimensions {
            width,
            height,
            len: bitmap.len(),
        });
    }

    let lines = ContourBuilder::new(width, height, true)
        .x_origin(-0.5)
        .y_origin(-0.5)
        .x_step(1.0 / width as f32)
        .y_step(1.0 / width as f32)
        .contours(dots.as_slice(), &[1.0])
        .unwrap_or_else(|_| unreachable!());

    let mut vertices: Vec<Vector> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    for triangulation in lines.into_iter().flat_map(|line| {
        line.into_inner()
            .0
             .0
            .into_iter()
            .map(|poly| poly.simplify(&threshold).earcut_triangles_raw())
        // .skip(10)
        // .take(2)
    }) {
        let last_index = vertices.len();

        indices.extend(triangulation.triangle_indices.chunks_exact(3).map(|c| {
            [
                (c[0] + last_index) as u32,
                (c[1] + last_index) as u32,
                (c[2] + last_index) as u32,
            ]
        }));

        vertices.extend(
            triangulation
                .vertices
                .chunks_exact(2)
                .map(|e| Vector::new(e[0], -e[1])),
        );
    }
    Ok(Collider::trimesh(vertices, indices))
}

pub fn compute_collider_for_texture(image: &Image, optimization_threshold: f32) -> Collider {
    let rows = image.size().y;
    let cols = image.size().x;

    let mut processed: Vec<bool> = Vec::with_capacity((rows * cols) as usize);
    for y in 0..rows {
        for x in 0..rows {
            let color = image.get_color_at(x, y).unwrap_or_else(|err| {
                error!(?err);
                std::process::exit(1)
            });
            processed.push(color.a() > 0.5);
        }
    }

    compute_collider(processed.as_slice(), cols, rows, optimization_threshold).unwrap_or_else(
        |err| {
            error!(?err);
            std::process::exit(1)
        },
    )
}
