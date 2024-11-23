use glam::{Mat2, Vec2};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::{pixel::{rgba::Rgba, Pixel}, sampler::{Sampler, WrapMode2D}, Image}, render_graph::ANY_IMAGE};

use super::{luminance::LuminanceMethod, Pass};

#[derive(ParsePass, FromParsedValue)]
pub struct Blend {
    mode: BlendMode,
    /// Rotation of the first image, in radians.
    #[nprs(default = 0.0)]
    rotate_a: f32,
    /// Rotation of the second image, in radians.
    #[nprs(default = 0.0)]
    rotate_b: f32,
    /// Scale of the first image.
    #[nprs(default = Vec2::ONE)]
    scale_a: Vec2,
    /// Scale of the second image.
    #[nprs(default = Vec2::ONE)]
    scale_b: Vec2,
    #[nprs(default = false)]
    invert_a: bool,
    #[nprs(default = false)]
    invert_b: bool,
    #[nprs(default = false)]
    invert: bool,
    #[nprs(default = 1.0)]
    strength: f32,
}

#[derive(FromParsedValue)]
pub enum BlendMode {
    /// Add the channel values of the first image with the corresponding channel values of the
    /// second image.
    Add,
    /// Multiply the channel values of the first image with the corresponding channel values of the
    /// second image.
    Multiply,
    /// Multiply the channel values of the first image with the alpha of the second image.
    AlphaMultiply,
    Overlay(LuminanceMethod),
}

impl Pass for Blend {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE, ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let im_a = aux_images[0];
        let im_b = aux_images[1];

        target.for_each_with_positions(|pixel, pos| {
            let pos_a = Mat2::from_scale_angle(1.0 / self.scale_a, -self.rotate_a) * pos.as_vec2();
            let pos_b = Mat2::from_scale_angle(1.0 / self.scale_b, -self.rotate_b) * pos.as_vec2();

            let mut a = im_a.sample_absolute(pos_a, Sampler::LINEAR_REPEAT);
            let mut b = im_b.sample_absolute(pos_b, Sampler::LINEAR_REPEAT);

            if self.invert_a {
                a = a.invert();
            }

            if self.invert_b {
                b = b.invert();
            }

            let mut col = match self.mode {
                BlendMode::Add => a + b,
                BlendMode::Multiply => a * b,
                BlendMode::AlphaMultiply => a * b.a,
                BlendMode::Overlay(lum) => {
                    if lum.luminance(a.r, a.g, a.b) < 0.5 {
                        a * b * 2.0
                    } else {
                        (a.invert() * b.invert() * 2.0).invert()
                    }
                },
            }.saturate();

            if self.invert {
                col = col.invert();
            }

            *pixel = a + (col - a) * self.strength;
        })
    }
}
