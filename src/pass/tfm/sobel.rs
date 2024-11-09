use glam::UVec2;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::Pass};

pub struct Sobel {
    
}

impl Sobel {
    const NAME: &'static str = "sobel";
}

impl<'a> Pass<'a> for Sobel {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        vec!["sobel_pre_blur"]
    }
    
    fn target(&self) -> &'a str {
        "tangent_flow_map"
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        vec![]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _dependencies: &[&Image<4, f32, Rgba<f32>>]) {
        let gx_image = target.convolve(&[
            -0.125, 0.0, 0.125,
            -0.25, 0.0, 0.25,
            -0.125, 0.0, 0.125,
        ], UVec2::splat(3));

        let gy_image = target.convolve(&[
            -0.125, -0.25, -0.125,
            0.0, 0.0, 0.0,
            0.125, 0.25, 0.125,
        ], UVec2::splat(3));

        target.map_in_place_with_positions(|pixel, pos| {
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
