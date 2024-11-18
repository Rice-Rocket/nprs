use glam::Vec3;

use crate::image::format::PixelFormat;

use super::{luma::Luma, luma_alpha::LumaAlpha, rgba::Rgba, FromPixel, Pixel};

#[derive(Clone, Copy, Debug)]
pub struct Rgb<F: PixelFormat> {
    pub r: F,
    pub g: F,
    pub b: F,
}

impl<F: PixelFormat> Rgb<F> {
    #[inline]
    pub fn new(r: F, g: F, b: F) -> Rgb<F> {
        Rgb::<F> { r, g, b }
    }

    #[inline]
    pub fn splat(v: F) -> Rgb<F> {
        Self::new(v, v, v)
    }

    #[inline]
    pub fn dot(self, other: Self) -> F {
        self.r * other.r + self.g * other.g + self.b * other.b
    }
}

impl Rgb<f32> {
    #[inline]
    pub fn saturate(self) -> Self {
        Self::new(self.r.clamp(0.0, 1.0), self.g.clamp(0.0, 1.0), self.b.clamp(0.0, 1.0))
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite()
    }
}

impl<F: PixelFormat> Pixel<3> for Rgb<F> {
    type Format = F;

    const BLACK: Self = Self { r: F::BLACK, g: F::BLACK, b: F::BLACK };
    const WHITE: Self = Self { r: F::WHITE, g: F::WHITE, b: F::WHITE };

    fn from_channels(channels: [Self::Format; 3]) -> Self {
        Rgb {
            r: channels[0],
            g: channels[1],
            b: channels[2],
        }
    }
    
    fn channels(&self) -> [Self::Format; 3] {
        [self.r, self.g, self.b]
    }

    fn invert(self) -> Self {
        Self {
            r: self.r.invert(),
            g: self.g.invert(),
            b: self.b.invert(),
        }
    }
}

impl From<Rgb<f32>> for Vec3 {
    fn from(val: Rgb<f32>) -> Self {
        Vec3::new(val.r, val.g, val.b)
    }
}

impl<F: PixelFormat> FromPixel<Luma<F>> for Rgb<F> {
    fn from_pixel(pixel: Luma<F>) -> Self {
        Self {
            r: pixel.v,
            g: pixel.v,
            b: pixel.v,
        }
    }
}

impl<F: PixelFormat> FromPixel<LumaAlpha<F>> for Rgb<F> {
    fn from_pixel(pixel: LumaAlpha<F>) -> Self {
        Self::from_pixel(Luma::<F>::from_pixel(pixel))
    }
}

impl<F: PixelFormat> FromPixel<Rgb<F>> for Rgb<F> {
    fn from_pixel(pixel: Rgb<F>) -> Self {
        pixel
    }
}

impl<F: PixelFormat> FromPixel<Rgba<F>> for Rgb<F> {
    fn from_pixel(pixel: Rgba<F>) -> Self {
        let (r, g, b, a) = (pixel.r.to_scaled_float(), pixel.g.to_scaled_float(), pixel.b.to_scaled_float(), pixel.a.to_scaled_float());
        Self {
            r: F::from_scaled_float(r * a),
            g: F::from_scaled_float(g * a),
            b: F::from_scaled_float(b * a),
        }
    }
}
