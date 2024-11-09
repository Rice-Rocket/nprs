use crate::image::format::PixelFormat;

use super::{luma::Luma, luma_alpha::LumaAlpha, rgba::Rgba, FromPixel, Pixel};

#[derive(Clone, Copy, Debug)]
pub struct Rgb<F: PixelFormat> {
    pub r: F,
    pub g: F,
    pub b: F,
}

impl<F: PixelFormat> Pixel<3> for Rgb<F> {
    type Format = F;

    const BLACK: Self = Self { r: F::BLACK, g: F::BLACK, b: F::BLACK };

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
