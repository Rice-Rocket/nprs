use nprs_derive::{FromParsedValue, ParsePass};

use crate::{pass::luminance::Luminance, pixel::Rgba, render_graph::ANY_IMAGE, Image, Pass, SubPass};

use super::{blur::gaussian_blur::GaussianBlur, luminance::LuminanceMethod};

#[derive(ParsePass, FromParsedValue)]
#[nprs(from = BloomBuilder)]
pub struct Bloom {
    lum: LuminanceMethod,
    threshold: f32,
    blur: GaussianBlur,
    gamma: f32,
    intensity: f32,
}

impl Pass for Bloom {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        *target = source.map(|pixel| {
            if self.lum.luminance(pixel.r, pixel.g, pixel.b) > self.threshold {
                Rgba::new(1.0, 1.0, 1.0, 1.0)
            } else {
                Rgba::new(0.0, 0.0, 0.0, 1.0)
            }
        });

        self.blur.apply_subpass(target, &[]);

        target.for_each(|pixel| {
            pixel.r = self.intensity * pixel.r.powf(self.gamma);
            pixel.g = self.intensity * pixel.g.powf(self.gamma);
            pixel.b = self.intensity * pixel.b.powf(self.gamma);
        });
    }
}

#[derive(FromParsedValue)]
struct BloomBuilder {
    lum: LuminanceMethod,
    threshold: f32,
    sigma: f32,
    #[nprs(default = (__sigma * 3.5).floor() as usize)]
    kernel_radius: usize,
    #[nprs(default = 1.0 / 2.2)]
    gamma: f32,
    intensity: f32,
}

impl From<BloomBuilder> for Bloom {
    fn from(value: BloomBuilder) -> Self {
        Bloom {
            lum: value.lum,
            threshold: value.threshold,
            blur: GaussianBlur::new(value.sigma, value.kernel_radius),
            gamma: value.gamma,
            intensity: value.intensity,
        }
    }
}
