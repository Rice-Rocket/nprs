use std::{collections::{HashMap, HashSet, VecDeque}, ops::Deref};

use glam::UVec2;

use crate::{image::{pixel::{rgba::Rgba, Pixel}, Image}, pass::Pass};

pub struct RenderGraph<'a> {
    pub images: HashMap<NodeId, Image<4, f32, Rgba<f32>>>,

    /// The edges of the graph, where a node is directed towards its dependencies.
    pub edges: HashMap<NodeId, Vec<NodeId>>,

    pub passes: HashMap<NodeId, Box<dyn Pass<'a>>>,
    pub names: HashSet<&'a str>,
    
    root: NodeId,
    node_count: NodeId,
    resolution: UVec2,
}

impl<'a> RenderGraph<'a> {
    pub fn new(image: Image<4, f32, Rgba<f32>>) -> Self {
        let resolution = image.resolution();

        let mut images = HashMap::new();
        images.insert(NodeId::SOURCE, image);

        let mut names = HashSet::new();
        names.insert("main");

        RenderGraph {
            images,
            // main_image: target,
            // aux_images: HashMap::new(),
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
    pub fn add_node<P: Pass<'a> + 'static>(&mut self, node: P, dependencies: &[NodeId]) -> NodeId {
        let id = self.node_count;

        // TODO: make sure dependencies match what the pass requires.
        for dependency in dependencies {
            self.add_edge(id, *dependency);
        }

        self.names.insert(node.name());
        self.passes.insert(id, Box::new(node));
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

    pub fn verify(&mut self) {
        let mut visited = HashSet::new();
        let mut visit_stack = VecDeque::new();

        // Detect cyclic graph
        for &node in self.passes.keys().chain([NodeId::SOURCE].iter()) {
            if self.is_cyclic(node, &mut visited, &mut visit_stack) {
                panic!("graph is cyclic");
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
                    panic!("graph has more than one root");
                }

                self.root = node;
                found_root = true;
            }

            if !to_nodes.contains(&node) && !from_nodes.contains(&node) {
                // TODO: just warn and remove
                panic!("graph contains isolated node of id {}", node);
            }
        }

        // Check for missing dependencies
        for (node, pass) in self.passes.iter() {
            for dependency in pass.dependencies() {
                if !self.names.contains(dependency) {
                    panic!("graph is missing dependency '{}' for pass '{}'", dependency, pass.name());
                }
            }
        }

        // Prepare auxiliary images
        for (node, pass) in self.passes.iter() {
            if !self.images.contains_key(node) {
                self.images.insert(*node, Image::<4, f32, Rgba<f32>>::new_fill(self.resolution, Rgba::<f32>::BLACK));
            }
        }
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
