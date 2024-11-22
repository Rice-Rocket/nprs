use std::{collections::HashMap, marker::PhantomData};

use nprs_derive::{FromParsedValue, Pass};

use crate::{image::{pixel::rgba::Rgba, Image}, parser::{interpreter::ParsedValue, FromParsedValue, ParseValueError}, pass::{PassRegistration, RegistrationValueParser, RenderPassError}, render_graph::ANY_IMAGE};

use super::{Pass, SubPass};

/// A pass that computes the luminance of each pixel on the `target` image.
#[derive(Pass, FromParsedValue)]
pub struct Luminance {
    method: LuminanceMethod,
}

impl Luminance {
    pub fn new(method: LuminanceMethod) -> Self {
        Self { method }
    }
}

impl Pass for Luminance {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        target.for_each_with_positions(|pixel, pos| {
            let main_pixel = source.load(pos);
            let l = self.method.luminance(main_pixel.r, main_pixel.g, main_pixel.b);
            pixel.r = l;
            pixel.g = l;
            pixel.b = l;
            pixel.a = 1.0;
        });
    }
}

impl SubPass for Luminance {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        target.for_each(|pixel| {
            let l = self.method.luminance(pixel.r, pixel.g, pixel.b);
            pixel.r = l;
        })
    }
}

#[derive(FromParsedValue, Clone, Copy)]
pub enum LuminanceMethod {
    Standard,
    FastPerceived,
    Perceived,
}

impl LuminanceMethod {
    pub fn luminance(&self, r: f32, g: f32, b: f32) -> f32 {
        match self {
            LuminanceMethod::Standard => 0.2126 * r + 0.7152 * g + 0.0722 * b,
            LuminanceMethod::FastPerceived => 0.299 * r + 0.587 * g + 0.114 * b,
            LuminanceMethod::Perceived => f32::sqrt(0.299 * r * r + 0.587 * g * g + 0.114 * b * b),
        }
    }
}
