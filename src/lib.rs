#![allow(unused)]

extern crate self as nprs;

use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use half::f16;
use image::{pixel::{rgb::Rgb, rgba::Rgba}, Image, ImageError};
use parser::{cli::PassArg, RawRenderGraph, RenderGraphReadError};
use pass::{difference_of_gaussians::DifferenceOfGaussians, kuwahara::Kuwahara, luminance::{Luminance, LuminanceMethod}, tfm::TangentFlowMap, voronoi::RelaxedVoronoi};
use render_graph::{NodeId, RenderGraph, RenderGraphVerifyError};
use thiserror::Error;

mod pass;
mod image;
mod render_graph;
mod parser;

extern crate alloc;

// Used by proc macros
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
    /// The path to read as the render graph.
    render_graph: PathBuf,

    /// The file to read as the input image.
    input: PathBuf,

    /// The file to write the processed image to.
    outfile: PathBuf,

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

pub fn render() -> Result<(), NprsError> {
    let args = Args::parse();

    let input = Image::<4, f32, Rgba<f32>>::read(args.input)?;

    let (mut render_graph, display_node) = RawRenderGraph::read(args.render_graph, args.args)?.build(input)?;

    render_graph.verify()?;
    render_graph.render();

    let image = render_graph.pop_image(display_node).unwrap();

    let image_u8 = image.map(|pixel| pixel.rgb() * pixel.a).to_format::<f16, Rgb<f16>>();
    image_u8.write(args.outfile)?;

    Ok(())
}
