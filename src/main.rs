#![allow(unused)]

extern crate self as nprs;

use std::path::PathBuf;

use clap::Parser;
use half::f16;
use image::{pixel::{rgb::Rgb, rgba::Rgba}, Image, ImageError};
use parser::{RawRenderGraph, RenderGraphReadError};
use pass::{difference_of_gaussians::DifferenceOfGaussians, kuwahara::Kuwahara, luminance::{Luminance, LuminanceMethod}, tfm::TangentFlowMap, voronoi::RelaxedVoronoi};
use render_graph::{NodeId, RenderGraph, RenderGraphVerifyError};
use thiserror::Error;

mod pass;
mod image;
mod render_graph;
mod parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The path to read as the render graph.
    render_graph: PathBuf,

    /// The file to read as the input image.
    input: PathBuf,

    /// The file to write the processed image to.
    outfile: PathBuf,
}

#[derive(Debug, Error)]
enum NprsError {
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

fn render() -> Result<(), NprsError> {
    let args = Args::parse();

    let input = Image::<4, f32, Rgba<f32>>::read(args.input)?;

    let (mut render_graph, display_node) = RawRenderGraph::read(args.render_graph)?.build(input)?;

    render_graph.verify()?;
    render_graph.render();

    let image = render_graph.pop_image(display_node).unwrap();

    let image_u8 = image.map(|pixel| pixel.rgb() * pixel.a).to_format::<f16, Rgb<f16>>();
    image_u8.write(args.outfile)?;

    Ok(())
}

fn main() {
    match render() {
        Ok(_) => (),
        Err(err) => println!("error: {}", err),
    }
}
