use nprs_derive::FromParsedValue;

use crate::image::format::PixelFormat;

use super::{luma::Luma, luma_alpha::LumaAlpha, rgb::Rgb, FromPixel, Pixel};

#[derive(FromParsedValue, Clone, Copy, Debug, PartialEq)]
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
        Self::new(v, v, v, F::WHITE)
    }

    #[inline]
    pub fn splat_with_alpha(v: F) -> Rgba<F> {
        Self::new(v, v, v, v)
    }

    #[inline]
    pub fn rgb(self) -> Rgb<F> {
        Rgb::new(self.r, self.g, self.b)
    }

    #[inline]
    pub fn dot(self, other: Self) -> F {
        self.r * other.r + self.g * other.g + self.b * other.b + self.a * other.a
    }
}

impl Rgba<f32> {
    #[inline]
    pub fn saturate(self) -> Self {
        Self::new(self.r.clamp(0.0, 1.0), self.g.clamp(0.0, 1.0), self.b.clamp(0.0, 1.0), self.a.clamp(0.0, 1.0))
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite()
    }
}

impl<F: PixelFormat> Pixel<4> for Rgba<F> {
    type Format = F;

    const BLACK: Self = Self { r: F::BLACK, g: F::BLACK, b: F::BLACK, a: F::BLACK };
    const WHITE: Self = Self { r: F::WHITE, g: F::WHITE, b: F::WHITE, a: F::WHITE };

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

    fn invert(self) -> Self {
        Self {
            r: self.r.invert(),
            g: self.g.invert(),
            b: self.b.invert(),
            a: self.a.invert(),
        }
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
