use std::{collections::HashMap, io::Read};

use serde::Deserialize;
use thiserror::Error;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::{Pass, RenderPass}, render_graph::{NodeId, RenderGraph}};

#[derive(Deserialize)]
pub struct RawRenderGraph {
    passes: HashMap<String, RenderPass>,
    edges: HashMap<String, Vec<String>>,
    display: String
}

#[derive(Debug, Error)]
pub enum RenderGraphReadError {
    /// An IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A RON error.
    #[error(transparent)]
    Ron(#[from] ron::error::SpannedError),
    /// Reference to undefined pass.
    #[error("reference to undefined pass '{0}'")]
    UndefinedPass(String),
    /// Duplicate pass name.
    #[error("duplicate pass name '{0}'")]
    DuplicateName(String),
}

impl RawRenderGraph {
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> Result<RawRenderGraph, RenderGraphReadError> {
        Ok(ron::de::from_reader(std::fs::File::open(path)?)?)
    }

    pub fn build(self, input: Image<4, f32, Rgba<f32>>) -> Result<(RenderGraph, NodeId), RenderGraphReadError> {
        let mut render_graph = RenderGraph::new(input);

        let mut nodes = HashMap::new();

        for (name, pass) in self.passes.into_iter() {
            if nodes.insert(name.clone(), render_graph.add_node(pass.into_pass(), &[])).is_some() {
                return Err(RenderGraphReadError::DuplicateName(name))
            }
        }

        for (name, id) in nodes.iter() {
            let Some(edges) = self.edges.get(name) else { continue };

            for edge in edges {
                if edge == "source" {
                    render_graph.add_edge(*id, NodeId::SOURCE);
                    continue;
                }

                let Some(edge_id) = nodes.get(edge) else {
                    return Err(RenderGraphReadError::UndefinedPass(edge.to_string()));
                };

                render_graph.add_edge(*id, *edge_id);
            }
        }

        let Some(display_node) = nodes.get(&self.display) else {
            return Err(RenderGraphReadError::UndefinedPass(self.display));
        };

        Ok((render_graph, *display_node))
    }
}
