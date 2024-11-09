use std::ops::{Add, Div, Mul, Sub};

use super::luma::Luma;
use super::luma_alpha::LumaAlpha;
use super::rgb::Rgb;
use super::rgba::Rgba;
use super::PixelFormat;

macro_rules! impl_pixel_op {
    ($op_trait:ident, $op_fn:ident, $op:tt; $($ty:ident: $($v:ident),*);*) => {
        $(
            impl<F: PixelFormat> $op_trait<$ty<F>> for $ty<F> {
                type Output = $ty<F>;

                fn $op_fn(self, rhs: $ty<F>) -> Self::Output {
                    Self { $($v: self.$v $op rhs.$v),* }
                }
            }
        )*
    }
}

impl_pixel_op!(Add, add, +; Luma: v; LumaAlpha: v, a; Rgb: r, g, b; Rgba: r, g, b, a);
impl_pixel_op!(Sub, sub, -; Luma: v; LumaAlpha: v, a; Rgb: r, g, b; Rgba: r, g, b, a);
impl_pixel_op!(Mul, mul, *; Luma: v; LumaAlpha: v, a; Rgb: r, g, b; Rgba: r, g, b, a);
impl_pixel_op!(Div, div, /; Luma: v; LumaAlpha: v, a; Rgb: r, g, b; Rgba: r, g, b, a);

macro_rules! impl_pixel_scale_op {
    ($op_trait:ident, $op_fn:ident, $op:tt; $($ty:ident: $($v:ident),*);*) => {
        $(
            impl<F: PixelFormat> $op_trait<F> for $ty<F> {
                type Output = $ty<F>;

                fn $op_fn(self, rhs: F) -> Self::Output {
                    Self { $($v: self.$v $op rhs),* }
                }
            }
        )*
    }
}

impl_pixel_scale_op!(Mul, mul, *; Luma: v; LumaAlpha: v, a; Rgb: r, g, b; Rgba: r, g, b, a);
impl_pixel_scale_op!(Div, div, /; Luma: v; LumaAlpha: v, a; Rgb: r, g, b; Rgba: r, g, b, a);
