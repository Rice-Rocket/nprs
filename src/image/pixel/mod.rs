use std::ops::{Add, Div, Mul, Sub};

use luma::Luma;
use luma_alpha::LumaAlpha;
use nprs_derive::FromParsedValue;
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
    + Send + Sync
    + Clone + Copy
{
    type Format: PixelFormat;
    
    const BLACK: Self;
    const WHITE: Self;

    fn from_channels(channels: [Self::Format; CHANNELS]) -> Self;

    fn channels(&self) -> [Self::Format; CHANNELS];

    fn invert(self) -> Self;
}

pub trait FromPixel<T> {
    fn from_pixel(pixel: T) -> Self;
}

#[derive(FromParsedValue)]
pub enum Color {
    Luma(f32),
    LumaU8(u8),
    Rg(f32, f32),
    RgU8(u8, u8),
    Rgb(f32, f32, f32),
    RgbU8(u8, u8, u8),
    Rgba(f32, f32, f32, f32),
    RgbaU8(u8, u8, u8, u8),
}
