use std::{collections::{HashMap, HashSet, VecDeque}, ops::Deref};

use glam::UVec2;
use thiserror::Error;

use crate::{image::{pixel::{rgba::Rgba, Pixel}, Image}, pass::Pass};

/// The string representing the main image dependency.
pub const MAIN_IMAGE: &str = "main";

/// The string representing a wildcard dependency, meaning the pass can accept any image in that
/// spot.
pub const ANY_IMAGE: &str = "*";

pub struct RenderGraph {
    pub images: HashMap<NodeId, Image<4, f32, Rgba<f32>>>,

    /// The edges of the graph, where a node is directed towards its dependencies.
    pub edges: HashMap<NodeId, Vec<NodeId>>,

    pub passes: HashMap<NodeId, Box<dyn Pass>>,
    pub names: HashSet<&'static str>,
    
    root: NodeId,
    node_count: NodeId,
    resolution: UVec2,
}

impl RenderGraph {
    pub fn new(image: Image<4, f32, Rgba<f32>>) -> Self {
        let resolution = image.resolution();

        let mut images = HashMap::new();
        images.insert(NodeId::SOURCE, image);

        let mut names = HashSet::new();
        names.insert(MAIN_IMAGE);

        RenderGraph {
            images,
            edges: HashMap::new(),
            passes: HashMap::new(),
            names,
            root: NodeId(0),
            node_count: NodeId(1),
            resolution,
        }
    }

    pub fn connections(&self, node: NodeId) -> &[NodeId] {
        match self.edges.get(&node) {
            Some(e) => e,
            None => &[],
        }
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.entry(from).and_modify(|edges| edges.push(to)).or_insert_with(|| vec![to]);
    }

    /// Adds a [`Pass`] to this [`RenderGraph`], returning its corresponding [`NodeId`].
    pub fn add_node(&mut self, node: Box<dyn Pass>, dependencies: &[NodeId]) -> NodeId {
        let id = self.node_count;

        for dependency in dependencies {
            self.add_edge(id, *dependency);
        }

        self.names.insert(node.name());
        self.passes.insert(id, node);
        self.node_count += 1;
        
        id
    }

    fn is_cyclic(
        &self,
        node: NodeId,
        visited: &mut HashSet<NodeId>,
        visit_stack: &mut VecDeque<NodeId>,
    ) -> bool {
        if !visited.contains(&node) {
            visited.insert(node);
            visit_stack.push_back(node);

            for &connection in self.connections(node) {
                if self.is_cyclic(connection, visited, visit_stack)
                    || visit_stack.contains(&connection)
                {
                    return true;
                }
            }
        }

        visit_stack.pop_back();
        false
    }

