use glam::{IVec2, Vec2};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::sampler::WrapMode2D, pass::luminance::Luminance, pixel::Rgba, render_graph::ANY_IMAGE, Image, Pass, SubPass};

use super::gaussian;

#[derive(ParsePass, FromParsedValue)]
pub struct BasicDifferenceOfGaussians {
    #[nprs(default = 2)]
    kernel_size: i32,
    #[nprs(default = 2.0)]
    stdev: f32,
    #[nprs(default = 1.6)]
    stdev_scale: f32,
    #[nprs(default = 1.0)]
    sharpness: f32,
    #[nprs(default = 0.005)]
    white_point: f32,
    #[nprs(default = false)]
    invert: bool,
}

impl Pass for BasicDifferenceOfGaussians {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![Luminance::PASS_NAME]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        target.for_each_with_positions(|pixel, pos| {
            let mut blur = Vec2::ZERO;
            let mut kernel_sum = Vec2::ZERO;

            for x in -self.kernel_size..=self.kernel_size {
                let lum = source.load_wrapped(pos.as_ivec2() + IVec2::new(x, 0), WrapMode2D::CLAMP).r;
                let gauss = Vec2::new(gaussian(self.stdev, x as f32), gaussian(self.stdev * self.stdev_scale, x as f32));

                blur += lum * gauss;
                kernel_sum += gauss;
            }

            pixel.r = blur.x / kernel_sum.x;
            pixel.g = blur.y / kernel_sum.y;
        });

        let source = target.clone();

        target.for_each_with_positions(|pixel, pos| {
            let mut blur = Vec2::ZERO;
            let mut kernel_sum = Vec2::ZERO;

            for y in -self.kernel_size..=self.kernel_size {
                let lum = source.load_wrapped(pos.as_ivec2() + IVec2::new(0, y), WrapMode2D::CLAMP);
                let gauss = Vec2::new(gaussian(self.stdev, y as f32), gaussian(self.stdev * self.stdev_scale, y as f32));

                blur += Vec2::new(lum.r, lum.g) * gauss;
                kernel_sum += gauss;
            }

            blur /= kernel_sum;

            let d = blur.x - self.sharpness * blur.y;

            let mut d = if d >= self.white_point {
                1
            } else {
                0
            };

            if self.invert {
                d = 1 - d;
            }

            pixel.r = d as f32;
            pixel.g = d as f32;
            pixel.b = d as f32;
            pixel.a = 1.0;
        });
    }
}

impl SubPass for BasicDifferenceOfGaussians {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = target.clone();
        self.apply(target, &[&source]);
    }
}
