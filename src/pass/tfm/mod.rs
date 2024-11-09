use super::{box_blur::BoxBlur, SpecializedPass};

pub mod sobel;

pub struct SobelPreBlur<'a, const KERNEL_SIZE: usize>(BoxBlur<'a, KERNEL_SIZE>);

impl<const KERNEL_SIZE: usize> SobelPreBlur<'_, KERNEL_SIZE> {
    const NAME: &'static str = "sobel_pre_blur";

    pub fn new() -> Self {
        Self(BoxBlur::<KERNEL_SIZE>::new(Self::NAME, "tangent_flow_map", vec![]).with_source("main"))
    }
}

impl<'a, const KERNEL_SIZE: usize> SpecializedPass<'a> for SobelPreBlur<'a, KERNEL_SIZE> {
    type Parent = BoxBlur<'a, KERNEL_SIZE>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}
