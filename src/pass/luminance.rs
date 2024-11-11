use std::marker::PhantomData;

use crate::image::{pixel::rgba::Rgba, Image};

use super::{Pass, SpecializedPass};

/// A pass that computes the luminance of each pixel on the `target` image.
pub struct Luminance<'a, M: LuminanceMethod> {
    /// The name of this pass.
    name: &'a str,

    /// The image to write the resulting luminance to.
    ///
    /// If `source` is not specified, this is also the image used to compute the luminance,
    /// resulting in the operation being performed in-place.
    target: &'a str,

    /// The image to compute the luminance of.
    source: Option<&'a str>,

    /// The passes to run before this pass.
    dependencies: Vec<&'a str>,

    _phantom: PhantomData<M>,
}

impl<'a, M: LuminanceMethod> Luminance<'a, M> {
    pub fn new(
        name: &'a str,
        target: &'a str,
        dependencies: Vec<&'a str>,
    ) ->Self {
        Self { name, target, dependencies, source: None, _phantom: PhantomData }
    }

    pub fn with_source(mut self, source: &'a str) -> Self {
        self.source = Some(source);
        self
    }
}

impl<'a, M: LuminanceMethod> Pass<'a> for Luminance<'a, M> {
    fn name(&self) -> &'a str {
        self.name
    }

    fn dependencies(&self) -> Vec<&'a str> {
        self.dependencies.clone()
    }

    fn target(&self) -> &'a str {
        self.target
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        if let Some(source) = self.source {
            vec![source]
        } else {
            vec![]
        }
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        if self.source.is_some() {
            let source = aux_images[0];

            target.for_each_with_positions(|pixel, pos| {
                let main_pixel = source.load(pos);
                let l = M::luminance(main_pixel.r, main_pixel.g, main_pixel.b);
                pixel.r = l;
            });
        } else {
            target.for_each(|pixel| {
                let l = M::luminance(pixel.r, pixel.g, pixel.b);
                pixel.r = l;
            })
        }
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

pub struct MainLuminance<'a, M: LuminanceMethod>(Luminance<'a, M>);

impl<M: LuminanceMethod> MainLuminance<'_, M> {
    pub fn new() -> Self {
        Self(Luminance::new("main_luminance", "main_luminance", vec![]).with_source("main"))
    }
}

impl<'a, M: LuminanceMethod> SpecializedPass<'a> for MainLuminance<'a, M> {
    type Parent = Luminance<'a, M>;

    fn parent(&self) -> &Self::Parent {
        &self.0
    }
}
