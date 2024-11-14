use glam::UVec2;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::{Pass, SubPass}, render_graph::ANY_IMAGE};

/// A pass that performs a box blur on the `target` image.
pub struct BoxBlur {
    /// The size of the kernel.
    kernel_size: usize,
}

impl BoxBlur {
    pub const NAME: &'static str = "boxblur";

    pub fn new(kernel_radius: usize) -> Self {
        let kernel_size = 2 * kernel_radius + 1;

        Self { kernel_size }
    }
}

impl<'a> Pass<'a> for BoxBlur {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let kernel_area = self.kernel_size * self.kernel_size;
        let w = 1.0 / kernel_area as f32;

        let source = aux_images[0];
        *target = source.convolve(&vec![w; kernel_area], UVec2::splat(self.kernel_size as u32));
    }
}

impl SubPass for BoxBlur {
    /// Applies this pass as a subpass, blurring the `target` in-place.
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let kernel_area = self.kernel_size * self.kernel_size;
        let w = 1.0 / kernel_area as f32;

        *target = target.convolve(&vec![w; kernel_area], UVec2::splat(self.kernel_size as u32));
    }
}
