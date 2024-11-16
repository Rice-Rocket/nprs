use std::{collections::HashMap, io::Read};

use serde::Deserialize;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::{Pass, RenderPass}, render_graph::{NodeId, RenderGraph}};

#[derive(Deserialize)]
pub struct RawRenderGraph {
    passes: HashMap<String, RenderPass>,
    edges: HashMap<String, Vec<String>>,
    display: String
}

impl RawRenderGraph {
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> RawRenderGraph {
        ron::de::from_reader(std::fs::File::open(path).unwrap()).unwrap()
    }

    pub fn build(self, input: Image<4, f32, Rgba<f32>>) -> (RenderGraph, NodeId) {
        let mut render_graph = RenderGraph::new(input);

        let mut nodes = HashMap::new();

        for (name, pass) in self.passes.into_iter() {
            nodes.insert(name, render_graph.add_node(pass.into_pass(), &[]));
        }

        for (name, id) in nodes.iter() {
            let Some(edges) = self.edges.get(name) else { continue };

            for edge in edges {
                if edge == "source" {
                    render_graph.add_edge(*id, NodeId::SOURCE);
                    continue;
                }

                let Some(edge_id) = nodes.get(edge) else {
                    panic!("reference to undefined pass '{}'", edge);
                };

                render_graph.add_edge(*id, *edge_id);
            }
        }

        let Some(display_node) = nodes.get(&self.display) else {
            panic!("reference to undefined pass '{}'", self.display);
        };

        (render_graph, *display_node)
    }
}
