use glam::{Mat2, Vec2, Vec3, Vec4, Vec4Swizzles as _};

use crate::image::{pixel::{rgba::Rgba, Pixel}, sampler::{WrapMode, WrapMode2D}, Image};

use super::Pass;

/// A pass that applies the kuwahara filter.
pub struct Kuwahara {
    pub kernel_size: u32,
    pub sharpness: f32,
    pub hardness: f32,
    pub alpha: f32,
    pub zero_crossing: f32,
    pub zeta: Option<f32>,
    pub passes: u32,
}

impl Kuwahara {
    const NAME: &'static str = "kuwahara";
}

impl<'a> Pass<'a> for Kuwahara {
    fn name(&self) -> &'a str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'a str> {
        vec!["tfm"]
    }

    fn target(&self) -> &'a str {
        "main"
    }

    fn auxiliary_images(&self) -> Vec<&'a str> {
        vec!["tangent_flow_map"]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let tfm = aux_images[0];

        let zeta = if let Some(zeta) = self.zeta {
            zeta
        } else {
            1.0 / (self.kernel_size as f32 / 2.0)
        };

        let kernel_radius = self.kernel_size / 2;

        let source = target.clone();

        target.for_each_with_positions(|pixel, pos| {
            let t = tfm.load(pos);

            let a = kernel_radius as f32 * f32::clamp((self.alpha + t.a) / self.alpha, 0.1, 2.0);
            let b = kernel_radius as f32 * f32::clamp(self.alpha / (self.alpha + t.a), 0.1, 2.0);

            let phi = -f32::atan2(t.g, t.r);
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            let r = Mat2::from_cols(Vec2::new(cos_phi, -sin_phi), Vec2::new(sin_phi, cos_phi));
            let s = Mat2::from_cols(Vec2::new(0.5 / a, 0.0), Vec2::new(0.0, 0.5 / b));

            let sr = s * r;

            let max_x = f32::sqrt(a * a * cos_phi * cos_phi + b * b * sin_phi * sin_phi) as i32;
            let max_y = f32::sqrt(a * a * sin_phi * sin_phi + b * b * cos_phi * cos_phi) as i32;

            let sin_zero_cross = self.zero_crossing.sin();
            let eta = (zeta + self.zero_crossing.cos()) / (sin_zero_cross * sin_zero_cross);
            
            let mut m = [Vec4::ZERO; 8];
            let mut s = [Vec3::ZERO; 8];

            for y in -max_y..=max_y {
                for x in -max_x..=max_x {
                    let p = Vec2::new(x as f32, y as f32);
                    let mut v = sr * p;
                    let mut c: Vec3 = source.load_wrapped(pos.as_ivec2() + p.as_ivec2(), WrapMode2D::CLAMP).rgb().into();
                    
                    if v.dot(v) <= 0.25 {
                        c = c.clamp(Vec3::ZERO, Vec3::ONE);
                        let mut sum = 0.0;
                        let mut w = [0.0; 8];

                        let mut z: f32;
                        let mut vxx: f32;
                        let mut vyy: f32;

                        vxx = zeta - eta * v.x * v.x;
                        vyy = zeta - eta * v.y * v.y;
                        z = f32::max(0.0, v.y + vxx); 
                        w[0] = z * z;
                        sum += w[0];
                        z = f32::max(0.0, -v.x + vyy); 
                        w[2] = z * z;
                        sum += w[2];
                        z = f32::max(0.0, -v.y + vxx); 
                        w[4] = z * z;
                        sum += w[4];
                        z = f32::max(0.0, v.x + vyy); 
                        w[6] = z * z;
                        sum += w[6];
                        v = std::f32::consts::SQRT_2 / 2.0 * Vec2::new(v.x - v.y, v.x + v.y);
                        vxx = zeta - eta * v.x * v.x;
                        vyy = zeta - eta * v.y * v.y;
                        z = f32::max(0.0, v.y + vxx); 
                        w[1] = z * z;
                        sum += w[1];
                        z = f32::max(0.0, -v.x + vyy); 
                        w[3] = z * z;
                        sum += w[3];
                        z = f32::max(0.0, -v.y + vxx); 
                        w[5] = z * z;
                        sum += w[5];
                        z = f32::max(0.0, v.x + vyy); 
                        w[7] = z * z;
                        sum += w[7];

                        let g = f32::exp(-3.125 * v.dot(v)) / sum;

                        for k in 0..8 {
                            let wk = w[k] * g;
                            let cwk = c * wk;
                            m[k] += Vec4::new(cwk.x, cwk.y, cwk.z, wk);
                            s[k] += c * c * wk;
                        }
                    }
                }
            }

            let mut output = Rgba::BLACK;

            for k in 0..8 {
                let mkw = m[k].w;
                let mk = if mkw != 0.0 { m[k].xyz() / mkw } else { m[k].xyz() };
                let sk = if mkw != 0.0 { (s[k] / mkw - mk * mk).abs() } else { (s[k] - mk * mk).abs() };

                let sigma2 = sk.x + sk.y + sk.z;
                let w = 1.0 / (1.0 + f32::powf(self.hardness * 1000.0 * sigma2, 0.5 * self.sharpness));

                let m = mk * w;
                output = output + Rgba::new(m.x, m.y, m.z, w);
            }

            *pixel = (output / output.a).saturate();
        });
    }
}
