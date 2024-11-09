use std::ops::{Add, Div, Mul, Sub};

use luma::Luma;
use luma_alpha::LumaAlpha;
use rgb::Rgb;
use rgba::Rgba;

use super::format::PixelFormat;

pub mod luma;
pub mod luma_alpha;
pub mod rgb;
pub mod rgba;
mod ops;

pub trait Pixel<const CHANNELS: usize>:
    FromPixel<Luma<Self::Format>>
    + FromPixel<LumaAlpha<Self::Format>>
    + FromPixel<Rgb<Self::Format>>
    + FromPixel<Rgba<Self::Format>>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Mul<Self::Format, Output = Self>
    + Div<Self::Format, Output = Self>
    + Clone + Copy
{
    type Format: PixelFormat;
    
    const BLACK: Self;

    fn from_channels(channels: [Self::Format; CHANNELS]) -> Self;

    fn channels(&self) -> [Self::Format; CHANNELS];
}

pub trait FromPixel<T> {
    fn from_pixel(pixel: T) -> Self;
}
