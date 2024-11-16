use blur::{box_blur::BoxBlur, gaussian_blur::GaussianBlur};
use difference_of_gaussians::DifferenceOfGaussians;
use kuwahara::Kuwahara;
use luminance::Luminance;
use serde::Deserialize;
use tfm::TangentFlowMap;
use voronoi::RelaxedVoronoi;

use crate::image::{pixel::rgba::Rgba, Image};

pub mod tfm;
pub mod blur;
pub mod luminance;
pub mod kuwahara;
pub mod voronoi;
pub mod difference_of_gaussians;

/// A render pass that represents a node in the render graph.
pub trait Pass {
    /// The name of this [`Pass`].
    fn name(&self) -> &'static str;

    /// The passes this [`Pass`] will be guaranteed to run after.
    fn dependencies(&self) -> Vec<&'static str>;

    /// Apply this [`Pass`] to the `target` image, given the requisite auxiliary images from graph
    /// connections.
    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}

pub trait SubPass {
    /// Apply this [`SubPass`] to the `target` image, given the requisite auxiliary images.
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}

// TODO: This is currently necessary for reading in the passes.
// In the future, I'd like to have a system that allows deserialization of dyn traits (probably
// something along the lines of what the typetag crate does)
#[derive(Deserialize)]
#[serde(rename = "Pass")]
pub enum RenderPass {
    BoxBlur(BoxBlur),
    GaussianBlur(GaussianBlur),
    TangentFlowMap(TangentFlowMap),
    Luminance(Luminance),
    Kuwahara(Kuwahara),
    RelaxedVoronoi(RelaxedVoronoi),
    DifferenceOfGaussians(DifferenceOfGaussians),
}

impl RenderPass {
    pub fn into_pass(self) -> Box<dyn Pass> {
        match self {
            RenderPass::BoxBlur(p) => Box::new(p),
            RenderPass::GaussianBlur(p) => Box::new(p),
            RenderPass::TangentFlowMap(p) => Box::new(p),
            RenderPass::Luminance(p) => Box::new(p),
            RenderPass::Kuwahara(p) => Box::new(p),
            RenderPass::RelaxedVoronoi(p) => Box::new(p),
            RenderPass::DifferenceOfGaussians(p) => Box::new(p),
        }
    }
}
