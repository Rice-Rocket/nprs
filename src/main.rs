use half::f16;
use image::{pixel::rgba::Rgba, Image};
use pass::{merge::Merge, luminance::{Luminance, LuminanceFastPerceivedMethod}, sobel::Sobel};
use render_graph::RenderGraph;

mod pass;
mod image;
mod render_graph;

fn main() {
    let input = Image::<4, f32, Rgba<f32>>::read("images/pagoda.png").unwrap();

    let mut render_graph = RenderGraph::new(input);

    render_graph.add_node(Luminance::<LuminanceFastPerceivedMethod>::new());
    render_graph.add_node(Sobel {});
    render_graph.add_node(Merge::new(&["tangent_flow_map"], &["sobel"]).ensure_opaque());

    render_graph.verify();
    render_graph.render();

    let image = render_graph.main_image();

    let image_u8 = image.to_format::<f16, Rgba<f16>>();
    image_u8.write("output/pagoda.png").unwrap();
}
