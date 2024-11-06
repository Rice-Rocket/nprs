use half::f16;

pub trait PixelFormat: Sized + Clone + Copy {
    fn from_bytes(bytes: &[u8]) -> Self;

    fn from_scaled_float(v: f32) -> Self;

    fn to_scaled_float(self) -> f32;
}

impl PixelFormat for u8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }

    fn from_scaled_float(v: f32) -> Self {
        (v * 255.0).clamp(0.0, 255.0) as u8
    }

    fn to_scaled_float(self) -> f32 {
        (self as f32) / 255.0
    }
}

impl PixelFormat for f16 {
    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => f16::from_f32(bytes[0].to_scaled_float()),
            2 => f16::from_le_bytes([bytes[0], bytes[1]]),
            _ => panic!(),
        }
    }

    fn from_scaled_float(v: f32) -> Self {
        f16::from_f32(v)
    }

    fn to_scaled_float(self) -> f32 {
        self.to_f32()
    }
}

impl PixelFormat for f32 {
    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => (bytes[0] as f32) / 255.0,
            2 => f16::from_le_bytes([bytes[0], bytes[1]]).to_f32(),
            4 => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            _ => panic!(),
        }
    }

    fn from_scaled_float(v: f32) -> Self {
        v
    }

    fn to_scaled_float(self) -> f32 {
        self
    }
}
