use image::{pixel::luma::Luma, Image};

mod image;

fn main() {
    let image = Image::<1, f32, Luma<f32>>::read("images/pagoda.png").unwrap();
}
