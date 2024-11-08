use std::marker::PhantomData;

use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

pub struct Luminance<M: LuminanceMethod> {
    _phantom: PhantomData<M>,
}

impl<M: LuminanceMethod> Luminance<M> {
    const NAME: &'static str = "luminance";

    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<M: LuminanceMethod> Pass for Luminance<M> {
    fn name() -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _dependencies: &[&Image<4, f32, Rgba<f32>>]) {
        target.map_in_place(|pixel| {
            let l = M::luminance(pixel.r, pixel.g, pixel.b);
            pixel.r = l;
            pixel.g = l;
            pixel.b = l;
            pixel.a = 1.0;
        });
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
