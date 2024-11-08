use crate::image::format::PixelFormat;

use super::{luma_alpha::LumaAlpha, rgb::Rgb, rgba::Rgba, FromPixel, Pixel};

#[derive(Clone, Copy)]
pub struct Luma<F: PixelFormat>(pub F);

impl<F: PixelFormat> Pixel<1> for Luma<F> {
    type Format = F;

    fn from_channels(channels: [Self::Format; 1]) -> Self {
        Luma(channels[0])
    }

    fn channels(&self) -> [Self::Format; 1] {
        [self.0]
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
        Self(F::from_scaled_float(l))
    }
}

impl<F: PixelFormat> FromPixel<Rgb<F>> for Luma<F> {
    fn from_pixel(pixel: Rgb<F>) -> Self {
        let (r, g, b) = (pixel.r.to_scaled_float(), pixel.g.to_scaled_float(), pixel.b.to_scaled_float());
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        Self(F::from_scaled_float(l))
    }
}

impl<F: PixelFormat> FromPixel<Rgba<F>> for Luma<F> {
    fn from_pixel(pixel: Rgba<F>) -> Self {
        let (r, g, b) = (pixel.r.to_scaled_float(), pixel.g.to_scaled_float(), pixel.b.to_scaled_float());
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        Self(F::from_scaled_float(l * pixel.a.to_scaled_float()))
    }
}
