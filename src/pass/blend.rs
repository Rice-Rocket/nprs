use glam::{Mat2, Vec2};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::{pixel::{rgba::Rgba, Pixel}, sampler::Sampler, Image}, pixel::Rgb, render_graph::ANY_IMAGE, SubPass};

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

impl Default for Blend {
    fn default() -> Self {
        Self {
            mode: BlendMode::Add,
            rotate_a: 0.0,
            rotate_b: 0.0,
            scale_a: Vec2::ONE,
            scale_b: Vec2::ONE,
            invert_a: false,
            invert_b: false,
            invert: false,
            strength: 1.0,
        }
    }
}

#[derive(FromParsedValue)]
pub enum BlendMode {
    /// Add the channel values of the first image with the corresponding channel values of the
    /// second image.
    Add,
    /// Subtract the channel values of the second image from the corresponding channel values of the
    /// first image.
    Subtract,
    /// Multiply the channel values of the first image with the corresponding channel values of the
    /// second image.
    Multiply,
    Screen,
    Overlay(LuminanceMethod),
    SoftLight(LuminanceMethod),
    ColorDodge,
    ColorBurn,
    VividLight(LuminanceMethod),
}

impl Blend {
    pub fn from_mode(mode: BlendMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }
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

            let mut a_rgba = im_a.sample_absolute(pos_a, Sampler::LINEAR_REPEAT);
            let mut b_rgba = im_b.sample_absolute(pos_b, Sampler::LINEAR_REPEAT);

            let a = if self.invert_a {
                a_rgba.rgb().invert()
            } else {
                a_rgba.rgb()
            };

            let b = if self.invert_b {
                b_rgba.rgb().invert()
            } else {
                b_rgba.rgb()
            };

            let mut col = match self.mode {
                BlendMode::Add => a + b,
                BlendMode::Subtract => a - b,
                BlendMode::Multiply => a * b,
                BlendMode::Screen => {
                    (a.invert() * b.invert()).invert()
                },
                BlendMode::Overlay(lum) => {
                    if lum.luminance(a.r, a.g, a.b) < 0.5 {
                        a * b * 2.0
                    } else {
                        (a.invert() * b.invert() * 2.0).invert()
                    }
                },
                BlendMode::SoftLight(lum) => {
                    if lum.luminance(b.r, b.g, b.b) < 0.5 {
                        a * b * 2.0 + (a * a) * (b * 2.0).invert()
                    } else {
                        a * 2.0 * b.invert() + a.sqrt() * (b * 2.0 - Rgb::splat(1.0))
                    }
                },
                BlendMode::ColorDodge => {
                    (a / (b - Rgb::splat(0.001)).invert()).saturate()
                },
                BlendMode::ColorBurn => {
                    (a.invert() / (b + Rgb::splat(0.001))).invert().saturate()
                },
                BlendMode::VividLight(lum) => {
                    if lum.luminance(b.r, b.g, b.b) <= 0.5 {
                        (a.invert() / ((b + Rgb::splat(0.001)) * 2.0)).invert()
                    } else {
                        a / ((b - Rgb::splat(0.001)).invert() * 2.0)
                    }
                },
            }.saturate();

            if self.invert {
                col = col.invert();
            }

            let final_col = a + (col - a) * self.strength;
            pixel.r = final_col.r;
            pixel.g = final_col.g;
            pixel.b = final_col.b;
            pixel.a = a_rgba.a;
        })
    }
}

impl SubPass for Blend {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        // TODO: unnecessary clone here
        let im_a = target.clone();
        self.apply(target, &[&im_a, aux_images[0]]);
    }
}
