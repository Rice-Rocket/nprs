use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

pub struct Sobel {
    
}

impl Sobel {
    const NAME: &'static str = "sobel";
}

impl<'a> Pass<'a> for Sobel {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> &[&'a str] {
        &["luminance"]
    }
    
    fn target(&self) -> &'a str {
        "tangent_flow_map"
    }

    fn auxiliary_images(&self) -> &[&'a str] {
        &[]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _dependencies: &[&Image<4, f32, Rgba<f32>>]) {
        let gx = target.convolve([
            [-0.125, 0.0, 0.125],
            [-0.25, 0.0, 0.25],
            [-0.125, 0.0, 0.125],
        ]);

        let gy = target.convolve([
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
