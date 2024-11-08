use half::f16;
use image::{pixel::luma::Luma, Image};

mod pass;
mod image;
mod render_graph;

fn main() {
    let image = Image::<1, f32, Luma<f32>>::read("images/pagoda.png").unwrap();

    let image_u8 = image.to_format::<f16, Luma<f16>>();
    image_u8.write("output/pagoda.png").unwrap();
}
