use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

/// A pass that merges an auxiliary image into the main image.
pub struct Merge<'a> {
    /// The auxiliary image that will be merged.
    pub target: &'a str,

    /// The passes to merge after.
    pub dependencies: Vec<&'a str>,

    /// Ensures the merged image is completely opaque.
    ensure_opaque: bool,
}

impl<'a> Merge<'a> {
    const NAME: &'static str = "merge";

    pub const fn new(
        target: &'a str,
        dependencies: Vec<&'a str>,
    ) -> Self {
        Self {
            target,
            dependencies,
            ensure_opaque: false,
        }
    }

    /// Ensures the merged image is completely opaque.
    pub fn ensure_opaque(mut self) -> Self {
        self.ensure_opaque = true;
        self
    }
}

impl<'a> Pass<'a> for Merge<'a> {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        self.dependencies.clone()
    }

    fn target(&self) -> &'a str {
        "main"
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        vec![self.target]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let aux = aux_images[0];

        *target = aux.clone();

        if self.ensure_opaque {
            target.for_each(|pixel| {
                pixel.a = 1.0;
            });
        }
    }
}
