use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

/// A pass that performs a box blur on the `target` image.
pub struct BoxBlur<'a, const KERNEL_SIZE: usize> {
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
}

impl<'a, const KERNEL_SIZE: usize> BoxBlur<'a, KERNEL_SIZE> {
    pub fn new(
        name: &'a str,
        target: &'a str,
        dependencies: Vec<&'a str>,
    ) -> Self {
        Self { name, target, dependencies, source: None }
    }

    pub fn with_source(mut self, source: &'a str) -> Self {
        self.source = Some(source);
        self
    }
}

impl<'a, const KERNEL_SIZE: usize> Pass<'a> for BoxBlur<'a, KERNEL_SIZE> {
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
        let w = 1.0 / (KERNEL_SIZE * KERNEL_SIZE) as f32;

        if self.source.is_some() {
            let source = aux_images[0];
            *target = source.convolve([[w; KERNEL_SIZE]; KERNEL_SIZE]);
        } else {
            *target = target.convolve([[w; KERNEL_SIZE]; KERNEL_SIZE]);
        }
    }
}

