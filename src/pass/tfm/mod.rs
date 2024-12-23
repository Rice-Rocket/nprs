use nprs_derive::{FromParsedValue, ParsePass};
use sobel::Sobel;
use structure_tensor::TangentFlowStructureTensor;

use crate::{image::{pixel::rgba::Rgba, Image}, render_graph::{ANY_IMAGE, MAIN_IMAGE}};

use super::{blur::{box_blur::BoxBlur, gaussian_blur::GaussianBlur}, Pass, SubPass};

pub mod sobel;
mod structure_tensor;

#[derive(ParsePass, FromParsedValue)]
#[nprs(from = TangentFlowMapBuilder)]
pub struct TangentFlowMap {
    sobel_pre_blur: BoxBlur,
    sobel: Sobel,
    sobel_post_blur: GaussianBlur,
    structure_tensor: TangentFlowStructureTensor,
}

impl TangentFlowMap {
    pub fn new(
        pre_blur_kernel_size: usize,
        post_blur_sigma: f32,
    ) -> Self {
        Self {
            sobel_pre_blur: BoxBlur::new(pre_blur_kernel_size),
            sobel: Sobel,
            sobel_post_blur: GaussianBlur::new(post_blur_sigma, (post_blur_sigma * 2.45).floor() as usize),
            structure_tensor: TangentFlowStructureTensor,
        }
    }
}

impl Pass for TangentFlowMap {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        self.sobel_pre_blur.apply(target, aux_images);
        self.sobel.apply_subpass(target, aux_images);
        self.sobel_post_blur.apply_subpass(target, aux_images);
        self.structure_tensor.apply_subpass(target, aux_images);
    }
}

#[derive(FromParsedValue)]
pub struct TangentFlowMapBuilder {
    pre_blur_kernel_size: usize,
    post_blur_sigma: f32,
}

impl From<TangentFlowMapBuilder> for TangentFlowMap {
    fn from(builder: TangentFlowMapBuilder) -> Self {
        TangentFlowMap::new(builder.pre_blur_kernel_size, builder.post_blur_sigma)
    }
}
