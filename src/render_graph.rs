use std::collections::{HashMap, HashSet, VecDeque};

use glam::UVec2;

use crate::{image::{pixel::{rgba::Rgba, Pixel}, Image}, pass::Pass};

pub struct RenderGraph<'a> {
    // TODO: don't require static lifetime for name of image names
    pub images: HashMap<&'a str, Image<4, f32, Rgba<f32>>>,
    // pub main_image: &'a mut Image<4, f32, Rgba<f32>>,
    // pub aux_images: HashMap<u32, Image<4, f32, Rgba<f32>>>,

    /// The edges of the graph, where a node is directed towards its dependencies.
    pub edges: HashMap<u32, Vec<u32>>,

    pub passes: HashMap<u32, Box<dyn Pass<'a>>>,
    // TODO: don't require static lifetime for name of pass names
    pub names: HashMap<&'a str, u32>,
    
    root: u32,
    node_count: u32,
    resolution: UVec2,
}

impl<'a> RenderGraph<'a> {
    pub fn new(image: Image<4, f32, Rgba<f32>>) -> Self {
        let resolution = image.resolution();

        let mut images = HashMap::new();
        images.insert("main", image);

        RenderGraph {
            images,
            // main_image: target,
            // aux_images: HashMap::new(),
            edges: HashMap::new(),
            passes: HashMap::new(),
            names: HashMap::new(),
            root: 0,
            node_count: 1,
            resolution,
        }
    }

    pub fn connections(&self, node: u32) -> &[u32] {
        match self.edges.get(&node) {
            Some(e) => e,
            None => &[],
        }
    }

    pub fn add_edge(&mut self, from: u32, to: u32) {
        self.edges.entry(from).and_modify(|edges| edges.push(to)).or_insert_with(|| vec![to]);
    }

    pub fn add_node<P: Pass<'a> + 'static>(&mut self, node: P) {
        if self.names.contains_key(node.name()) {
            panic!("graph already contains node {}", node.name());
        }

        let dependencies = node.dependencies();
        let n_dependencies = dependencies.len();

        if n_dependencies == 0 {
            self.add_edge(self.node_count, 0);
        }

        for dependency in dependencies {
            let Some(&id) = self.names.get(dependency) else { continue };
            self.add_edge(self.node_count, id);
        }

        self.names.insert(node.name(), self.node_count);
        self.passes.insert(self.node_count, Box::new(node));
        self.node_count += 1;
    }

    fn is_cyclic(
        &self,
        node: u32,
        visited: &mut HashSet<u32>,
        visit_stack: &mut VecDeque<u32>,
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
        for (_name, &node) in self.names.iter() {
            if self.is_cyclic(node, &mut visited, &mut visit_stack) {
                panic!("graph is cyclic");
            }
        }

        // Out-degree != 0
        let mut from_nodes: HashSet<u32> = HashSet::new();
        // In-degree != 0
        let mut to_nodes: HashSet<u32> = HashSet::new();

        for &from_node in self.edges.keys() {
            from_nodes.insert(from_node);
        }

        for &to_node in self.edges.values().flat_map(|v| v.iter()) {
            to_nodes.insert(to_node);
        }

        let mut found_root = false;

        // Find root and isolated nodes
        for (name, &node) in self.names.iter() {
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
                panic!("graph contains isolated node {}", name);
            }
        }

        // Prepare auxiliary images
        for (_node, pass) in self.passes.iter() {
            let target = pass.target();
            if !self.images.contains_key(target) {
                self.images.insert(target, Image::<4, f32, Rgba<f32>>::new_fill(self.resolution, Rgba::<f32>::BLACK));
            }
        }
    }

    fn render_node(&mut self, node: u32) {
        let mut aux_images = Vec::new();

        if let Some(connections) = self.edges.get(&node) {
            for dependency in connections.clone().into_iter() {
                self.render_node(dependency);
            }
        }

        // If the node doesn't correspond to any pass, that means it is the 'main image' node and
        // we don't need to do anything.
        let Some(pass) = self.passes.get(&node) else {
            return;
        };

        for name in pass.auxiliary_images() {
            // SAFETY: a node will never have its target image also be an auxiliary image.
            unsafe {
                aux_images.push(&*(self.images.get(name).unwrap() as *const _));
            }
        }

        let target = self.images.get_mut(pass.target()).unwrap();

        pass.apply(target, &aux_images);
    }

    pub fn render(&mut self) {
        self.render_node(self.root);
    }

    pub fn main_image(mut self) -> Image<4, f32, Rgba<f32>> {
        self.images.remove("main").unwrap()
    }
}
