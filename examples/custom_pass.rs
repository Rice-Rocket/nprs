use nprs::{pixel::*, render_graph::ANY_IMAGE, FromParsedValue, Image, ParsePass, Pass};

// The `ParsePass` and `FromParsedValue` macros make your pass available in render graph descriptor
// files (.nprs files).
#[derive(ParsePass, FromParsedValue)]
struct Desaturate;

impl Pass for Desaturate {
    fn name(&self) -> &'static str {
        // The `PASS_NAME` constant is defined when `ParsePass` is derived.
        Self::PASS_NAME
    }

    // The types of passes this pass will run after.
    // This custom pass can run on any image. 
    // Some passes may only run on certain images (like luminance) which can be declared here.
    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        // Get the source image. Note that `aux_images` come from dependencies defined in the
        // render graph descriptor (.nprs) file.
        let source = aux_images[0];

        // Iterate over every pixel in the image.
        target.for_each_with_positions(|pixel, pos| {
            // Get the pixel in the source image.
            let source_pixel = source.load(pos);

            // Compute the pixel's value.
            let value = f32::max(source_pixel.r, f32::max(source_pixel.g, source_pixel.b));

            // Set the pixel's new color on the target image.
            pixel.r = value;
            pixel.g = value;
            pixel.b = value;
            pixel.a = 1.0;
        })
    }
}

fn main() {
    match nprs::run_cli() {
        Ok(_) => (),
        Err(err) => println!("error: {}", err),
    }
}
