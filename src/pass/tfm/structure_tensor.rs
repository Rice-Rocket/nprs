use glam::{Vec2, Vec3};

use crate::{image::{pixel::rgba::Rgba, Image}, pass::Pass};

pub struct TangentFlowStructureTensor {

}

impl TangentFlowStructureTensor {
    const NAME: &'static str = "tfm";
}

impl<'a> Pass<'a> for TangentFlowStructureTensor {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        vec!["sobel_post_blur"]
    }

    fn target(&self) -> &'a str {
        "tangent_flow_map"
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        vec![]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        target.map_in_place(|pixel| {
            let g: Vec3 = pixel.rgb().into();

            let lambda1 = 0.5 * (g.y + g.x + f32::sqrt(g.y * g.y - 2.0 * g.x * g.y + g.x * g.x + 4.0 * g.z * g.z));
            let lambda2 = 0.5 * (g.y + g.x + f32::sqrt(g.y * g.y + 2.0 * g.x * g.y + g.x * g.x + 4.0 * g.z * g.z));

            let d = Vec2::new(g.x - lambda1, g.z);
            let len_d = d.length();

            let t = if len_d > 0.0 { d / len_d } else { Vec2::Y };
            let phi = -f32::atan2(t.y, t.x);
            let a = if lambda1 + lambda2 > 0.0 {
                (lambda2 - lambda1) / (lambda2 + lambda1)
            } else {
                0.0
            };

            pixel.r = t.x;
            pixel.g = t.y;
            pixel.b = phi;
            pixel.a = a;
        })
    }
}
