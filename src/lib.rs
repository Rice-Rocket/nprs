extern crate self as nprs;

use std::path::PathBuf;

use clap::Parser;
use half::f16;
use image::{pixel::{rgb::Rgb, rgba::Rgba}, ImageError};
use parser::{cli::PassArg, RenderGraphReadError};
use render_graph::RenderGraphVerifyError;
use thiserror::Error;

pub mod pass;
pub mod image;
pub mod render_graph;
pub mod parser;

pub mod pixel {
    pub use nprs::image::pixel::{
        luma::Luma,
        luma_alpha::LumaAlpha,
        rgb::Rgb,
        rgba::Rgba,
    };
}

pub use nprs_derive::{ParsePass, FromParsedValue};
pub use nprs::{
    pass::{Pass, SubPass},
    parser::RawRenderGraph,
    image::Image,
};

pub extern crate glam;
pub extern crate half;

extern crate alloc;

// Used in proc macro expansion
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub extern crate inventory;

    #[doc(hidden)]
    pub use core::option::Option;
    #[doc(hidden)]
    pub use core::result::Result;

    #[doc(hidden)]
    pub type Box<T> = alloc::boxed::Box<T>;
    #[doc(hidden)]
    pub type String = alloc::string::String;
}

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The path to read as the render graph descriptor.
    render_graph: PathBuf,

    /// The file to read as the input image.
    input: PathBuf,

    /// The file to write the processed image to.
    outfile: PathBuf,

    /// Additional arguments, formatted NAME=VALUE, that will be supplied to the render graph.
    /// NAME should match the identifier used in the given .nprs file and VALUE should be a valid
    /// expression in the nprs language.
    args: Vec<PassArg>,
}

#[derive(Debug, Error)]
pub enum NprsError {
    /// An image error.
    #[error(transparent)]
    Image(#[from] ImageError),
    /// A render graph reading error.
    #[error(transparent)]
    RenderGraphRead(#[from] RenderGraphReadError),
    /// A render graph verification error.
    #[error(transparent)]
    RenderGraphVerify(#[from] RenderGraphVerifyError),
}

pub fn run_cli() -> Result<(), NprsError> {
    let args = Args::parse();

    let input = Image::<4, f32, Rgba<f32>>::read(args.input)?;

    let (mut render_graph, display_node) = RawRenderGraph::read(args.render_graph, args.args)?.build(input)?;

    render_graph.verify()?;
    render_graph.render();

    let image = render_graph.pop_image(display_node).unwrap();

    let image_f16 = image.map(|pixel| pixel.rgb() * pixel.a).to_format::<f16, Rgb<f16>>();
    image_f16.write(args.outfile)?;

    Ok(())
}
