use std::collections::HashMap;

use blend::Blend;
use blur::{box_blur::BoxBlur, gaussian_blur::GaussianBlur};
use difference_of_gaussians::DifferenceOfGaussians;
use kuwahara::Kuwahara;
use luminance::Luminance;
use texture::Texture;
use tfm::TangentFlowMap;
use thiserror::Error;
use voronoi::RelaxedVoronoi;

use crate::{image::{pixel::rgba::Rgba, Image}, parser::{interpreter::ParsedValue, FromParsedValue, ParseValueError}};

pub mod tfm;
pub mod blur;
pub mod luminance;
pub mod kuwahara;
pub mod voronoi;
pub mod difference_of_gaussians;
pub mod blend;
pub mod texture;

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

#[derive(Debug, Error)]
pub enum RenderPassError {
    #[error(transparent)]
    ParseValue(#[from] ParseValueError),
    #[error("unknown pass '{0}'")]
    UnknownPass(String),
}

pub trait FromNamedParsedValue: Sized {
    fn from_named_parsed_value(name: &str, value: ParsedValue) -> Result<Self, RenderPassError>;
}

impl FromNamedParsedValue for Box<dyn Pass> {
    // TODO: Explicitly writing out each pass name is currently necessary for reading in the passes.
    // In the future, I'd like to have a system that allows deserialization of dyn traits (probably
    // something along the lines of what the typetag crate does)
    fn from_named_parsed_value(name: &str, value: ParsedValue) -> Result<Self, RenderPassError> {
        Ok(match name {
            "Texture" => Box::new(Texture::from_parsed_value(value)?),
            "Blend" => Box::new(Blend::from_parsed_value(value)?),
            "Luminance" => Box::new(Luminance::from_parsed_value(value)?),
            "TangentFlowMap" => Box::new(TangentFlowMap::from_parsed_value(value)?),
            "BoxBlur" => Box::new(BoxBlur::from_parsed_value(value)?),
            "GaussianBlur" => Box::new(GaussianBlur::from_parsed_value(value)?),
            "Kuwahara" => Box::new(Kuwahara::from_parsed_value(value)?),
            "DifferenceOfGaussians" => Box::new(DifferenceOfGaussians::from_parsed_value(value)?),
            _ => return Err(RenderPassError::UnknownPass(name.to_string())),
        })
    }
}
