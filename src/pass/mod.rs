use crate::image::{pixel::rgba::Rgba, Image};

pub mod luminance;
pub mod sobel;
pub mod merge;

pub trait Pass {
    fn name() -> &'static str
    where
        Self: Sized;

    /// The passes this pass will be guaranteed to run after.
    fn dependencies(&self) -> &[&'static str];

    /// The image this pass will write to.
    fn target(&self) -> &'static str;

    /// The images other than the target image this pass will read from.
    fn auxiliary_images(&self) -> &[&'static str];

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}
