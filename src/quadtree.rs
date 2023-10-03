use nickname::Nickname;
use std::{collections::HashMap, thread::current};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::{log, Agent, MAX_CHILDREN, MAX_LEVELS};

use serde::{Deserialize, Serialize};

// Clockwise winding of the quads
const QUAD_ORDER: [(f32, f32); 4] = [(0., 0.), (1., 0.), (1., 1.), (0., 1.)];

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct QuadTree {
    pub children: Vec<(Uuid, (f32, f32))>,
    pub child_nodes: Vec<Box<QuadTree>>,
    pub is_leaf: bool,
    pub position: (f32, f32),
    pub size: f32,
    pub level: usize,
    pub index: usize,
    pub address: Vec<usize>,
    pub name: String,
}

impl QuadTree {
    pub fn new(size: f32) -> QuadTree {
        QuadTree {
            children: Vec::new(),
            child_nodes: Vec::with_capacity(4),
            is_leaf: true,
            position: (0., 0.),
            size,
            level: 0,
            index: 0,
            address: vec![0],
            name: String::from("root"),
        }
    }

    pub fn subdivide(&mut self, namer: &Nickname, agent_list: &HashMap<Uuid, Agent>) {
        /* log(&format!("Subdividing at level {}", self.level)); */
        for i in 0..4 {
            let mut address = self.address.clone();
            address.push(i);
            let q = QuadTree {
                children: Vec::new(),
                child_nodes: Vec::with_capacity(4),
                is_leaf: true,
                position: (
                    self.position.0 + self.size / 2. * QUAD_ORDER[i].0,
                    self.position.1 + self.size / 2. * QUAD_ORDER[i].1,
                ),
                size: self.size / 2.,
                level: self.level + 1,
                index: i,
                address: address.clone(),
                name: namer.name(),
            };
            self.child_nodes.push(Box::new(q));
            self.is_leaf = false;
            /* log(&format!("Subdivided at address {:?}", address)); */
        }
        while self.children.len() >= 1 {
            let child = self.children.pop().unwrap();
            for child_node in &mut self.child_nodes {
                if child_node.insert(child.clone(), agent_list, namer) {
                    break;
                }
            }
        }
        /* log(&format!("{:#?}", self)); */
        println!("{:#?}", self);
    }

    pub fn contains(&self, point: (f32, f32)) -> bool {
        /* log(&format!("{},{} + {}", self.position.0, self.position.1, self.size)); */

        point.0 >= self.position.0
            && point.0 <= self.position.0 + self.size
            && point.1 >= self.position.1
            && point.1 <= self.position.1 + self.size
    }

    pub fn insert(
        &mut self,
        point: (Uuid, (f32, f32)),
        agent_list: &HashMap<Uuid, Agent>,
        namer: &Nickname,
    ) -> bool {
        if self.contains(point.1) && self.is_leaf {
            if self.children.len() < MAX_CHILDREN {
                self.children.push(point.clone());
                return true;
            } else {
                if self.level + 1 <= MAX_LEVELS {
                    self.subdivide(&namer, agent_list);
                    for child in &mut self.child_nodes {
                        if child.insert(point, agent_list, namer) {
                            return true;
                        };
                    }
                    for child_agent in &self.children {
                        for child in &mut self.child_nodes {
                            if child.insert(point, agent_list, namer) {
                                return true;
                            };
                        }
                    }
                } else {
                    self.children.push(point.clone());
                    return true;
                }
                panic!("Failed to insert child into quadtree");
            }
        } else {
            for child in &mut self.child_nodes {
                if child.insert(point, agent_list, namer) {
                    return true;
                };
            }
            false
        }
    }

    pub fn gather_children(node: &QuadTree, mut result: Vec<QuadTree>) -> Vec<QuadTree> {
        result.push(node.clone());

        for child in &node.child_nodes {
            result = Self::gather_children(child, result);
        }

        result
    }

    pub fn get_child_at(&self, mut address: Vec<usize>) -> &QuadTree {
        log(&format!("{:?}", address));
        address.reverse();
        let mut current_node = self;
        while address.len() > 1 {
            log(&format!("{:?}", address.len()));
            current_node = &*current_node.child_nodes[address.pop().unwrap()]
        }
        current_node
    }

    pub fn get_mut_child_at(&mut self, mut address: Vec<usize>) -> &mut QuadTree {
        address.reverse();
        let mut current_node = self;
        while address.len() > 1 {
            current_node = &mut *current_node.child_nodes[address.pop().unwrap()]
        }
        current_node
    }

    pub fn find_quad_containing_point(&self, point: (f32, f32)) -> Option<&QuadTree> {
        if self.is_leaf && self.contains(point) {
            return Some(&self);
        }
        for child in &self.child_nodes {
            match child.find_quad_containing_point(point) {
                Some(quad) => return Some(quad),
                None => continue,
            };
        }
        None
    }

    pub fn get_all(&self) -> Vec<QuadTree> {
        let mut result = Vec::new();
        result.push(self.clone());
        for child in &self.child_nodes {
            result.append(&mut child.get_all())
        }
        result
    }

    pub fn intersects_circle(&self, position: (f32, f32), radius: f32) -> bool {
        let closest_point = (
            position
                .0
                .max(self.position.0)
                .min(self.position.0 + self.size),
            position
                .1
                .max(self.position.1)
                .min(self.position.1 + self.size),
        );
        let distance =
            (position.0 - closest_point.0).powi(2) + (position.1 - closest_point.1).powi(2);
        // This is an optimization to avoid the sqrt
        distance < radius.powi(2)
    }

    pub fn get_children_in_radius(
        &self,
        position: (f32, f32),
        radius: f32,
        mut result: Vec<(Uuid, (f32, f32))>,
    ) -> Vec<(Uuid, (f32, f32))> {
        if self.intersects_circle(position, radius) {
            let last_len = result.len();
            for child in &self.children {
                if (child.1 .0 - position.0).powi(2) + (child.1 .1 - position.1).powi(2)
                    < radius.powi(2)
                {
                    result.push(*child);
                }
            }
            for child in &self.child_nodes {
                result = child.get_children_in_radius(position, radius, result)
            }
            if result.len() != last_len {
                /* log(&format!(
                    "Found {} children in radius",
                    result.len() - last_len
                )); */
            }
        }
        result
    }
}

/* #[cfg(test)]
mod tests {
    use nickname::NameGen;

    use super::QuadTree;

    #[test]
    fn quadtree_tests() {
        let namer = NameGen::new();

        let mut q = QuadTree::new(1.);
        q.subdivide(&namer);
    }
}
 */
