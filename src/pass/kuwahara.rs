use glam::{Mat2, Vec2, Vec3, Vec4, Vec4Swizzles as _};
use nprs_derive::{FromParsedValue, Pass};

use crate::{image::{pixel::{rgba::Rgba, Pixel}, sampler::{WrapMode, WrapMode2D}, Image}, pass::tfm::TangentFlowMap, render_graph::ANY_IMAGE};

use super::Pass;

/// A pass that applies the kuwahara filter.
#[derive(Pass, FromParsedValue)]
pub struct Kuwahara {
    kernel_size: u32,
    sharpness: f32,
    hardness: f32,
    #[nprs(default = 1.0)]
    alpha: f32,
    #[nprs(default = 0.58)]
    zero_crossing: f32,
    #[nprs(default = None)]
    zeta: Option<f32>,
    #[nprs(default = 1)]
    passes: u32,
}

impl Kuwahara {
    pub const NAME: &'static str = "kuwahara";

    /// Creates a new [`Kuwahara`] pass with default options.
    pub fn new() -> Kuwahara {
        Kuwahara {
            kernel_size: 20,
            sharpness: 8.0,
            hardness: 8.0,
            alpha: 1.0,
            zero_crossing: 0.58,
            zeta: None,
            passes: 1,
        }
    }

    /// The size of the kuwahara kernel. Larger kernel sizes create larger so-called "brush strokes".
    ///
    /// Defaults to `20`
    pub fn kernel_size(mut self, kernel_size: u32) -> Self {
        self.kernel_size = kernel_size;
        self
    }

    /// Defaults to `8.0`
    pub fn sharpness(mut self, sharpness: f32) -> Self {
        self.sharpness = sharpness;
        self
    }

    /// Defaults to `8.0`
    pub fn hardness(mut self, hardness: f32) -> Self {
        self.hardness = hardness;
        self
    }

    /// Defaults to `1.0`
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    /// Defaults to `0.58`
    pub fn zero_crossing(mut self, zero_crossing: f32) -> Self {
        self.zero_crossing = zero_crossing;
        self
    }

    /// Defaults to [`None`]
    pub fn zeta(mut self, zeta: Option<f32>) -> Self {
        self.zeta = zeta;
        self
    }

    /// Defaults to `1`
    pub fn passes(mut self, passes: u32) -> Self {
        self.passes = passes;
        self
    }
}

impl Pass for Kuwahara {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE, TangentFlowMap::NAME]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];
        let tfm = aux_images[1];

        let zeta = if let Some(zeta) = self.zeta {
            zeta
        } else {
            1.0 / (self.kernel_size as f32 / 2.0)
        };

        let kernel_radius = self.kernel_size / 2;

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
