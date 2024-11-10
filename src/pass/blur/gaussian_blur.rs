use std::f32::consts::PI;

use glam::UVec2;

use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

/// A pass that performs a gaussian blur on the `target` image.
pub struct GaussianBlur<'a> {
    /// The name of this pass.
    name: &'a str,

    /// The image to write the resulting blur to.
    ///
    /// If `source` is not specified, this is also the image used to compute the blur, resulting in
    /// the operation being performed in-place.
    target: &'a str,

    /// The image to blur.
    source: Option<&'a str>,

    /// The passes to run before this pass.
    dependencies: Vec<&'a str>,

    /// The gaussian kernel.
    kernel: Vec<f32>,

    /// The size of the gaussian kernel.
    kernel_size: usize,
}

impl<'a> GaussianBlur<'a> {
    pub fn new(
        name: &'a str,
        target: &'a str,
        dependencies: Vec<&'a str>,
        sigma: f32,
    ) -> Self {
        let kernel_size = 2 * (sigma * 2.45).floor() as usize + 1;

        let mut kernel = Vec::new();

        for x in -(kernel_size as i32 / 2)..=(kernel_size as i32 / 2) {
            for y in -(kernel_size as i32 / 2)..=(kernel_size as i32 / 2) {
                let g = gaussian(sigma, x as f32, y as f32);
                kernel.push(g);
            }
        }
        
        Self {
            name,
            target,
            source: None,
            dependencies,
            kernel,
            kernel_size,
        }
    }

    pub fn with_source(mut self, source: &'a str) -> Self {
        self.source = Some(source);
        self
    }
}

impl<'a> Pass<'a> for GaussianBlur<'a> {
    fn name(&self) -> &'a str {
        self.name
    }

    fn dependencies(&self) -> Vec<&'a str> {
        self.dependencies.clone()
    }

    fn target(&self) -> &'a str {
        self.target
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        if let Some(source) = self.source {
            vec![source]
        } else {
            vec![]
        }
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        if self.source.is_some() {
            let source = aux_images[0];
            *target = source.convolve(&self.kernel, UVec2::splat(self.kernel_size as u32));
        } else {
            *target = target.convolve(&self.kernel, UVec2::splat(self.kernel_size as u32));
        }
    }
}

fn gaussian(sigma: f32, x: f32, y: f32) -> f32 {
    (1.0 / f32::sqrt(2.0 * PI * sigma * sigma)) * f32::exp(-(x * x + y * y) / (2.0 * sigma * sigma))
}