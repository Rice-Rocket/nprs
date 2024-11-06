use image::Image;
use pixel::{Luma, Rgba};

mod image;
mod pixel;

fn main() {
    let image = Image::<4, u8, Rgba<u8>>::read("images/pagoda.png").unwrap();
}
