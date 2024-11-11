use crate::image::{pixel::rgba::Rgba, Image};

pub mod tfm;
pub mod blur;
pub mod luminance;
pub mod merge;
pub mod kuwahara;
pub mod stipple;

pub trait Pass<'a> {
    fn name(&self) -> &'a str;

    /// The passes this pass will be guaranteed to run after.
    fn dependencies(&self) -> Vec<&'a str>;

    /// The image this pass will write to.
    fn target(&self) -> &'a str;

    /// The images other than the target image this pass will read from.
    fn auxiliary_images(&self) -> Vec<&'a str>;

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]);
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

    fn target(&self) -> &'a str {
        self.parent().target()
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        self.parent().auxiliary_images()
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        self.parent().apply(target, aux_images)
    }
}
