use std::f32::consts::PI;

use glam::UVec2;
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::{pixel::rgba::Rgba, Image}, pass::{Pass, SubPass}, render_graph::ANY_IMAGE};

/// A pass that performs a gaussian blur on the `target` image.
#[derive(ParsePass, FromParsedValue)]
#[nprs(from = GaussianBlurBuilder)]
pub struct GaussianBlur {
    /// The gaussian kernel.
    kernel: Vec<f32>,

    /// The size of the gaussian kernel.
    kernel_size: usize,
}

impl GaussianBlur {
    pub fn new(sigma: f32) -> Self {
        let kernel_size = 2 * (sigma * 2.45).floor() as usize + 1;

        let mut kernel = Vec::new();
        let mut kernel_sum = 0.0;

        for x in -(kernel_size as i32 / 2)..=(kernel_size as i32 / 2) {
            for y in -(kernel_size as i32 / 2)..=(kernel_size as i32 / 2) {
                let g = gaussian(sigma, x as f32, y as f32);
                kernel_sum += g;
                kernel.push(g);
            }
        }

        kernel.iter_mut().for_each(|v| *v /= kernel_sum);
        
        Self {
            kernel,
            kernel_size,
        }
    }
}

impl Pass for GaussianBlur {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];
        *target = source.convolve(&self.kernel, UVec2::splat(self.kernel_size as u32));
    }
}

impl SubPass for GaussianBlur {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        *target = target.convolve(&self.kernel, UVec2::splat(self.kernel_size as u32));
    }
}

fn gaussian(sigma: f32, x: f32, y: f32) -> f32 {
    (1.0 / f32::sqrt(2.0 * PI * sigma * sigma)) * f32::exp(-(x * x + y * y) / (2.0 * sigma * sigma))
}

#[derive(FromParsedValue)]
pub struct GaussianBlurBuilder {
    sigma: f32,
}

impl From<GaussianBlurBuilder> for GaussianBlur {
    fn from(builder: GaussianBlurBuilder) -> Self {
        GaussianBlur::new(builder.sigma)
    }
}
