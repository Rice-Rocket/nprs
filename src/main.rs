#![allow(unused)]

use half::f16;
use image::{pixel::rgba::Rgba, Image};
use parser::RawRenderGraph;
use pass::{difference_of_gaussians::DifferenceOfGaussians, kuwahara::Kuwahara, luminance::{Luminance, LuminanceMethod}, tfm::TangentFlowMap, voronoi::RelaxedVoronoi};
use render_graph::{NodeId, RenderGraph};

mod pass;
mod image;
mod render_graph;
mod parser;

fn main() {
    let input = Image::<4, f32, Rgba<f32>>::read("images/pagoda.png").unwrap();

    let (mut render_graph, display_node) = RawRenderGraph::read("render_graphs/dog/basic.ron").build(input);

    render_graph.verify();
    render_graph.render();

    let image = render_graph.pop_image(display_node).unwrap();

    let image_u8 = image.to_format::<f16, Rgba<f16>>();
    image_u8.write("output/pagoda-dog.png").unwrap();
}
