use glam::{Vec2, Vec3};

use crate::{image::{pixel::rgba::Rgba, Image}, pass::SubPass};

pub struct TangentFlowStructureTensor;

impl SubPass for TangentFlowStructureTensor {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        target.for_each(|pixel| {
            let g: Vec3 = pixel.rgb().into();

            let lambda1 = 0.5 * (g.y + g.x + f32::sqrt((g.y * g.y - 2.0 * g.x * g.y + g.x * g.x + 4.0 * g.z * g.z).max(0.0)));
            let lambda2 = 0.5 * (g.y + g.x + f32::sqrt((g.y * g.y + 2.0 * g.x * g.y + g.x * g.x + 4.0 * g.z * g.z).max(0.0)));

            let d = Vec2::new(g.x - lambda1, g.z);
            let len_d = d.length();

            let t = if len_d > 0.0 { d / len_d } else { Vec2::Y };
            let a = if lambda1 + lambda2 > 0.0 {
                (lambda2 - lambda1) / (lambda2 + lambda1)
            } else {
                0.0
            };

            pixel.r = t.x;
            pixel.g = t.y;
            pixel.b = len_d;
            pixel.a = a;
        })
    }
}
