use glam::Vec2;

use crate::{image::{pixel::rgba::Rgba, sampler::Sampler, Image}, pass::SubPass};

use super::gaussian;

pub struct FDoGBlur1 {
    /// DoG Deviation.
    pub sigma_e: f32,

    /// DoG Standard Deviation Scale.
    pub k: f32,

    /// DoG Sharpness.
    pub tau: f32,
}

impl SubPass for FDoGBlur1 {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let lum = aux_images[0];
        let tfm = aux_images[1];
        let pixel_size = 1.0 / target.resolution().as_vec2();

        target.for_each_with_uvs(|pixel, uv| {
            let t = tfm.sample(uv, Sampler::LINEAR_CLAMP);
            let mut n = Vec2::new(t.g, -t.r);
            let nabs = n.abs();
            let ds = 1.0 / (if nabs.x > nabs.y { nabs.x } else { nabs.y });
            n *= pixel_size;

            // let mut col = Vec2::splat(0.0);
            let mut col = Vec2::splat(lum.sample(uv, Sampler::LINEAR_CLAMP).r);
            let mut kernel_sum = Vec2::ONE;

            let kernel_size = if self.sigma_e * 2.0 > 1.0 { (self.sigma_e * 2.0).floor() as i32 } else { 1 };

            for x in (ds as i32)..=kernel_size {
                let g1 = gaussian(self.sigma_e, x as f32);
                let g2 = gaussian(self.sigma_e * self.k, x as f32);

                let c1 = lum.sample(uv - x as f32 * n, Sampler::LINEAR_CLAMP).r;
                let c2 = lum.sample(uv + x as f32 * n, Sampler::LINEAR_CLAMP).r;

                col.x += (c1 + c2) * g1;
                kernel_sum.x += 2.0 * g1;

                col.y += (c1 + c2) * g2;
                kernel_sum.y += 2.0 * g2;
            }

            col /= kernel_sum;

            pixel.r = 1.0;
            pixel.g = 1.0;
            pixel.b = 1.0;
            pixel.a = (1.0 + self.tau) * col.x - self.tau * col.y;
        });
    }
}
