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

    fn target(&self) -> &'static str {
        "tangent_flow_map"
    }

    fn auxiliary_images(&self) -> &[&'static str] {
        &["main"]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let main_image = aux_images[0];

        target.map_in_place_with_positions(|pixel, pos| {
            let main_pixel = main_image.load(pos);
            let l = M::luminance(main_pixel.r, main_pixel.g, main_pixel.b);
            pixel.r = l;
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
