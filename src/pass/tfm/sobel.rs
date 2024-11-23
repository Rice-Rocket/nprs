use glam::UVec2;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::SubPass};

pub struct Sobel;

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
