#![allow(unused)]

use half::f16;
use image::{pixel::rgba::Rgba, Image};
use pass::{kuwahara::Kuwahara, luminance::{LuminanceFastPerceivedMethod, LuminanceStandardMethod, MainLuminance}, tfm::{sobel::Sobel, structure_tensor::TangentFlowStructureTensor, SobelPostBlur, SobelPreBlur, TangentFlowMap}, voronoi::{RelaxedVoronoi, VoronoiMode, VoronoiRelaxWeightMode}};
use render_graph::{NodeId, RenderGraph};

mod pass;
mod image;
mod render_graph;

fn main() {
    let input = Image::<4, f32, Rgba<f32>>::read("images/lalaland.png").unwrap();

    let mut render_graph = RenderGraph::new(input);

    let luminance = render_graph.add_node(MainLuminance::<LuminanceStandardMethod>::new(), &[NodeId::SOURCE]);
    // let tfm = render_graph.add_node(TangentFlowMap::new(1, 5.0), &[NodeId::SOURCE]);
    let voronoi = render_graph.add_node(RelaxedVoronoi {
        points: 50000,
        relax_iterations: 20,
        relax_mode: VoronoiRelaxWeightMode::Luminance,
        mode: VoronoiMode::Stippling {
            background: Rgba::new(1.0, 1.0, 1.0, 1.0),
            stipple: Rgba::new(0.0, 0.0, 0.0, 1.0),
            stipple_radius: 1.0,
        },
        invert: true,
        weight_scale: 10.0,
    }, &[NodeId::SOURCE, luminance]);
    // let kuwahara = render_graph.add_node(Kuwahara {
    //     kernel_size: 20,
    //     sharpness: 8.0,
    //     hardness: 8.0,
    //     alpha: 1.0,
    //     zero_crossing: 0.58,
    //     zeta: None,
    //     passes: 1,
    // }, &[NodeId::SOURCE, tfm]);

    render_graph.verify();
    render_graph.render();

    let image = render_graph.pop_image(voronoi).unwrap();

    let image_u8 = image.to_format::<f16, Rgba<f16>>();
    image_u8.write("output/lalaland-stipple.png").unwrap();
}
