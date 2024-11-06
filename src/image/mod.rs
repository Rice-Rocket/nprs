use std::{fs::File, io, path::Path};

use format::PixelFormat;
use glam::UVec2;
use pixel::{luma::Luma, luma_alpha::LumaAlpha, rgb::Rgb, rgba::Rgba, Pixel};

pub mod pixel;
pub mod format;

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

    pub fn read<S: AsRef<Path>>(path: S) -> Result<Image<CHANNELS, F, P>, io::Error> {
        let path = path.as_ref();

        match path.extension() {
            Some(ext) => if ext.eq("png") {
                Self::read_png(path)
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidInput, format!("unsupported image file extension '{}'", ext.to_str().unwrap())))
            },
            None => Err(io::Error::new(io::ErrorKind::InvalidInput, "image file has no extension")),
        }
    }

    fn read_png(path: &Path) -> Result<Image<CHANNELS, F, P>, io::Error> {
        let mut decoder = png::Decoder::new(File::open(path)?);

        if decoder.read_header_info().ok().map(|h| h.color_type) == Some(png::ColorType::Indexed) {
            decoder.set_transformations(png::Transformations::EXPAND);
        } else {
            decoder.set_transformations(png::Transformations::IDENTITY);
        }

        let mut reader = match decoder.read_info() {
            Ok(reader) => reader,
            Err(_) => panic!(),
        };

        let mut im_data = vec![0; reader.output_buffer_size()];
        let info = match reader.next_frame(&mut im_data) {
            Ok(info) => info,
            Err(_) => panic!(),
        };


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
}
