#![allow(unused)]

use half::f16;
use image::{pixel::rgba::Rgba, Image};
use pass::{kuwahara::Kuwahara, luminance::{Luminance, LuminanceStandardMethod}, tfm::TangentFlowMap};
use render_graph::{NodeId, RenderGraph};

mod pass;
mod image;
mod render_graph;

fn main() {
    let input = Image::<4, f32, Rgba<f32>>::read("images/lalaland.png").unwrap();

    let mut render_graph = RenderGraph::new(input);

    // let luminance = render_graph.add_node(Luminance::<LuminanceStandardMethod>::new(), &[NodeId::SOURCE]);
    let tfm = render_graph.add_node(TangentFlowMap::new(1, 5.0), &[NodeId::SOURCE]);

    // let voronoi = render_graph.add_node(
    //     RelaxedVoronoi::mosaic(5000).relax_iterations(20).weight_scale(0.5),
    //     &[NodeId::SOURCE, tfm],
    // );

    let kuwahara = render_graph.add_node(
        Kuwahara::new(),
        &[NodeId::SOURCE, tfm],
    );

    render_graph.verify();
    render_graph.render();

    let image = render_graph.pop_image(kuwahara).unwrap();

    let image_u8 = image.to_format::<f16, Rgba<f16>>();
    image_u8.write("output/lalaland-kuwahara.png").unwrap();
}
