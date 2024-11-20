use glam::{DVec2, DVec3, DVec4, I16Vec2, I16Vec3, I16Vec4, I64Vec2, I64Vec3, I64Vec4, I8Vec2, I8Vec3, I8Vec4, IVec2, IVec3, IVec4, U16Vec2, U16Vec3, U16Vec4, U64Vec2, U64Vec3, U64Vec4, U8Vec2, U8Vec3, U8Vec4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use half::f16;

use super::{interpreter::ParsedValue, FromParsedValue, ParseValueError};

macro_rules! impl_parse_num {
    ($($ty:ident, $value:ident, $expected:literal);* $(;)*) => {
        $(
            impl FromParsedValue for $ty {
                fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
                    if let ParsedValue::$value(v) = value {
                        Ok(v as $ty)
                    } else {
                        Err(ParseValueError::WrongType(String::from($expected), value.type_name()))
                    }
                }
            }
        )*
    }
}

impl_parse_num!(
    u8, Int, "int";
    u16, Int, "int";
    u32, Int, "int";
    u64, Int, "int";
    u128, Int, "int";
    usize, Int, "int";
    i8, Int, "int";
    i16, Int, "int";
    i32, Int, "int";
    i64, Int, "int";
    i128, Int, "int";
    isize, Int, "int";
    f32, Float, "float";
    f64, Float, "float";
);

impl FromParsedValue for f16 {
    fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
        if let ParsedValue::Float(v) = value {
            Ok(f16::from_f32(v))
        } else {
            Err(ParseValueError::WrongType(String::from("float"), value.type_name()))
        }
    }
}

impl FromParsedValue for bool {
    fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
        if let ParsedValue::Bool(v) = value {
            Ok(v)
        } else {
            Err(ParseValueError::WrongType(String::from("bool"), value.type_name()))
        }
    }
}

impl<T: FromParsedValue> FromParsedValue for Option<T> {
    fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
        let type_name = value.type_name();
        let Some((name, fields)) = value.struct_properties() else {
            return Err(ParseValueError::WrongType(String::from("`Some` or `None`"), type_name));
        };

        match name.as_str() {
            "Some" => {
                let mut v: Option<T> = None;

                for (param, value) in fields.into_iter() {
                    match param.as_str() {
                        "0" => {
                            if v.is_some() {
                                return Err(ParseValueError::DuplicateField(String::from("0")));
                            };

                            v = Some(T::from_parsed_value(*value)?);
                        },
                        _ => return Err(ParseValueError::UnknownField(param.to_string())),
                    }
                }

                let Some(v) = v else {
                    return Err(ParseValueError::MissingField(String::from("0")));
                };

                Ok(Some(v))
            },
            "None" => Ok(None),
            _ => Err(ParseValueError::UnknownVariant(name)),
        }
    }
}

macro_rules! impl_parse_vec {
    ($($ty:ident, $prim:ident, $expected:literal: $([$var:ident, $str:literal, $idx:literal]),*);* $(;)*) => {
        $(
            impl FromParsedValue for $ty {
                fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError> {
                    let type_name = value.type_name();
                    let Some((name, fields)) = value.struct_properties() else {
                        return Err(ParseValueError::WrongType(String::from($expected), type_name));
                    };

                    $(
                        let mut $var: Option<$prim> = None;
                    )*

                    for (param, value) in fields.into_iter() {
                        match param.as_str() {
                            $(
                                $idx | $str => {
                                    if $var.is_some() {
                                        return Err(ParseValueError::DuplicateField(String::from($str)));
                                    }

                                    $var = Some($prim::from_parsed_value(*value)?);
                                },
                            )*
                            _ => return Err(ParseValueError::UnknownField(param.to_string())),
                        }
                    }

                    $(
                        let Some($var) = $var else {
                            return Err(ParseValueError::MissingField(String::from($str)));
                        };
                    )*

                    Ok($ty::new($($var,)*))
                }
            }
        )*
    }
}

impl_parse_vec!(
    Vec2, f32, "struct `Vec2`": [x, "x", "0"], [y, "y", "1"];
    Vec3, f32, "struct `Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    Vec4, f32, "struct `Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    DVec2, f64, "struct `Vec2`": [x, "x", "0"], [y, "y", "1"];
    DVec3, f64, "struct `Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    DVec4, f64, "struct `Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    I8Vec2, i8, "struct `I8Vec2`": [x, "x", "0"], [y, "y", "1"];
    I8Vec3, i8, "struct `I8Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    I8Vec4, i8, "struct `I8Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    I16Vec2, i16, "struct `I16Vec2`": [x, "x", "0"], [y, "y", "1"];
    I16Vec3, i16, "struct `I16Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    I16Vec4, i16, "struct `I16Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    IVec2, i32, "struct `IVec2`": [x, "x", "0"], [y, "y", "1"];
    IVec3, i32, "struct `IVec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    IVec4, i32, "struct `IVec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    I64Vec2, i64, "struct `I64Vec2`": [x, "x", "0"], [y, "y", "1"];
    I64Vec3, i64, "struct `I64Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    I64Vec4, i64, "struct `I64Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    U8Vec2, u8, "struct `U8Vec2`": [x, "x", "0"], [y, "y", "1"];
    U8Vec3, u8, "struct `U8Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    U8Vec4, u8, "struct `U8Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    U16Vec2, u16, "struct `U16Vec2`": [x, "x", "0"], [y, "y", "1"];
    U16Vec3, u16, "struct `U16Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    U16Vec4, u16, "struct `U16Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    UVec2, u32, "struct `UVec2`": [x, "x", "0"], [y, "y", "1"];
    UVec3, u32, "struct `UVec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    UVec4, u32, "struct `UVec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
    U64Vec2, u64, "struct `U64Vec2`": [x, "x", "0"], [y, "y", "1"];
    U64Vec3, u64, "struct `U64Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"];
    U64Vec4, u64, "struct `U64Vec3`": [x, "x", "0"], [y, "y", "1"], [z, "z", "2"], [w, "w", "3"];
);
