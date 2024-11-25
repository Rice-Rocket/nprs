use glam::UVec2;
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{pixel::Rgba, render_graph::ANY_IMAGE, Image, Pass};

#[derive(ParsePass, FromParsedValue)]
#[nprs(from = SharpnessBuilder)]
pub struct Sharpness {
    kernel: Vec<f32>
}

impl Sharpness {
    pub fn new(amount: f32) -> Sharpness {
        let neighbor = -amount;
        let center = amount * 4.0 + 1.0;

        let kernel = vec![
            0.0, neighbor, 0.0,
            neighbor, center, neighbor,
            0.0, neighbor, 0.0,
        ];

        Sharpness { kernel }
    }
}

impl Pass for Sharpness {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        *target = source.convolve(&self.kernel, UVec2::new(3, 3));
    }
}

#[derive(FromParsedValue)]
struct SharpnessBuilder {
    amount: f32,
}

impl From<SharpnessBuilder> for Sharpness {
    fn from(value: SharpnessBuilder) -> Self {
        Sharpness::new(value.amount)
    }
}
