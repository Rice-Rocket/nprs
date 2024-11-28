use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::{pixel::rgba::Rgba, Image}, pixel::Rgb};

use super::Pass;

#[derive(ParsePass, FromParsedValue)]
pub struct Texture(TextureType);

#[derive(FromParsedValue)]
pub enum TextureType {
    Image(Image<4, f32, Rgba<f32>>),
    Constant(Rgb<f32>),
}

impl Pass for Texture {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        match &self.0 {
            TextureType::Image(im) => *target = im.clone(),
            TextureType::Constant(col) => {
                target.for_each(|pixel| {
                    pixel.r = col.r;
                    pixel.g = col.g;
                    pixel.b = col.b;
                    pixel.a = 1.0;
                })
            },
        }
    }
}
