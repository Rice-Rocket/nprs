use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

pub struct Sobel {
    
}

impl Sobel {
    const NAME: &'static str = "sobel";
}

impl Pass for Sobel {
    fn name() -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> &[&'static str] {
        &["luminance"]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, dependencies: &[&Image<4, f32, Rgba<f32>>]) {
        let luminance = dependencies[0];

        let gx = luminance.convolve([
            [-0.125, 0.0, 0.125],
            [-0.25, 0.0, 0.25],
            [-0.125, 0.0, 0.125],
        ]);

        let gy = luminance.convolve([
            [-0.125, 0.0, 0.125],
            [-0.25, 0.0, 0.25],
            [-0.125, 0.0, 0.125],
        ]);

        target.map_in_place_with_positions(|pixel, pos| {
            pixel.r = gx.load(pos).r;
            pixel.g = gy.load(pos).r;
            pixel.b = 0.0;
            pixel.a = 1.0;
        });
    }
}
