use glam::{Vec2, Vec4};

use crate::{image::{pixel::rgba::Rgba, sampler::Sampler, Image}, pass::SubPass};

use super::gaussian;

pub struct FDoGAntiAlias {
    pub sigma_a: f32,
    pub integral_convolution_stepsizes: Vec2,
}

impl SubPass for FDoGAntiAlias {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let tfm = aux_images[0];
        let source = target.clone();
        let pixel_size = 1.0 / target.resolution().as_vec2();

        target.for_each_with_uvs(|pixel, uv| {
            let kernel_size = self.sigma_a * 2.0;
            let mut g = pixel.a;
            let mut w = 1.0;

            let t = tfm.sample(uv, Sampler::LINEAR_CLAMP);
            let v = Vec2::new(t.r, t.g) * pixel_size;

            let mut st0 = uv;
            let mut v0 = v;

            for d in 1..(kernel_size.floor() as i32) {
                st0 += v0 * self.integral_convolution_stepsizes.x;
                let c = source.sample(st0, Sampler::LINEAR_CLAMP).a;
                let g1 = gaussian(self.sigma_a, d as f32);

                g += g1 * c;
                w += g1;

                let v = tfm.sample(st0, Sampler::LINEAR_CLAMP);
                v0 = Vec2::new(v.r, v.g) * pixel_size;
            }

            let mut st1 = uv;
            let mut v1 = v;

            for d in 1..(kernel_size.floor() as i32) {
                st1 -= v1 * self.integral_convolution_stepsizes.y;
                let c = source.sample(st1, Sampler::LINEAR_CLAMP).a;
                let g1 = gaussian(self.sigma_a, d as f32);

                g += g1 * c;
                w += g1;

                let v = tfm.sample(st1, Sampler::LINEAR_CLAMP);
                v1 = Vec2::new(v.r, v.g) * pixel_size;
            }

            pixel.a = g / w;
        })
    }
}
