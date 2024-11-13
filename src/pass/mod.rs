use crate::image::{pixel::rgba::Rgba, Image};

pub mod tfm;
pub mod blur;
pub mod luminance;
pub mod kuwahara;
pub mod voronoi;

pub trait Pass<'a> {
    fn name(&self) -> &'a str;

    /// The passes this pass will be guaranteed to run after.
    fn dependencies(&self) -> Vec<&'a str>;

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}

pub trait SubPass {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
}

pub trait SpecializedPass<'a> {
    type Parent: Pass<'a>;

    fn parent(&self) -> &Self::Parent;
}

impl<'a, T> Pass<'a> for T
where
    T: SpecializedPass<'a>
{
    fn name(&self) -> &'a str {
        self.parent().name()
    }

    fn dependencies(&self) -> Vec<&'a str> {
        self.parent().dependencies()
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        self.parent().apply(target, aux_images)
    }
}

pub trait SpecializedSubPass {
    type Parent: SubPass;

    fn parent(&self) -> &Self::Parent;
}

impl<T> SubPass for T
where
    T: SpecializedSubPass
{
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        self.parent().apply_subpass(target, aux_images);
    }
}
