use std::{fs::File, path::Path};

use format::PixelFormat;
use glam::{IVec2, UVec2, Vec2};
use pixel::{luma::Luma, luma_alpha::LumaAlpha, rgb::Rgb, rgba::Rgba, Pixel};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use thiserror::Error;
use sampler::{Filter, Sampler, WrapMode2D};

pub mod pixel;
pub mod format;
pub mod sampler;

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

    pub fn sample(&self, uv: Vec2, sampler: Sampler) -> P {
        match sampler.filter {
            Filter::NearestNeighbor => {
                let p = (uv * self.resolution.as_vec2()).round().as_ivec2();
                self.load_wrapped(p, sampler.wrap_mode)
            },
            Filter::Linear => {
                let p = uv * self.resolution.as_vec2() - 0.5;
                let pi = p.floor().as_ivec2();
                let d = p - p.floor();

                let v00 = self.load_wrapped(pi, sampler.wrap_mode);
                let v10 = self.load_wrapped(pi + IVec2::new(1, 0), sampler.wrap_mode);
                let v01 = self.load_wrapped(pi + IVec2::new(0, 1), sampler.wrap_mode);
                let v11 = self.load_wrapped(pi + IVec2::new(1, 1), sampler.wrap_mode);

                v00 * F::from_scaled_float((1.0 - d.x) * (1.0 - d.y))
                    + v10 * F::from_scaled_float(d.x * (1.0 - d.y))
                    + v01 * F::from_scaled_float((1.0 - d.x) * d.y)
                    + v11 * F::from_scaled_float(d.x * d.y)
            },
        }
    }

    pub fn load_wrapped(&self, p: IVec2, wrap_mode: WrapMode2D) -> P {
        if let Some(p) = wrap_mode.remap(p, self.resolution.as_ivec2()) {
            self.pixels[(p.y * self.resolution.x + p.x) as usize]
        } else {
            P::BLACK
        }
    }

    pub fn load(&self, p: UVec2) -> P {
        assert!(p.x < self.resolution.x && p.y < self.resolution.y);
        self.pixels[(p.y * self.resolution.x + p.x) as usize]
    }

    pub fn store(&mut self, p: UVec2, c: P) {
        assert!(p.x < self.resolution.x && p.y < self.resolution.y);
        self.pixels[(p.y * self.resolution.x + p.x) as usize] = c;
    }

    pub fn store_wrapped(&mut self, p: IVec2, c: P) {
        if p.x < self.resolution.x as i32 && p.y < self.resolution.y as i32 && p.x >= 0 && p.y >= 0 {
            self.pixels[(p.y as u32 * self.resolution.x + p.x as u32) as usize] = c;
        }
    }

    /// Map a function over every pixel in the image, returning a new image.
    pub fn map<Convert, ToPixel, ToFormat>(&self, f: Convert) -> Image<CHANNELS, ToFormat, ToPixel>
    where
        ToPixel: Pixel<CHANNELS, Format = ToFormat>,
        Convert: Fn(&P) -> ToPixel + Send + Sync,
        ToFormat: PixelFormat,
    {
        Image::<CHANNELS, ToFormat, ToPixel>::new(self.resolution, self.pixels.par_iter().map(f).collect())
    }

    /// Map a function over every pixel in the image, given its position.
    pub fn map_with_positions<Convert, ToPixel, ToFormat>(&self, f: Convert) -> Image<CHANNELS, ToFormat, ToPixel>
    where
        ToPixel: Pixel<CHANNELS, Format = ToFormat>,
        Convert: Fn(&P, UVec2) -> ToPixel + Sync,
        ToFormat: PixelFormat,
    {
        Image::<CHANNELS, ToFormat, ToPixel>::new(
            self.resolution,
            self.pixels.par_iter()
                .enumerate()
                .map(|(i, p)| (p, UVec2::new(i as u32 % self.resolution.x, i as u32 / self.resolution.x)))
                .map(|(pixel, pos)| f(pixel, pos))
                .collect(),
        )
    }

    /// Apply a function to each pixel in the image.
    pub fn for_each<Convert>(&mut self, f: Convert)
    where
        Convert: Fn(&mut P) + Send + Sync
    {
        self.pixels.par_iter_mut().for_each(f);
    }

    /// Apply a function to each pixel in the image, given its position.
    pub fn for_each_with_positions<Convert>(&mut self, f: Convert)
    where
        Convert: Fn(&mut P, UVec2) + Sync
    {
        self.pixels
            .par_iter_mut()
            .enumerate()
            .map(|(i, p)| (p, UVec2::new(i as u32 % self.resolution.x, i as u32 / self.resolution.x)))
            .for_each(|(pixel, pos)| f(pixel, pos));
    }

    /// Apply a function to each pixel in the image, given its UV coordinates.
    pub fn for_each_with_uvs<Convert>(&mut self, f: Convert)
    where
        Convert: Fn(&mut P, Vec2) + Sync
    {
        self.pixels
            .par_iter_mut()
            .enumerate()
            .map(|(i, p)| (p, UVec2::new(i as u32 % self.resolution.x, i as u32 / self.resolution.x).as_vec2() / self.resolution.as_vec2()))
            .for_each(|(pixel, uv)| f(pixel, uv));
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

    // TODO: blend modes
    pub fn blit(&mut self, im: Image<CHANNELS, F, P>, position: UVec2) {
        for x in 0..im.resolution.x {
            for y in 0..im.resolution.y {
                let p = UVec2::new(x, y);
                let pi = position.as_ivec2() + p.as_ivec2() - im.resolution.as_ivec2() / 2;

                if pi.x < 0 || pi.x >= self.resolution.x as i32 || pi.y < 0 || pi.y >= self.resolution.y as i32 {
                    continue;
                }

                let pixel = im.load(p);
                self.store(pi.as_uvec2(), pixel);
            }
        }
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

impl<const CHANNELS: usize, P: Pixel<CHANNELS, Format = f32>> Image<CHANNELS, f32, P> {
    // TODO: FFT-based convolution algorithm
    pub fn convolve(&self, kernel: &[f32], kernel_size: UVec2) -> Image<CHANNELS, f32, P> {
        assert!(kernel.len() as u32 == kernel_size.x * kernel_size.y);

        let kernel_size = kernel_size.as_ivec2();
        let mut image = Image::<CHANNELS, f32, P>::new_fill(self.resolution, P::BLACK);

        self.map_with_positions(|pixel, pos| {
            let mut c = P::BLACK;

            for i in -(kernel_size.x / 2)..=(kernel_size.x / 2) {
                for j in -(kernel_size.y / 2)..=(kernel_size.y / 2) {
                    let p = IVec2::new(pos.x as i32 + i, pos.y as i32 + j);
                    let v = self.load_wrapped(p, WrapMode2D::CLAMP);
                    let w = kernel[((j + kernel_size.y / 2) * kernel_size.x + (i + kernel_size.x / 2)) as usize];
                    c = c + (v * w);
                }
            }

            c
        })
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
