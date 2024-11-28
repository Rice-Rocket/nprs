use glam::{Vec2, Vec2Swizzles};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{image::sampler::Sampler, pixel::Rgba, render_graph::ANY_IMAGE, Image, Pass};

#[derive(ParsePass, FromParsedValue)]
pub struct Crt {
    /// Curvature of the corners of the screen.
    #[nprs(default = 10.0)]
    curvature: f32,
    /// The width of the vignette.
    #[nprs(default = 30.0)]
    vignette_width: f32,
    /// Adjust width of CRT lines by 2 ^ line_size.
    #[nprs(default = 0)]
    line_size: u32,
    /// Strength of the CRT lines.
    #[nprs(default = 1.0)]
    line_strength: f32,
    /// Brightness of the CRT lines.
    #[nprs(default = 0.0)]
    line_brightness: f32,
}

impl Pass for Crt {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];
        let res = source.resolution().as_vec2();

        target.for_each_with_uvs(|pixel, uv| {
            let mut crt_uv = uv * 2.0 - 1.0;
            let offset = crt_uv.yx() / self.curvature;
            crt_uv += crt_uv * offset * offset;
            crt_uv = crt_uv * 0.5 + 0.5;

            let mut col = source.sample(crt_uv, Sampler::NEAREST_BLACK);

            crt_uv = crt_uv * 2.0 - 1.0;
            let mut vignette = self.vignette_width / res;
            vignette = smoothstep(Vec2::ZERO, vignette, 1.0 - crt_uv.abs());
            vignette = vignette.clamp(Vec2::ZERO, Vec2::ONE);

            let line_pos = uv.y * (res.y / 2u32.pow(self.line_size) as f32) * 2.0;
            col.g *= (f32::sin(line_pos) + 1.0) * 0.15 * self.line_strength + 1.0 * self.line_brightness;
            
            let rb = (f32::cos(line_pos) + 1.0) * 0.135 * self.line_strength + 1.0 * self.line_brightness;
            col.r *= rb;
            col.b *= rb;

            col = col.saturate() * vignette.x * vignette.y;

            pixel.r = col.r;
            pixel.g = col.g;
            pixel.b = col.b;
            pixel.a = 1.0;
        });
    }
}

fn smoothstep(a: Vec2, b: Vec2, mut t: Vec2) -> Vec2 {
    t = Vec2::clamp((t - a) / (b - a), Vec2::ZERO, Vec2::ONE);
    t * t * (3.0 - 2.0 * t)
}
