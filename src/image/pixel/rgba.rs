use crate::image::format::PixelFormat;

use super::{luma::Luma, luma_alpha::LumaAlpha, rgb::Rgb, FromPixel, Pixel};

#[derive(Clone, Copy, Debug)]
pub struct Rgba<F: PixelFormat> {
    pub r: F,
    pub g: F,
    pub b: F,
    pub a: F,
}

impl<F: PixelFormat> Rgba<F> {
    #[inline]
    pub fn new(r: F, g: F, b: F, a: F) -> Rgba<F> {
        Rgba::<F> { r, g, b, a }
    }

    #[inline]
    pub fn splat(v: F) -> Rgba<F> {
        Rgba::<F> { r: v, g: v, b: v, a: F::WHITE }
    }

    #[inline]
    pub fn splat_with_alpha(v: F) -> Rgba<F> {
        Rgba::<F> { r: v, g: v, b: v, a: v }
    }
}

impl<F: PixelFormat> Pixel<4> for Rgba<F> {
    type Format = F;

    const BLACK: Self = Self { r: F::BLACK, g: F::BLACK, b: F::BLACK, a: F::BLACK };

    fn from_channels(channels: [Self::Format; 4]) -> Self {
        Rgba {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            a: channels[3],
        }
    }

    fn channels(&self) -> [Self::Format; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl<F: PixelFormat> FromPixel<Luma<F>> for Rgba<F> {
    fn from_pixel(pixel: Luma<F>) -> Self {
        Self::from_pixel(Rgb::<F>::from_pixel(pixel))
    }
}

impl<F: PixelFormat> FromPixel<LumaAlpha<F>> for Rgba<F> {
    fn from_pixel(pixel: LumaAlpha<F>) -> Self {
        Self::from_pixel(Rgb::<F>::from_pixel(pixel))
    }
}

impl<F: PixelFormat> FromPixel<Rgb<F>> for Rgba<F> {
    fn from_pixel(pixel: Rgb<F>) -> Self {
        Self {
            r: pixel.r,
            g: pixel.g,
            b: pixel.b,
            a: F::WHITE,
        }
    }
}

impl<F: PixelFormat> FromPixel<Rgba<F>> for Rgba<F> {
    fn from_pixel(pixel: Rgba<F>) -> Self {
        pixel
    }
}
