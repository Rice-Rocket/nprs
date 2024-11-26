use glam::{IVec2, UVec2};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::sampler::WrapMode2D, pixel::{Rgb, Rgba}, render_graph::ANY_IMAGE, Image, Pass};

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

#[derive(ParsePass, FromParsedValue)]
#[nprs(from = SharpnessBuilder)]
pub struct ContrastAdaptiveSharpness {
    sharpness: f32
}

impl Pass for ContrastAdaptiveSharpness {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        *target = source.map_with_positions(|pixel, pos| {
            let p = pos.as_ivec2();

            let a = source.load_wrapped(p + IVec2::new(-1, -1), WrapMode2D::CLAMP).rgb();
            let b = source.load_wrapped(p + IVec2::new(0, -1), WrapMode2D::CLAMP).rgb();
            let c = source.load_wrapped(p + IVec2::new(1, -1), WrapMode2D::CLAMP).rgb();
            let d = source.load_wrapped(p + IVec2::new(-1, 0), WrapMode2D::CLAMP).rgb();
            let e = pixel.rgb();
            let f = source.load_wrapped(p + IVec2::new(1, 0), WrapMode2D::CLAMP).rgb();
            let g = source.load_wrapped(p + IVec2::new(-1, 1), WrapMode2D::CLAMP).rgb();
            let h = source.load_wrapped(p + IVec2::new(0, 1), WrapMode2D::CLAMP).rgb();
            let i = source.load_wrapped(p + IVec2::new(1, 1), WrapMode2D::CLAMP).rgb();

            let mut min_rgb = min3(min3(d, e, f), b, h);
            let min_rgb2 = min3(min3(min_rgb, a, c), g, i);

            min_rgb = min_rgb + min_rgb2;
            
            let mut max_rgb = max3(max3(d, e, f), b, h);
            let max_rgb2 = max3(max3(max_rgb, a, c), g, i);

            max_rgb = max_rgb + max_rgb2;

            let rcp_m = Rgb::splat(1.0) / max_rgb;
            let mut amp = (min_rgb.min(Rgb::splat(2.0) - max_rgb) * rcp_m).saturate();
            amp = amp.sqrt();

            let w = amp * self.sharpness;
            let rcp_w = Rgb::splat(1.0) / (Rgb::splat(1.0) + Rgb::splat(4.0) * w);
            
            let output = ((b * w + d * w + f * w + h * w + e) * rcp_w).saturate();

            Rgba::new(output.r, output.g, output.b, 1.0)
        })
    }
}

fn min3(x: Rgb<f32>, y: Rgb<f32>, z: Rgb<f32>) -> Rgb<f32> {
    x.min(y).min(z)
}

fn max3(x: Rgb<f32>, y: Rgb<f32>, z: Rgb<f32>) -> Rgb<f32> {
    x.max(y).max(z)
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

impl From<SharpnessBuilder> for ContrastAdaptiveSharpness {
    fn from(value: SharpnessBuilder) -> Self {
        let amount = 10.0 + (7.0 - 10.0) * value.amount.clamp(0.0, 1.0);
        let sharpness = -(1.0 / amount);
        ContrastAdaptiveSharpness { sharpness }
    }
}
