use glam::UVec2;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::Pass};

/// A pass that performs a box blur on the `target` image.
pub struct BoxBlur<'a> {
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

    /// The size of the kernel.
    kernel_size: usize,
}

impl<'a> BoxBlur<'a> {
    pub fn new(
        name: &'a str,
        target: &'a str,
        dependencies: Vec<&'a str>,
        kernel_radius: usize,
    ) -> Self {
        let kernel_size = 2 * kernel_radius + 1;

        Self { name, target, dependencies, source: None, kernel_size }
    }

    pub fn with_source(mut self, source: &'a str) -> Self {
        self.source = Some(source);
        self
    }
}

impl<'a> Pass<'a> for BoxBlur<'a> {
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
        let kernel_area = self.kernel_size * self.kernel_size;
        let w = 1.0 / kernel_area as f32;

        if self.source.is_some() {
            let source = aux_images[0];
            *target = source.convolve(&vec![w; kernel_area], UVec2::splat(self.kernel_size as u32));
        } else {
            *target = target.convolve(&vec![w; kernel_area], UVec2::splat(self.kernel_size as u32));
        }
    }
}

