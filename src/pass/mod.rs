use crate::image::{pixel::rgba::Rgba, Image};

pub mod luminance;

pub trait Pass {
    fn name() -> &'static str
    where
        Self: Sized;

    fn dependencies(&self) -> &[&'static str];

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, dependencies: &[&Image<4, f32, Rgba<f32>>]);
}
