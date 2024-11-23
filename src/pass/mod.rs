use thiserror::Error;

use crate::{image::{pixel::rgba::Rgba, Image}, parser::{interpreter::ParsedValue, ParseValueError}};

mod tfm;
mod blur;
mod luminance;
mod kuwahara;
mod voronoi;
mod difference_of_gaussians;
mod blend;
mod texture;

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

pub type RegistrationValueParser = fn(ParsedValue) -> Result<Box<dyn Pass>, RenderPassError>;

pub struct PassRegistration {
    name: &'static str,
    value_parser: RegistrationValueParser,
}

inventory::collect!(PassRegistration);

impl dyn Pass {
    #[doc(hidden)]
    pub const fn register_pass(name: &'static str, value_parser: RegistrationValueParser) -> PassRegistration {
        PassRegistration { name, value_parser }
    }
}

pub trait FromNamedParsedValue: Sized {
    fn from_named_parsed_value(name: &str, value: ParsedValue) -> Result<Self, RenderPassError>;
}

impl FromNamedParsedValue for Box<dyn Pass> {
    fn from_named_parsed_value(name: &str, value: ParsedValue) -> Result<Self, RenderPassError> {
        for pass in inventory::iter::<PassRegistration> {
            if name == pass.name {
                return (pass.value_parser)(value);
            }
        }

        Err(RenderPassError::UnknownPass(name.to_string()))
    }
}
