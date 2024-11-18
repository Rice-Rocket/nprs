use std::{collections::HashMap, marker::PhantomData};

use crate::{image::{pixel::rgba::Rgba, Image}, parser::interpreter::{FromParsedValue, ParseValueError, ParsedValue}, render_graph::{ANY_IMAGE, MAIN_IMAGE}};

use super::{Pass, SubPass};

/// A pass that computes the luminance of each pixel on the `target` image.
pub struct Luminance {
    method: LuminanceMethod,
}

impl Luminance {
    pub const NAME: &'static str = "luminance";
    
    pub fn new(method: LuminanceMethod) -> Self {
        Self { method }
    }
}

impl FromParsedValue for Luminance {
    fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
        let type_name = value.type_name();
        let Some((name, fields)) = value.struct_properties() else {
            return Err(ParseValueError::WrongType(String::from("struct, tuple struct or unit struct"), type_name));
        };

        let mut method: Option<LuminanceMethod> = None;

        for (param, value) in fields.into_iter() {
            match param.as_str() {
                "method" => {
                    if method.is_some() {
                        return Err(ParseValueError::DuplicateField(String::from("method")));
                    }

                    method = Some(LuminanceMethod::from_parsed_value(*value)?)
                },
                _ => return Err(ParseValueError::UnknownField(param.to_string())),
            }
        }

        let Some(method) = method else {
            return Err(ParseValueError::MissingField(String::from("method")));
        };

        Ok(Self {
            method,
        })
    }
}

impl Pass for Luminance {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];

        target.for_each_with_positions(|pixel, pos| {
            let main_pixel = source.load(pos);
            let l = self.method.luminance(main_pixel.r, main_pixel.g, main_pixel.b);
            pixel.r = l;
        });
    }
}

impl SubPass for Luminance {
    fn apply_subpass(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        target.for_each(|pixel| {
            let l = self.method.luminance(pixel.r, pixel.g, pixel.b);
            pixel.r = l;
        })
    }
}

#[derive(Clone, Copy)]
pub enum LuminanceMethod {
    Standard,
    FastPerceived,
    Perceived,
}

impl LuminanceMethod {
    pub fn luminance(&self, r: f32, g: f32, b: f32) -> f32 {
        match self {
            LuminanceMethod::Standard => 0.2126 * r + 0.7152 * g + 0.0722 * b,
            LuminanceMethod::FastPerceived => 0.299 * r + 0.587 * g + 0.114 * b,
            LuminanceMethod::Perceived => f32::sqrt(0.299 * r * r + 0.587 * g * g + 0.114 * b * b),
        }
    }
}

impl FromParsedValue for LuminanceMethod {
    fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
        let type_name = value.type_name();
        let Some((name, fields)) = value.struct_properties() else {
            return Err(ParseValueError::WrongType(String::from("struct, tuple struct or unit struct"), type_name));
        };

        Ok(match name.as_str() {
            "Standard" => LuminanceMethod::Standard,
            "FastPerceived" => LuminanceMethod::FastPerceived,
            "Perceived" => LuminanceMethod::Perceived,
            _ => return Err(ParseValueError::UnknownVariant(name)),
        })
    }
}
