use glam::{UVec2, Vec2};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::{pixel::rgba::Rgba, Image}, pass::SubPass, render_graph::ANY_IMAGE, Pass};

#[derive(ParsePass, FromParsedValue)]
pub struct Sobel;

impl Pass for Sobel {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        let gx_image = source.convolve(&[
            -0.25, 0.0, 0.25,
            -0.5, 0.0, 0.5,
            -0.25, 0.0, 0.25,
        ], UVec2::splat(3));

        let gy_image = source.convolve(&[
            -0.25, -0.5, -0.25,
            0.0, 0.0, 0.0,
            0.25, 0.5, 0.25,
        ], UVec2::splat(3));

        target.for_each_with_positions(|pixel, pos| {
            let gx = gx_image.load(pos);
            let gy = gy_image.load(pos);

            let mut g = Vec2::new(gx.r, gy.r);
            let magnitude = g.length();

            if magnitude > 0.0 {
                g /= magnitude;
            } else {
                g = Vec2::ZERO;
            }

            pixel.r = g.x;
            pixel.g = g.y;
            pixel.b = magnitude;
            pixel.a = 1.0;
        })
    }
}

impl SubPass for Sobel {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, _dependencies: &[&Image<4, f32, Rgba<f32>>]) {
        let gx_image = target.convolve(&[
            -0.25, 0.0, 0.25,
            -0.5, 0.0, 0.5,
            -0.25, 0.0, 0.25,
        ], UVec2::splat(3));

        let gy_image = target.convolve(&[
            -0.25, -0.5, -0.25,
            0.0, 0.0, 0.0,
            0.25, 0.5, 0.25,
        ], UVec2::splat(3));

        target.for_each_with_positions(|pixel, pos| {
            let gx = gx_image.load(pos).rgb();
            let gy = gy_image.load(pos).rgb();

            let r = gx.dot(gx);
            let g = gy.dot(gy);
            let b = gx.dot(gy);

            pixel.r = r;
            pixel.g = g;
            pixel.b = b;
            pixel.a = 1.0;
        });
    }
}
