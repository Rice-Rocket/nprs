use image::{pixel::luma::Luma, Image};

mod image;

fn main() {
    let image = Image::<1, f32, Luma<f32>>::read("images/pagoda.png").unwrap();

    let image_u8 = image.to_format::<u8, Luma<u8>>();
    image_u8.write("output/pagoda.png").unwrap();
}
