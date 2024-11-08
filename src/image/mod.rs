use std::{fs::File, path::Path};

use format::PixelFormat;
use glam::UVec2;
use pixel::{luma::Luma, luma_alpha::LumaAlpha, rgb::Rgb, rgba::Rgba, Pixel};
use thiserror::Error;

pub mod pixel;
pub mod format;

#[derive(Clone)]
pub struct Image<const CHANNELS: usize, F, P>
where
    F: PixelFormat,
    P: Pixel<CHANNELS, Format = F>,
{
    pixels: Vec<P>,
    resolution: UVec2,
}

impl<const CHANNELS: usize, F: PixelFormat, P: Pixel<CHANNELS, Format = F>> Image<CHANNELS, F, P> {
    pub fn new(resolution: UVec2, pixels: Vec<P>) -> Image<CHANNELS, F, P> {
        Self {
            pixels,
            resolution,
        }
    }

    pub fn new_fill(resolution: UVec2, pixel: P) -> Image<CHANNELS, F, P> {
        Self {
            pixels: vec![pixel; (resolution.x * resolution.y) as usize],
            resolution,
        }
    }

    pub fn resolution(&self) -> UVec2 {
        self.resolution
    }

    pub fn map<Convert, ToPixel, ToFormat>(&self, f: Convert) -> Image<CHANNELS, ToFormat, ToPixel>
    where
        ToPixel: Pixel<CHANNELS, Format = ToFormat>,
        Convert: Fn(&P) -> ToPixel,
        ToFormat: PixelFormat,
    {
        Image::<CHANNELS, ToFormat, ToPixel>::new(self.resolution, self.pixels.iter().map(f).collect())
    }

    pub fn map_in_place<Convert>(&mut self, f: Convert)
    where
        Convert: Fn(&mut P)
    {
        self.pixels.iter_mut().for_each(f);
    }

    pub fn to_format<ToFormat: PixelFormat, ToPixel: Pixel<CHANNELS, Format = ToFormat>>(&self) -> Image<CHANNELS, ToFormat, ToPixel> {
        self.map(|pixel| {
            let channels: Result<[ToFormat; CHANNELS], _> = pixel.channels().into_iter()
                .map(|f| ToFormat::from_scaled_float(f.to_scaled_float()))
                .collect::<Vec<ToFormat>>()
                .try_into();

            match channels {
                Ok(v) => ToPixel::from_channels(v),
                Err(_) => ToPixel::from_channels([ToFormat::from_scaled_float(0.0); CHANNELS]),
            }
        })
    }

    pub fn read<S: AsRef<Path>>(path: S) -> Result<Image<CHANNELS, F, P>, ImageError> {
        let path = path.as_ref();

        match path.extension() {
            Some(ext) => if ext.eq("png") {
                Self::read_png(path)
            } else {
                Err(ImageError::InvalidExtension(ext.to_str().unwrap().to_string()))
            },
            None => Err(ImageError::NoExtension(path.to_str().unwrap().to_string()))
        }
    }

    fn read_png(path: &Path) -> Result<Image<CHANNELS, F, P>, ImageError> {
        let mut decoder = png::Decoder::new(File::open(path)?);

        if decoder.read_header_info().ok().map(|h| h.color_type) == Some(png::ColorType::Indexed) {
            decoder.set_transformations(png::Transformations::EXPAND);
        } else {
            decoder.set_transformations(png::Transformations::IDENTITY);
        }

        let mut reader = decoder.read_info()?;

        let mut im_data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut im_data)?;


        let chunk_size = match info.bit_depth {
            png::BitDepth::Eight => 1,
            png::BitDepth::Sixteen => 2,
            _ => todo!(),
        };

        let formatted_im_data: Vec<F> = im_data.chunks_exact(chunk_size).map(|bytes| F::from_bytes(bytes)).collect();

        let pixels = match info.color_type {
            png::ColorType::Grayscale => {
                formatted_im_data
                    .into_iter()
                    .map(|channels| P::from_pixel(Luma::<F>::from_channels([channels])))
                    .collect()
            },
            png::ColorType::GrayscaleAlpha => {
                formatted_im_data
                    .chunks_exact(2)
                    .flat_map(<[F; 2]>::try_from)
                    .map(|channels| P::from_pixel(LumaAlpha::<F>::from_channels(channels)))
                    .collect()
            },
            png::ColorType::Rgb => {
                formatted_im_data
                    .chunks_exact(3)
                    .flat_map(<[F; 3]>::try_from)
                    .map(|channels| P::from_pixel(Rgb::<F>::from_channels(channels)))
                    .collect()
            },
            png::ColorType::Rgba => {
                formatted_im_data
                    .chunks_exact(4)
                    .flat_map(<[F; 4]>::try_from)
                    .map(|channels| P::from_pixel(Rgba::<F>::from_channels(channels)))
                    .collect()
            },
            png::ColorType::Indexed => unreachable!(),
        };
        
        Ok(Self::new(
            UVec2::new(info.width, info.height),
            pixels,
        ))
    }

    pub fn write<S: AsRef<Path>>(&self, path: S) -> Result<(), ImageError> {
        let path = path.as_ref();

        match path.extension() {
            Some(ext) => if ext.eq("png") {
                self.write_png(path)
            } else {
                Err(ImageError::InvalidExtension(ext.to_str().unwrap().to_string()))
            },
            None => Err(ImageError::NoExtension(path.to_str().unwrap().to_string()))
        }
    }

    fn write_png(&self, path: &Path) -> Result<(), ImageError> {
        let file = File::create(path)?;
        let buf_writer = &mut std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(buf_writer, self.resolution.x, self.resolution.y);

        match CHANNELS {
            1 => encoder.set_color(png::ColorType::Grayscale),
            2 => encoder.set_color(png::ColorType::GrayscaleAlpha),
            3 => encoder.set_color(png::ColorType::Rgb),
            4 => encoder.set_color(png::ColorType::Rgba),
            _ => return Err(ImageError::BadChannelCount(CHANNELS, String::from("png")))
        }

        match F::bytes() {
            1 => encoder.set_depth(png::BitDepth::Eight),
            2 => encoder.set_depth(png::BitDepth::Sixteen),
            _ => return Err(ImageError::BadBitDepth(F::bytes(), String::from("png"))),
        }

        let data: Vec<_> = self.pixels.iter()
            .flat_map(|p| p.channels())
            .flat_map(|v| v.to_bytes())
            .collect();

        let mut writer = encoder.write_header()?;
        Ok(writer.write_image_data(&data)?)
    }
}

#[derive(Error, Debug)]
pub enum ImageError {
    /// Invalid extension for image file.
    #[error("invalid image file extension `{0}`.")]
    InvalidExtension(String),
    /// No extension for image file.
    #[error("expected some extension for image file `{0}`.")]
    NoExtension(String),
    /// Unsupported channel count for a given extension.
    #[error("{0} color channels not supported by {1}.")]
    BadChannelCount(usize, String),
    /// Unsupported 
    #[error("bit depth {0} not supported by {1}.")]
    BadBitDepth(u8, String),
    /// An IO Error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A PNG Decoding Error.
    #[error(transparent)]
    PngDecoding(#[from] png::DecodingError),
    /// A PNG Encoding Error.
    #[error(transparent)]
    PngEncoding(#[from] png::EncodingError)
}
