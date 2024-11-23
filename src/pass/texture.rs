use nprs_derive::{FromParsedValue, ParsePass};

use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

#[derive(ParsePass, FromParsedValue)]
pub struct Texture {
    im: Image<4, f32, Rgba<f32>>,
}

impl Pass for Texture {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        *target = self.im.clone();
    }
}
