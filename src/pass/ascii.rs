use std::f32::consts::PI;

use glam::UVec2;
use nprs_derive::{FromParsedValue, ParsePass};
use rayon::iter::ParallelIterator;

use crate::{image::{format::PixelFormat, pixel::Pixel}, pass::tfm::{sobel::Sobel, TangentFlowMap}, pixel::{Luma, LumaAlpha, Rgba}, render_graph::ANY_IMAGE, Image, Pass, SubPass};

use super::{blend::{Blend, BlendMode}, difference_of_gaussians::simple::BasicDifferenceOfGaussians, luminance::{Luminance, LuminanceMethod}};

#[derive(ParsePass, FromParsedValue)]
pub struct Ascii {
    fill_im: Image<4, f32, Rgba<f32>>,
    edge_im: Image<4, f32, Rgba<f32>>,
    edge_threshold: f32,
    edge_count_threshold: u8,
    dog: BasicDifferenceOfGaussians,
    #[nprs(default = 8)]
    char_size: u32,
    #[nprs(default = 10)]
    num_chars: u32,
    #[nprs(default = LuminanceMethod::Standard)]
    lum: LuminanceMethod,
    #[nprs(default = true)]
    fill: bool,
    #[nprs(default = true)]
    edges: bool,
}

impl Pass for Ascii {
    fn name(&self) -> &'static str {
        Self::PASS_NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let mut sobel = Image::<4, f32, Rgba<f32>>::new_fill(
            target.resolution(),
            Rgba::BLACK,
        );
        
        let lum = Luminance::new(self.lum);
        lum.apply(&mut sobel, &[aux_images[0]]);

        let mut dog = sobel.clone();

        self.dog.apply_subpass(&mut dog, &[]);
        Sobel.apply_subpass(&mut sobel, &[]);
        Blend::from_mode(BlendMode::Multiply).apply_subpass(&mut sobel, &[&dog]);

        drop(dog);

        let source = aux_images[0];
        
        let downscaled_res = (source.resolution().as_vec2() / self.char_size as f32).ceil().as_uvec2();

        let mut downscaled_lum = Image::<1, f32, Luma<f32>>::new_fill(
            downscaled_res,
            Luma::BLACK,
        );

        let mut downscaled_tfm_histogram = Image::<4, u8, Rgba<u8>>::new_fill(
            downscaled_res,
            Rgba::BLACK,
        );

        let mut downscaled_edges = Image::<1, u8, Luma<u8>>::new_fill(
            downscaled_res,
            Luma::BLACK,
        );

        source.iter_pixels_with_positions().for_each(|(pixel, pos)| {
            let pos_downscaled = pos / self.char_size;
            let mut col = downscaled_lum.get_mut(pos_downscaled);
            col.v += self.lum.luminance(pixel.r, pixel.g, pixel.b) / (self.char_size * self.char_size) as f32;
        });

        sobel.iter_pixels_with_positions().for_each(|(pixel, pos)| {
            let pos_downscaled = pos / self.char_size;
            let mut col = downscaled_tfm_histogram.get_mut(pos_downscaled);
            
            let theta = f32::atan2(pixel.r, pixel.g);
            let abs_theta = theta.abs() / PI;

            if (0.0..0.05).contains(&abs_theta)
                || (0.9 < abs_theta && abs_theta <= 1.0)
            {
                // vertical
                col.r += 1;
            } else if 0.45 < abs_theta && abs_theta < 0.55 {
                // horizontal
                col.g += 1;
            } else if 0.05 < abs_theta && abs_theta < 0.45 {
                // diagonal 1
                if theta.signum() > 0.0 {
                    col.b += 1;
                } else {
                    col.a += 1;
                }
            } else if 0.55 < abs_theta && abs_theta < 0.9 {
                // diagonal 2
                if theta.signum() > 0.0 {
                    col.a += 1;
                } else {
                    col.b += 1;
                }
            }

            let len_d = pixel.b;

            if len_d > self.edge_threshold {
                downscaled_edges.store(pos_downscaled, Luma::WHITE)
            }
        });

        downscaled_edges.for_each_with_positions(|pixel, pos| {
            let edge_counts = downscaled_tfm_histogram.load(pos);
            let max_idx = edge_counts.channels().into_iter().enumerate().max_by_key(|(_, count)| *count).unwrap().0;
            let max_count = edge_counts.channels().into_iter().max().unwrap();

            if pixel.v == <u8 as PixelFormat>::WHITE && max_count >= self.edge_count_threshold {
                pixel.v = max_idx as u8 + 1;
            } else {
                pixel.v = 0;
            }
        });

        target.for_each_with_positions(|pixel, pos| {
            let pos_downscaled = pos / self.char_size;
            let mut pos_char = pos % self.char_size;
            let edge = downscaled_edges.load(pos_downscaled);

            if edge.v != 0 {
                pos_char.x += edge.v as u32 * self.char_size;

                let edge_char = self.edge_im.load(pos_char);

                pixel.r = edge_char.r;
                pixel.g = edge_char.g;
                pixel.b = edge_char.b;
                pixel.a = 1.0;
            } else {
                let lum = downscaled_lum.load(pos_downscaled);

                let quantized_lum = (lum.v * self.num_chars as f32).floor() as u32;
                pos_char.x += quantized_lum * self.char_size;
                pos_char.x = pos_char.x.min(self.char_size * self.num_chars - 1);

                let fill_char = self.fill_im.load(pos_char);

                pixel.r = fill_char.r;
                pixel.g = fill_char.g;
                pixel.b = fill_char.b;
                pixel.a = 1.0;
            }
        });
    }
}
