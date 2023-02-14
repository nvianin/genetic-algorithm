use std::thread::current;
use wasm_bindgen::prelude::*;

use crate::log;

use serde::{Deserialize, Serialize};

// Clockwise winding of the quads
const QUAD_ORDER: [(f32, f32); 4] = [(0., 0.), (1., 0.), (1., 1.), (0., 1.)];

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct QuadTree {
    pub children: Vec<usize>,
    pub child_nodes: Vec<Box<QuadTree>>,
    pub position: (f32, f32),
    pub size: f32,
    pub level: usize,
    pub index: usize,
    pub address: Vec<usize>,
}

impl QuadTree {
    pub fn new(size: f32) -> QuadTree {
        QuadTree {
            children: Vec::new(),
            child_nodes: Vec::with_capacity(4),
            position: (0., 0.),
            size,
            level: 0,
            index: 0,
            address: vec![0],
        }
    }

    pub fn subdivide(&mut self) {
        /* log(&format!("Subdividing at level {}", self.level)); */
        for i in 0..4 {
            let mut address = self.address.clone();
            address.push(i);
            let q = QuadTree {
                children: Vec::new(),
                child_nodes: Vec::with_capacity(4),
                position: (
                    self.position.0 + self.size / 2. * QUAD_ORDER[i].0,
                    self.position.1 + self.size / 2. * QUAD_ORDER[i].1,
                ),
                size: self.size / 2.,
                level: self.level + 1,
                index: i,
                address,
            };
            self.child_nodes.push(Box::new(q));
        }
        /* log(&format!("{:#?}", self)); */
        println!("{:#?}", self);
    }

    pub fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.position.0
            && point.0 <= self.position.0 + self.size
            && point.1 >= self.position.1
            && point.1 <= self.position.1 + self.size
    }

    pub fn gather_children(node: &QuadTree, mut result: Vec<QuadTree>) -> Vec<QuadTree> {
        result.push(node.clone());

        for child in &node.child_nodes {
            result = Self::gather_children(child, result);
        }

        result
    }

    pub fn get_mut_child_at(&mut self, mut address: Vec<usize>) -> &mut QuadTree {
        address.reverse();
        let mut current_node = self;
        while address.len() > 1 {
            current_node = &mut *current_node.child_nodes[address.pop().unwrap()]
        }
        current_node
    }

    pub fn get_child_at(&self, mut address: Vec<usize>) -> &QuadTree {
        address.reverse();
        let mut current_node = self;
        while address.len() > 1 {
            current_node = &*current_node.child_nodes[address.pop().unwrap()]
        }
        current_node
    }

    pub fn get_all(&self) -> Vec<Box<QuadTree>> {
        let mut result = Vec::new();
        for child in &self.child_nodes {
            result.append(&mut child.get_all())
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::QuadTree;

    #[test]
    fn quadtree_tests() {
        let mut q = QuadTree::new(1.);
        q.subdivide();
    }
}
