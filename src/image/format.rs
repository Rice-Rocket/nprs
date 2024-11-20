use std::ops::{Add, Div, Mul, Sub};

use half::f16;

use crate::parser::FromParsedValue;

pub trait PixelFormat: 
    Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + FromParsedValue
    + Send + Sync
    + Sized + Clone + Copy
{
    const BLACK: Self;
    const WHITE: Self;

    fn bytes() -> u8;

    fn from_bytes(bytes: &[u8]) -> Self;

    fn to_bytes(self) -> Vec<u8>;

    fn from_scaled_float(v: f32) -> Self;

    fn to_scaled_float(self) -> f32;

    fn invert(self) -> Self;
}

impl PixelFormat for u8 {
    const BLACK: Self = 0;
    const WHITE: Self = 255;

    fn bytes() -> u8 {
        1
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }

    fn to_bytes(self) -> Vec<u8> {
        vec![self]
    }

    fn from_scaled_float(v: f32) -> Self {
        (v * 255.0).clamp(0.0, 255.0) as u8
    }

    fn to_scaled_float(self) -> f32 {
        (self as f32) / 255.0
    }

    fn invert(self) -> Self {
        255 - self
    }
}

impl PixelFormat for f16 {
    const BLACK: Self = f16::from_f32_const(0.0);
    const WHITE: Self = f16::from_f32_const(1.0);

    fn bytes() -> u8 {
        2
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => f16::from_f32(bytes[0].to_scaled_float()),
            2 => f16::from_le_bytes([bytes[0], bytes[1]]),
            _ => panic!(),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        ((self.to_f32().clamp(0.0, 1.0) * u16::MAX as f32) as u16).to_be_bytes().to_vec()
    }

    fn from_scaled_float(v: f32) -> Self {
        f16::from_f32(v)
    }

    fn to_scaled_float(self) -> f32 {
        self.to_f32()
    }

    fn invert(self) -> Self {
        f16::from_f32(1.0) - self
    }
}

impl PixelFormat for f32 {
    const BLACK: Self = 0.0;
    const WHITE: Self = 1.0;

    fn bytes() -> u8 {
        4
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => (bytes[0] as f32) / 255.0,
            2 => f16::from_le_bytes([bytes[0], bytes[1]]).to_f32(),
            4 => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            _ => panic!(),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_scaled_float(v: f32) -> Self {
        v
    }

    fn to_scaled_float(self) -> f32 {
        self
    }

    fn invert(self) -> Self {
        1.0 - self
    }
}
