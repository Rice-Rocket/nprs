use crate::image::{pixel::rgba::Rgba, Image};

pub mod tfm;
pub mod blur;
pub mod luminance;
pub mod kuwahara;
pub mod voronoi;

/// A render pass that represents a node in the render graph.
pub trait Pass<'a> {
    /// The name of this [`Pass`].
    fn name(&self) -> &'a str;

    /// The passes this [`Pass`] will be guaranteed to run after.
    fn dependencies(&self) -> Vec<&'a str>;

    /// Apply this [`Pass`] to the `target` image, given the requisite auxiliary images from graph
    /// connections.
    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}

pub trait SubPass {
    /// Apply this [`SubPass`] to the `target` image, given the requisite auxiliary images.
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}
