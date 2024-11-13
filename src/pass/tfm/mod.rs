use sobel::Sobel;
use structure_tensor::TangentFlowStructureTensor;

use crate::image::{pixel::rgba::Rgba, Image};

use super::{blur::{box_blur::BoxBlur, gaussian_blur::GaussianBlur}, Pass, SpecializedPass, SpecializedSubPass, SubPass};

pub mod sobel;
pub mod structure_tensor;

pub struct TangentFlowMap<'a> {
    sobel_pre_blur: SobelPreBlur<'a>,
    sobel: Sobel,
    sobel_post_blur: SobelPostBlur<'a>,
    structure_tensor: TangentFlowStructureTensor,
}

impl TangentFlowMap<'_> {
    pub const NAME: &'static str = "tangent_flow_map";

    pub fn new(
        pre_blur_kernel_size: usize,
        post_blur_sigma: f32,
    ) -> Self {
        Self {
            sobel_pre_blur: SobelPreBlur::new(pre_blur_kernel_size),
            sobel: Sobel,
            sobel_post_blur: SobelPostBlur::new(post_blur_sigma),
            structure_tensor: TangentFlowStructureTensor,
        }
    }
}

impl<'a> Pass<'a> for TangentFlowMap<'a> {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        vec!["main"]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        self.sobel_pre_blur.apply_subpass(target, aux_images);
        self.sobel.apply_subpass(target, aux_images);
        self.sobel_post_blur.apply_subpass(target, aux_images);
        self.structure_tensor.apply_subpass(target, aux_images);
    }
}

pub struct SobelPreBlur<'a>(BoxBlur<'a>);

impl SobelPreBlur<'_> {
    pub fn new(kernel_radius: usize) -> Self {
        Self(BoxBlur::new("", kernel_radius).with_source("main"))
    }
}

impl<'a> SpecializedSubPass for SobelPreBlur<'a> {
    type Parent = BoxBlur<'a>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}

pub struct SobelPostBlur<'a>(GaussianBlur<'a>);

impl SobelPostBlur<'_> {
    pub fn new(sigma: f32) -> Self {
        Self(GaussianBlur::new("", sigma))
    }
}

impl<'a> SpecializedSubPass for SobelPostBlur<'a> {
    type Parent = GaussianBlur<'a>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}
