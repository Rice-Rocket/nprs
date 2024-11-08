use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

pub struct Luminance {

}

impl Luminance {
    const NAME: &'static str = "luminance";
}

impl Pass for Luminance {
    fn name() -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _dependencies: &[&Image<4, f32, Rgba<f32>>]) {
        target.map_in_place(|pixel| {
            let l = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
            *pixel = Rgba::splat(l);
        });
    }
}
