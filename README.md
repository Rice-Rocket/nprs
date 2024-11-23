nprs
====

The nprs crate provides advanced utilities for creating and running non-photorealistic image effects. It comes with numerous built-in effects and its own language for defining custom effect pipelines.

## Basic Usage

Using only built-in effects and the default CLI:

```rust
fn main() {
    match nprs::run_cli() {
        Ok(_) => (),
        Err(err) => println!("error: {}", err),
    }
}
```

Run the program with:

```sh
cargo run --release -- --help
```

## The Nprs Language

The layout of render graphs are defined in .nprs files which are supplied to the CLI. The render graph essentially determines the order in which passes should be run and on which images they should depend. This is useful for creating complex effect pipelines with many steps. 

For example, to define a render graph that computes the luminance of an image: 

```text
lum := Luminance {
    method: Standard
};

lum -> source;

lum!
```

- The `:=` indicates that the `Luminance` pass (with its corresponding parameters) will added to the render graph under the name `lum`.
- The `->` indicates that the `lum` pass depends on the `source` pass, a static containing the originally supplied image. This means the original image will be supplied to the `lum` pass as a dependency, and the output of the `lum` pass will be based on the original image. A passes dependencies can be found in its implementation of the [`Pass`] trait under the [`dependencies`] function.
- The `!` indicates that the `lum` pass is the graph's root and thus will be the final output image.

Or, to gaussian blur with a configurable standard deviation after computing the luminance:

```text
sigma = *stdev | 5.0;

lum := Luminance {
    method: Standard
};

gauss := GaussianBlur {
    sigma: .sigma,
};

lum -> source;
gauss -> lum;

gauss!
```

- The `*` indicates that `stdev` should be read as an argument from the command line. For example, to run this pipeline with a standard deviation of 2.0, one might run: 

```sh
cargo run --release -- gaussian.nprs input.png output.png stdev=2.0
```

- The `|` indicates that if `stdev` is not supplied as an argument to use `5.0` by default. If this is omitted, the argument will be required.
- The `.sigma` indicates that the expression should evaluate to the value stored inside the `sigma` variable.

This language also supports more features, like struct update notation. For more complex pipelines, visit the `examples` and `effects` folders.

## Creating Custom Effects

To create a custom [`Pass`]:

```rust
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
```

To use this pass in an effect pipeline:

```text
desaturate := Desaturate;

desaturate -> source;

desaturate!
```
