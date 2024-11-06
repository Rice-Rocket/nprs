pub trait Pixel<const CHANNELS: usize> {
    type Format: PixelFormat;

    fn from_channels(channels: [Self::Format; CHANNELS]) -> Self;
}

pub trait PixelFormat: Sized + Clone + Copy {
    fn from_bytes(bytes: &[u8]) -> Self;
}

pub struct Luma<F: PixelFormat>(F);

impl<F: PixelFormat> Pixel<1> for Luma<F> {
    type Format = F;

    fn from_channels(channels: [Self::Format; 1]) -> Self {
        Luma(channels[0])
    }
}

pub struct Rgba<F: PixelFormat> {
    pub r: F,
    pub g: F,
    pub b: F,
    pub a: F,
}

impl<F: PixelFormat> Pixel<4> for Rgba<F> {
    type Format = F;

    fn from_channels(channels: [Self::Format; 4]) -> Self {
        Rgba {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            a: channels[3],
        }
    }
}

impl PixelFormat for u8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}
