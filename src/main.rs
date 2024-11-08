use half::f16;
use image::{pixel::rgba::Rgba, Image};
use pass::luminance::{Luminance, LuminanceFastPerceivedMethod};
use render_graph::RenderGraph;

mod pass;
mod image;
mod render_graph;

fn main() {
    let mut image = Image::<4, f32, Rgba<f32>>::read("images/pagoda.png").unwrap();

    let mut render_graph = RenderGraph::new(&mut image);

    render_graph.add_node(Luminance::<LuminanceFastPerceivedMethod>::new());

    render_graph.verify();
    render_graph.render();

    let image_u8 = image.to_format::<f16, Rgba<f16>>();
    image_u8.write("output/pagoda.png").unwrap();
}
