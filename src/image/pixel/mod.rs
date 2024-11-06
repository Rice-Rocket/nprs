use luma::Luma;
use luma_alpha::LumaAlpha;
use rgb::Rgb;
use rgba::Rgba;

use super::format::PixelFormat;

pub mod luma;
pub mod luma_alpha;
pub mod rgb;
pub mod rgba;

pub trait Pixel<const CHANNELS: usize>:
    FromPixel<Luma<Self::Format>>
    + FromPixel<LumaAlpha<Self::Format>>
    + FromPixel<Rgb<Self::Format>>
    + FromPixel<Rgba<Self::Format>>
{
    type Format: PixelFormat;

    fn from_channels(channels: [Self::Format; CHANNELS]) -> Self;
}

pub trait FromPixel<T> {
    fn from_pixel(pixel: T) -> Self;
}
