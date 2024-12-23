use nprs_derive::FromParsedValue;

use crate::{image::format::PixelFormat, pass::luminance::LuminanceMethod};

use super::{luma_alpha::LumaAlpha, rgb::Rgb, rgba::Rgba, Color, FromPixel, Pixel};

#[derive(FromParsedValue, Clone, Copy, Debug)]
#[nprs(from = Color)]
pub struct Luma<F: PixelFormat> {
    pub v: F
}

impl<F: PixelFormat> Pixel<1> for Luma<F> {
    type Format = F;

    const BLACK: Self = Self { v: F::BLACK };
    const WHITE: Self = Self { v: F::WHITE };

    fn from_channels(channels: [Self::Format; 1]) -> Self {
        Luma { v: channels[0] }
    }

    fn channels(&self) -> [Self::Format; 1] {
        [self.v]
    }

    fn invert(self) -> Self {
        Self { v: self.v.invert() }
    }
}

impl<F: PixelFormat> FromPixel<Luma<F>> for Luma<F> {
    fn from_pixel(pixel: Luma<F>) -> Self {
        pixel
    }
}

impl<F: PixelFormat> FromPixel<LumaAlpha<F>> for Luma<F> {
    fn from_pixel(pixel: LumaAlpha<F>) -> Self {
        let l = pixel.v.to_scaled_float() * pixel.a.to_scaled_float();
        Self { v: F::from_scaled_float(l) }
    }
}

impl<F: PixelFormat> FromPixel<Rgb<F>> for Luma<F> {
    fn from_pixel(pixel: Rgb<F>) -> Self {
        let (r, g, b) = (pixel.r.to_scaled_float(), pixel.g.to_scaled_float(), pixel.b.to_scaled_float());
        let l = LuminanceMethod::Standard.luminance(r, g, b);
        Self { v: F::from_scaled_float(l) }
    }
}

impl<F: PixelFormat> FromPixel<Rgba<F>> for Luma<F> {
    fn from_pixel(pixel: Rgba<F>) -> Self {
        let (r, g, b) = (pixel.r.to_scaled_float(), pixel.g.to_scaled_float(), pixel.b.to_scaled_float());
        let l = LuminanceMethod::Standard.luminance(r, g, b);
        Self { v: F::from_scaled_float(l * pixel.a.to_scaled_float()) }
    }
}

impl<F: PixelFormat> From<Color> for Luma<F> {
    fn from(value: Color) -> Self {
        match value {
            Color::Luma(l) => Luma { v: F::from_scaled_float(l) },
            Color::LumaU8(l) => Luma { v: F::from_scaled_float(l.to_scaled_float()) },
            Color::Rg(..) => Luma::<F>::from_pixel(LumaAlpha::<F>::from(value)),
            Color::RgU8(..) => Luma::<F>::from_pixel(LumaAlpha::<F>::from(value)),
            Color::Rgb(..) => Luma::<F>::from_pixel(Rgb::<F>::from(value)),
            Color::RgbU8(..) => Luma::<F>::from_pixel(Rgb::<F>::from(value)),
            Color::Rgba(..) => Luma::<F>::from_pixel(Rgba::<F>::from(value)),
            Color::RgbaU8(..) => Luma::<F>::from_pixel(Rgba::<F>::from(value)),
        }
    }
}
