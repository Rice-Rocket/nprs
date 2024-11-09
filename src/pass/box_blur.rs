use crate::image::{pixel::rgba::Rgba, Image};

use super::{Pass, SpecializedPass};

/// A pass that performs a box blur on the `target` image.
pub struct BoxBlur<'a, const KERNEL_SIZE: usize> {
    /// The name of this pass.
    name: &'a str,

    /// The image the blur.
    target: &'a str,

    /// The passes to run before this pass.
    dependencies: &'a [&'a str],
}

impl<'a, const KERNEL_SIZE: usize> BoxBlur<'a, KERNEL_SIZE> {
    pub fn new(
        name: &'a str,
        target: &'a str,
        dependencies: &'a [&'a str],
    ) -> Self {
        Self { name, target, dependencies }
    }
}

impl<'a, const KERNEL_SIZE: usize> Pass<'a> for BoxBlur<'a, KERNEL_SIZE> {
    fn name(&self) -> &'a str {
        self.name
    }

    fn dependencies(&self) -> &[&'a str] {
        self.dependencies
    }

    fn target(&self) -> &'a str {
        self.target
    }

    fn auxiliary_images(&self) -> &[&'a str] {
        &[]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let w = 1.0 / (KERNEL_SIZE * KERNEL_SIZE) as f32;

        *target = target.convolve([[w; KERNEL_SIZE]; KERNEL_SIZE]);
    }
}

pub struct SobelPreBlur<'a, const KERNEL_SIZE: usize>(BoxBlur<'a, KERNEL_SIZE>);

impl<const KERNEL_SIZE: usize> SobelPreBlur<'_, KERNEL_SIZE> {
    const NAME: &'static str = "sobel_pre_blur";

    pub fn new() -> Self {
        Self(BoxBlur::<KERNEL_SIZE>::new(Self::NAME, "tangent_flow_map", &["luminance"]))
    }
}

impl<'a, const KERNEL_SIZE: usize> SpecializedPass<'a> for SobelPreBlur<'a, KERNEL_SIZE> {
    type Parent = BoxBlur<'a, KERNEL_SIZE>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}
