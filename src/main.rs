#![allow(unused)]

use half::f16;
use image::{pixel::rgba::Rgba, Image};
use pass::{kuwahara::Kuwahara, luminance::{LuminanceFastPerceivedMethod, LuminanceStandardMethod, MainLuminance}, voronoi::{RelaxedVoronoi, VoronoiMode, VoronoiRelaxWeightMode}, tfm::{sobel::Sobel, structure_tensor::TangentFlowStructureTensor, SobelPostBlur, SobelPreBlur}};
use render_graph::RenderGraph;

mod pass;
mod image;
mod render_graph;

fn main() {
    let input = Image::<4, f32, Rgba<f32>>::read("images/lalaland.png").unwrap();

    let mut render_graph = RenderGraph::new(input);

    // render_graph.add_node(MainLuminance::<LuminanceStandardMethod>::new());
    render_graph.add_node(SobelPreBlur::new(1));
    render_graph.add_node(Sobel {});
    render_graph.add_node(SobelPostBlur::new(5.0));
    render_graph.add_node(TangentFlowStructureTensor {});
    render_graph.add_node(RelaxedVoronoi {
        points: 5000,
        relax_iterations: 50,
        relax_mode: VoronoiRelaxWeightMode::Frequency,
        mode: VoronoiMode::Mosaic,
        invert: false,
        weight_scale: 0.5,
    });
    // render_graph.add_node(Kuwahara {
    //     kernel_size: 20,
    //     sharpness: 8.0,
    //     hardness: 8.0,
    //     alpha: 1.0,
    //     zero_crossing: 0.58,
    //     zeta: None,
    //     passes: 1,
    // });
    // render_graph.add_node(Merge::new("tangent_flow_map", vec!["tfm"]).ensure_opaque());

    render_graph.verify();
    render_graph.render();

    let image = render_graph.main_image();

    let image_u8 = image.to_format::<f16, Rgba<f16>>();
    image_u8.write("output/lalaland.png").unwrap();
}
