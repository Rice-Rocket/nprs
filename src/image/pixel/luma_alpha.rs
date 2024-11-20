use nprs_derive::FromParsedValue;

use crate::{image::format::PixelFormat, parser::FromParsedValue};

use super::{luma::Luma, rgb::Rgb, rgba::Rgba, FromPixel, Pixel};

#[derive(FromParsedValue, Clone, Copy, Debug)]
pub struct LumaAlpha<F: PixelFormat> {
    pub v: F,
    pub a: F,
}

impl<F: PixelFormat> Pixel<2> for LumaAlpha<F> {
    type Format = F;

    const BLACK: Self = Self { v: F::BLACK, a: F::BLACK };
    const WHITE: Self = Self { v: F::WHITE, a: F::WHITE };

    fn from_channels(channels: [Self::Format; 2]) -> Self {
        LumaAlpha {
            v: channels[0],
            a: channels[1],
        }
    }

    fn channels(&self) -> [Self::Format; 2] {
        [self.v, self.a]
    }

    fn invert(self) -> Self {
        Self {
            v: self.v.invert(),
            a: self.a.invert(),
        }
    }
}

impl<F: PixelFormat> FromPixel<Luma<F>> for LumaAlpha<F> {
    fn from_pixel(pixel: Luma<F>) -> Self {
        LumaAlpha {
            v: pixel.v,
            a: F::WHITE,
        }
    }
}

impl<F: PixelFormat> FromPixel<LumaAlpha<F>> for LumaAlpha<F> {
    fn from_pixel(pixel: LumaAlpha<F>) -> Self {
        pixel
    }
}

impl<F: PixelFormat> FromPixel<Rgb<F>> for LumaAlpha<F> {
    fn from_pixel(pixel: Rgb<F>) -> Self {
        Self::from_pixel(Luma::<F>::from_pixel(pixel))
    }
}

impl<F: PixelFormat> FromPixel<Rgba<F>> for LumaAlpha<F> {
    fn from_pixel(pixel: Rgba<F>) -> Self {
        Self::from_pixel(Luma::<F>::from_pixel(pixel))
    }
}
