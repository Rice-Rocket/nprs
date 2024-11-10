use super::{blur::{box_blur::BoxBlur, gaussian_blur::GaussianBlur}, SpecializedPass};

pub mod sobel;
pub mod structure_tensor;

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

pub struct SobelPostBlur<'a>(GaussianBlur<'a>);

impl SobelPostBlur<'_> {
    const NAME: &'static str = "sobel_post_blur";

    pub fn new(sigma: f32) -> Self {
        Self(GaussianBlur::new(Self::NAME, "tangent_flow_map", vec!["sobel"], sigma))
    }
}

impl<'a> SpecializedPass<'a> for SobelPostBlur<'a> {
    type Parent = GaussianBlur<'a>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}
