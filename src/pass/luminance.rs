use std::marker::PhantomData;

use crate::{image::{pixel::rgba::Rgba, Image}, render_graph::{ANY_IMAGE, MAIN_IMAGE}};

use super::{Pass, SubPass};

/// A pass that computes the luminance of each pixel on the `target` image.
pub struct Luminance<M: LuminanceMethod> {
    _phantom: PhantomData<M>,
}

impl<M: LuminanceMethod> Luminance<M> {
    pub const NAME: &'static str = "luminance";
    
    pub fn new() ->Self {
        Self { _phantom: PhantomData }
    }
}

impl<'a, M: LuminanceMethod> Pass<'a> for Luminance<M> {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        target.for_each_with_positions(|pixel, pos| {
            let main_pixel = source.load(pos);
            let l = M::luminance(main_pixel.r, main_pixel.g, main_pixel.b);
            pixel.r = l;
        });
    }
}

impl<M: LuminanceMethod> SubPass for Luminance<M> {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        target.for_each(|pixel| {
            let l = M::luminance(pixel.r, pixel.g, pixel.b);
            pixel.r = l;
        })
    }
}

pub trait LuminanceMethod {
    fn luminance(r: f32, g: f32, b: f32) -> f32;
}

pub struct LuminanceStandardMethod;

impl LuminanceMethod for LuminanceStandardMethod {
    fn luminance(r: f32, g: f32, b: f32) -> f32 {
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

pub struct LuminanceFastPerceivedMethod;

impl LuminanceMethod for LuminanceFastPerceivedMethod {
    fn luminance(r: f32, g: f32, b: f32) -> f32 {
        0.299 * r + 0.587 * g + 0.114 * b
    }
}

pub struct LuminancePerceivedMethod;

impl LuminanceMethod for LuminancePerceivedMethod {
    fn luminance(r: f32, g: f32, b: f32) -> f32 {
        f32::sqrt(0.299 * r * r + 0.587 * g * g + 0.114 * b * b)
    }
}
