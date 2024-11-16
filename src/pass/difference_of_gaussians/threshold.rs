use glam::{Vec2, Vec4};

use crate::{image::{pixel::rgba::Rgba, sampler::Sampler, Image}, pass::SubPass};

use super::{gaussian, FDoGThresholdMode};

pub struct FDoGBlur2Theshold {
    /// Line Integral Deviation.
    pub sigma_m: f32,

    pub integral_convolution_stepsizes: Vec2,
    pub threshold_mode: FDoGThresholdMode,

    /// Whether or not to invert the output.
    pub invert: bool,
}

impl SubPass for FDoGBlur2Theshold {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let tfm = aux_images[0];
        let source = target.clone();
        let pixel_size = 1.0 / target.resolution().as_vec2();

        target.for_each_with_uvs(|pixel, uv| {
            let kernel_size = self.sigma_m * 2.0;
            let mut w = 1.0;
            let mut g = pixel.a;

            let t = tfm.sample(uv, Sampler::LINEAR_CLAMP);
            let v = Vec2::new(t.r, t.g) * pixel_size;

            let mut st0 = uv;
            let mut v0 = v;

            for d in 1..(kernel_size.floor() as i32) {
                st0 += v0 * self.integral_convolution_stepsizes.x;
                let c = source.sample(st0, Sampler::LINEAR_CLAMP).a;
                let g1 = gaussian(self.sigma_m, d as f32);

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
                let g1 = gaussian(self.sigma_m, d as f32);

                g += g1 * c;
                w += g1;

                let v = tfm.sample(st1, Sampler::LINEAR_CLAMP);
                v1 = Vec2::new(v.r, v.g) * pixel_size;
            }

            let d = g / w;

            let mut output = match self.threshold_mode {
                FDoGThresholdMode::HyperbolicTangent { white_point, phi } => {
                    if d >= white_point { 1.0 } else { 1.0 + (phi * (d - white_point)).tanh() }
                },
                FDoGThresholdMode::Quantization { white_point: b, palette_size, phi } => {
                    let a = 1.0 / palette_size;

                    if d >= b { 1.0 } else { a * f32::floor((d.powf(phi) - (a * b / 2.0)) / (a * b) + 0.5) }
                },
                FDoGThresholdMode::SmoothQuantization { palette_size, phi } => {
                    let qn = (d * palette_size + 0.5).floor() / palette_size;
                    let qs = smoothstep(-2.0, 2.0, phi * (d - qn) * 10.0) - 0.5;
                    
                    qn + qs / palette_size
                },
                FDoGThresholdMode::WhitePoint { white_point } => {
                    if d > white_point { 1.0 } else { 0.0 }
                },
                FDoGThresholdMode::None => {
                    d
                },
            };

            if self.invert {
                output = 1.0 - output;
            }

            pixel.a = output;
        });
    }
}

fn smoothstep(a: f32, b: f32, mut t: f32) -> f32 {
    t = f32::clamp((t - a) / (b - a), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
