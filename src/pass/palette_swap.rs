use std::{f32::consts::PI, ops::Range};

use glam::{Mat3, Vec3};
use nprs_derive::{FromParsedValue, ParsePass};

use crate::{pixel::{Rgb, Rgba}, render_graph::ANY_IMAGE, Image, Pass};

use super::luminance::LuminanceMethod;

#[derive(ParsePass, FromParsedValue)]
pub struct PaletteSwap {
    palette: PaletteSwapColors,
    mode: PalleteSwapMode,
}

impl Pass for PaletteSwap {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        *target = source.map(|pixel| {
            let v = self.mode.map(pixel.rgb());
            let idx = ((v * self.palette.size() as f32).floor() as usize).min(self.palette.size() - 1);
            let col = self.palette.colors[idx];

            Rgba::new(col.r, col.g, col.b, 1.0)
        });
    }
}

#[derive(FromParsedValue)]
pub enum PalleteSwapMode {
    Luminance(LuminanceMethod),
}

impl PalleteSwapMode {
    fn map(&self, col: Rgb<f32>) -> f32 {
        match self {
            PalleteSwapMode::Luminance(lum) => lum.luminance(col.r, col.g, col.b),
        }
    }
}

#[derive(FromParsedValue)]
#[nprs(from = PaletteSwapColorsBuilder)]
pub struct PaletteSwapColors {
    colors: Vec<Rgb<f32>>,
}

impl PaletteSwapColors {
    fn new_fixed(
        colors: Vec<Rgb<f32>>,
    ) -> PaletteSwapColors {
        PaletteSwapColors { colors }
    }

    #[allow(clippy::too_many_arguments)]
    fn generate(
        palette_size: u32,
        seed: u32,
        hue: PaletteSwapChannelMode,
        hue_contrast: PaletteSwapChannelMode,
        luminance: PaletteSwapChannelMode,
        luminance_contrast: PaletteSwapChannelMode,
        chroma: PaletteSwapChannelMode,
        chroma_contrast: PaletteSwapChannelMode,
        hue_mode: u32
    ) -> PaletteSwapColors {
        let base_hue = hue.get_color(seed) * 2.0 * PI;
        let hue_contrast = hue_contrast.get_color(seed + 2);
        let base_lum = luminance.get_color(seed + 13);
        let lum_contrast = luminance_contrast.get_color(seed + 3);
        let base_chroma = chroma.get_color(seed + 5);
        let chroma_contrast = chroma_contrast.get_color(seed + 7);

        let mut colors = Vec::new();

        for i in 0..palette_size {
            let linear = i as f32 / (palette_size as f32 - 1.0);
            let mut hue_offset = hue_contrast * linear * 2.0 * PI + (PI / 4.0);

            if hue_mode == 0 { hue_offset *= 0.0 };
            if hue_mode == 1 { hue_offset *= 0.25 };
            if hue_mode == 2 { hue_offset *= 0.33 };
            if hue_mode == 3 { hue_offset *= 0.66 };
            if hue_mode == 4 { hue_offset *= 0.75 };
            
            let lum_offset = base_lum + lum_contrast * linear;
            let chroma_offset = base_chroma + chroma_contrast * linear;
            
            colors.push(oklch_to_rgb(lum_offset, chroma_offset, base_hue + hue_offset));
        }

        PaletteSwapColors { colors }
    }

    fn size(&self) -> usize {
        self.colors.len()
    }
}

const LRGB_2_CONE: Mat3 = Mat3::from_cols_array(&[
    0.412_165_6, 0.211_859_1, 0.088_309_795,
    0.536_275_2, 0.680_718_96, 0.281_847_42,
    0.051_457_565, 0.107_406_58, 0.630_261_36
]);

const CONE_2_LAB: Mat3 = Mat3::from_cols_array(&[
    0.210_454_26, 1.977_998_5, 0.025_904_037,
    0.793_617_8, -2.428_592_2, 0.782_771_77,
    0.004_072_047, 0.450_593_7, -0.808_675_77
]);

const LAB_2_CONE: Mat3 = Mat3::from_cols_array(&[
    4.076_741_7, -1.268_438, -0.0041960863,
    -3.307_711_6, 2.609_757_4, -0.703_418_6,
    0.230_969_94, -0.341_319_38, 1.707_614_7
]);

const CONE_2_LRGB: Mat3 = Mat3::from_cols_array(&[
    1.0, 1.0, 1.0,
    0.396_337_78, -0.105_561_346, -0.089_484_18,
    0.215_803_76, -0.063_854_17, -1.291_485_5
]);

fn hash(mut n: u32) -> f32 {
    n = n.wrapping_shl(13) ^ n;
    n = n.wrapping_mul(n.wrapping_mul(n.wrapping_mul(15731).wrapping_add(789221))).wrapping_add(1376312589);
    (n & 0x7fffffff) as f32 / 0x7fffffff as f32
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn oklab_to_rgb(mut col: Vec3) -> Rgb<f32> {
    col = CONE_2_LRGB * col;
    col = col * col * col;
    col = LAB_2_CONE * col;
    Rgb::new(col.x, col.y, col.z)
}

fn oklch_to_rgb(l: f32, c: f32, h: f32) -> Rgb<f32> {
    oklab_to_rgb(Vec3::new(l, c * h.cos(), c * h.sin()))
}

#[derive(FromParsedValue)]
enum PaletteSwapChannelMode {
    Fixed(f32),
    Range(f32, f32),
}

impl PaletteSwapChannelMode {
    fn get_color(self, seed: u32) -> f32 {
        match self {
            PaletteSwapChannelMode::Fixed(v) => v.clamp(0.0, 1.0),
            PaletteSwapChannelMode::Range(min, max) => mix(min, max, hash(seed)).clamp(0.0, 1.0),
        }
    }
}

#[derive(FromParsedValue)]
enum PaletteSwapColorsBuilder {
    Generate {
        palette_size: u32,
        seed: u32,
        hue: PaletteSwapChannelMode,
        hue_contrast: PaletteSwapChannelMode,
        luminance: PaletteSwapChannelMode,
        luminance_contrast: PaletteSwapChannelMode,
        chroma: PaletteSwapChannelMode,
        chroma_contrast: PaletteSwapChannelMode,
        hue_mode: u32
    },
}

impl From<PaletteSwapColorsBuilder> for PaletteSwapColors {
    fn from(value: PaletteSwapColorsBuilder) -> Self {
        match value {
            PaletteSwapColorsBuilder::Generate {
                palette_size,
                seed,
                hue,
                hue_contrast,
                luminance,
                luminance_contrast,
                chroma,
                chroma_contrast,
                hue_mode,
            } => PaletteSwapColors::generate(
                palette_size,
                seed,
                hue,
                hue_contrast,
                luminance,
                luminance_contrast,
                chroma,
                chroma_contrast,
                hue_mode,
            ),
        }
    }
}
