use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

/// A pass that merges an auxiliary image into the main image.
pub struct Merge {
    /// The auxiliary image that will be merged.
    pub target: &'static [&'static str],

    /// The passes to merge after.
    pub dependencies: &'static [&'static str],

    /// Ensures the merged image is completely opaque.
    ensure_opaque: bool,
}

impl Merge {
    const NAME: &'static str = "merge";

    pub const fn new(target: &'static [&'static str], dependencies: &'static [&'static str]) -> Merge {
        Merge {
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

impl<'a> Pass<'a> for Merge {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> &[&'a str] {
        self.dependencies
    }

    fn target(&self) -> &'a str {
        "main"
    }

    fn auxiliary_images(&self) -> &[&'a str] {
        self.target
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let aux = aux_images[0];

        *target = aux.clone();

        if self.ensure_opaque {
            target.map_in_place(|pixel| {
                pixel.a = 1.0;
            });
        }
    }
}
