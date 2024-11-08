use std::collections::{hash_map::Entry, HashMap, HashSet, VecDeque};
use itertools::Itertools;

use glam::UVec2;

use crate::{image::{pixel::rgba::Rgba, Image}, pass::Pass};

pub struct RenderGraph {
    pub main_image: Image<4, f32, Rgba<f32>>,
    pub aux_images: HashMap<u32, Image<4, f32, Rgba<f32>>>,

    /// The edges of the graph, where a node is directed towards its dependencies.
    pub edges: HashMap<u32, Vec<u32>>,

    pub passes: HashMap<u32, Box<dyn Pass>>,
    pub names: HashMap<&'static str, u32>,
    
    root: u32,
    node_count: u32,
    resolution: UVec2,
}

impl RenderGraph {
    pub fn new(image_resolution: UVec2) -> Self {
        RenderGraph {
            main_image: Image::new_fill(image_resolution, Rgba::new(0.0, 0.0, 0.0, 0.0)),
            aux_images: HashMap::new(),
            edges: HashMap::new(),
            passes: HashMap::new(),
            names: HashMap::new(),
            root: 0,
            node_count: 0,
            resolution: image_resolution,
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

    pub fn add_node<P: Pass + 'static>(&mut self, node: P) {
        for dependency in node.dependencies() {
            let Some(&name) = self.names.get(dependency) else { continue };
            self.add_edge(name, self.node_count);
        }

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
                if self.is_cyclic(connection, visited, visit_stack) {
                    return true;
                } else if visit_stack.contains(&connection) {
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
            for &dependency in pass.dependencies() {
                let Some(&dependency_id) = self.names.get(dependency) else { continue };
                self.aux_images.insert(dependency_id, Image::new_fill(self.resolution, Rgba::new(0.0, 0.0, 0.0, 0.0)));
            }
        }
    }

    fn render_node(&mut self, node: u32) {
        let mut aux_images = Vec::new();

        if let Some(connections) = self.edges.get(&self.root) {
            for &dependency in connections.clone().iter() {
                self.render_node(dependency);

                // SAFETY: all mutable borrows of self.aux_images should be dropped at this point.
                unsafe {
                    aux_images.push(&*(self.aux_images.get(&dependency).unwrap() as *const _));
                }
            }
        }

        // SAFETY: a node will never be dependent on itself, meaning the node's aux image
        // (corresponding to this node) will never be borrowed by this point.
        let mut target = self.aux_images.get_mut(&node).unwrap();
        let pass = self.passes.get(&self.root).unwrap();
        pass.apply(&mut target, &aux_images);
    }

    pub fn render(&mut self) -> Image<4, f32, Rgba<f32>> {
        let mut aux_images = Vec::new();

        if let Some(connections) = self.edges.get(&self.root) {
            for &dependency in connections.clone().iter() {
                self.render_node(dependency);

                // SAFETY: all mutable borrows of self.aux_images should be dropped at this point.
                unsafe {
                    aux_images.push(&*(self.aux_images.get(&dependency).unwrap() as *const _));
                }
            }
        }

        let pass = self.passes.get(&self.root).unwrap();
        pass.apply(&mut self.main_image, &aux_images);

        self.main_image.clone()
    }
}
