use super::{box_blur::BoxBlur, SpecializedPass};

pub mod sobel;

pub struct SobelPreBlur<'a>(BoxBlur<'a>);

impl SobelPreBlur<'_> {
    const NAME: &'static str = "sobel_pre_blur";

    pub fn new(kernel_radius: usize) -> Self {
        Self(BoxBlur::new(Self::NAME, "tangent_flow_map", vec![], kernel_radius).with_source("main"))
    }
}

impl<'a> SpecializedPass<'a> for SobelPreBlur<'a> {
    type Parent = BoxBlur<'a>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}
