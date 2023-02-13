use vector2d::Vector2D;
use wasm_bindgen::prelude::*;

// Clockwise winding of the quads
const QUAD_ORDER: [Vector2D<f32>; 4] = [
    Vector2D { x: 0., y: 0. },
    Vector2D { x: 1., y: 0. },
    Vector2D { x: 1., y: 1. },
    Vector2D { x: 0., y: 1. },
];
const MAX_CHILDREN: usize = 2;
const MAX_LEVELS: usize = 32;

#[derive(Debug)]
pub struct QuadTree {
    pub children: Vec<usize>,
    pub child_nodes: Vec<Box<QuadTree>>,
    pub position: Vector2D<f32>,
    pub size: f32,
    pub level: i32,
}

impl QuadTree {
    pub fn new(size: f32) -> QuadTree {
        QuadTree {
            children: Vec::new(),
            child_nodes: Vec::with_capacity(4),
            position: Vector2D { x: 0., y: 0. },
            size,
            level: 0,
        }
    }

    pub fn subdivide(&mut self) {
        for i in 0..4 {
            let q = QuadTree {
                children: Vec::new(),
                child_nodes: Vec::with_capacity(4),
                position: Vector2D {
                    x: self.size / 2. * QUAD_ORDER[i].x,
                    y: self.size / 2. * QUAD_ORDER[i].y,
                },
                size: self.size / 2.,
                level: self.level + 1,
            };
            self.child_nodes.push(Box::new(q));
        }
        println!("{:#?}", self);
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