    pub fn verify(&mut self) -> Result<(), RenderGraphVerifyError> {
        let mut visited = HashSet::new();
        let mut visit_stack = VecDeque::new();

        // Detect cyclic graph
        for &node in self.passes.keys().chain([NodeId::SOURCE].iter()) {
            if self.is_cyclic(node, &mut visited, &mut visit_stack) {
                return Err(RenderGraphVerifyError::CyclicGraph);
            }
        }

        // Out-degree != 0
        let mut from_nodes: HashSet<NodeId> = HashSet::new();
        // In-degree != 0
        let mut to_nodes: HashSet<NodeId> = HashSet::new();

        for &from_node in self.edges.keys() {
            from_nodes.insert(from_node);
        }

        for &to_node in self.edges.values().flat_map(|v| v.iter()) {
            to_nodes.insert(to_node);
        }

        let mut found_root = false;

        // Find root and isolated nodes
        for &node in self.passes.keys().chain([NodeId::SOURCE].iter()) {
            // In-degree = 0
            if !to_nodes.contains(&node) {
                if found_root {
                    return Err(RenderGraphVerifyError::MultipleRoots);
                }

                self.root = node;
                found_root = true;
            }

            if !to_nodes.contains(&node) && !from_nodes.contains(&node) {
                // TODO: just warn and remove
                return Err(RenderGraphVerifyError::IsolatedNode(self.passes.get(&node).unwrap().name().to_string()));
            }
        }

        // Check for missing dependencies
        for (node, pass) in self.passes.iter() {
            for (i, dependency) in pass.dependencies().into_iter().enumerate() {
                let Some(dependency_node) = self.connections(*node).get(i) else {
                    if self.connections(*node).len() == pass.dependencies().len() {
                        return Err(RenderGraphVerifyError::MissingConnection(dependency.to_string(), pass.name().to_string()));
                    } else {
                        return Err(RenderGraphVerifyError::BadDependencyCount(
                            self.connections(*node).len(),
                            pass.dependencies().len(),
                            pass.name().to_string(),
                        ));
                    }
                };

                if dependency == ANY_IMAGE {
                    continue;
                }

                if !self.names.contains(dependency) {
                    return Err(RenderGraphVerifyError::MissingDependency(dependency.to_string(), pass.name().to_string()));
                }

                match self.passes.get(dependency_node) {
                    Some(edge_pass) => {
                        if edge_pass.name() != dependency {
                            return Err(RenderGraphVerifyError::MismatchedDependency(
                                pass.name().to_string(),
                                dependency.to_string(),
                                edge_pass.name().to_string(),
                                i,
                            ));
                        }
                    },
                    // Main image node
                    None => {
                        if dependency != MAIN_IMAGE {
                            return Err(RenderGraphVerifyError::MismatchedDependency(
                                pass.name().to_string(),
                                dependency.to_string(),
                                "main".to_string(),
                                i,
                            ));
                        }
                    }
                }
            }
        }

        // Prepare auxiliary images
        for (node, pass) in self.passes.iter() {
            if !self.images.contains_key(node) {
                self.images.insert(*node, Image::<4, f32, Rgba<f32>>::new_fill(self.resolution, Rgba::<f32>::BLACK));
            }
        }

        Ok(())
    }

    fn render_node(&mut self, node: NodeId) {
        let mut aux_images = Vec::new();

        if let Some(connections) = self.edges.get(&node) {
            for dependency in connections.clone().into_iter() {
                self.render_node(dependency);

                unsafe {
                    aux_images.push(&*(self.images.get(&dependency).unwrap() as *const _));
                }
            }
        }

        // If the node doesn't correspond to any pass, that means it is the 'source' node and
        // we don't need to do anything.
        let Some(pass) = self.passes.get(&node) else {
            return;
        };

        let target = self.images.get_mut(&node).unwrap();

        pass.apply(target, &aux_images);
    }

    pub fn render(&mut self) {
        self.render_node(self.root);
    }

    pub fn main_image(mut self) -> Image<4, f32, Rgba<f32>> {
        self.images.remove(&NodeId::SOURCE).unwrap()
    }

    pub fn pop_image(&mut self, node: NodeId) -> Option<Image<4, f32, Rgba<f32>>> {
        self.images.remove(&node)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct NodeId(u32);

impl NodeId {
    /// The [`NodeId`] corresponding to the original input image.
    pub const SOURCE: NodeId = NodeId(0);
}

impl Deref for NodeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Add<u32> for NodeId {
    type Output = NodeId;

    fn add(self, rhs: u32) -> Self::Output {
        NodeId(*self + rhs)
    }
}

impl std::ops::AddAssign<u32> for NodeId {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum RenderGraphVerifyError {
    /// Graph is cyclic.
    #[error("graph is cyclic.")]
    CyclicGraph,
    /// Graph has multiple roots.
    // TODO: report names of roots
    #[error("graph has more than one root.")]
    MultipleRoots,
    /// Graph contains isolated node.
    #[error("graph contains isolated node '{0}'")]
    IsolatedNode(String),
    /// Graph is missing connection between required dependency for pass.
    #[error("graph is missing connection between dependency '{0}' and pass '{1}'")]
    MissingConnection(String, String),
    /// Graph has different number of edges and dependencies associated with the same pass.
    #[error("graph has different number of edges ({0}) and dependencies ({1}) associated with the same pass '{2}'")]
    BadDependencyCount(usize, usize, String),
    /// Graph is missing dependency for pass.
    #[error("graph is missing dependency '{0}' for pass '{1}'")]
    MissingDependency(String, String),
    /// Graph has mismatched edge and dependency.
    #[error("graph has mismatched edge and dependency (pass '{0}' depends on '{1}', was given '{2}' at index {3})")]
    MismatchedDependency(String, String, String, usize),
}
