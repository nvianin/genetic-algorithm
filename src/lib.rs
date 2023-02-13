mod quadtree;
use quadtree::QuadTree;
mod utils;

use vector2d::Vector2D;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use rand::Rng;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
#[wasm_bindgen(js_namespace=console)]
extern "C" {
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct SerializedVector2(f32, f32);

#[derive(Serialize, Deserialize)]
pub struct SerializedQuadTree {
    pub locations: Vec<(f32, f32)>,
    pub sizes: Vec<f32>,
    pub children: Vec<Vec<usize>>,
}

#[wasm_bindgen]
pub struct World {
    quad: QuadTree,
    pub seed: u32,
    sheep_num: usize,
    wolf_num: usize,
    wolves: Vec<(f32, f32)>,
    sheep: Vec<(f32, f32)>,
}

// Main JS interface to the simulation
#[wasm_bindgen(inspectable)]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(sheep_num: usize, wolf_num: usize) -> World {
        let mut rng = rand::thread_rng();
        World {
            quad: QuadTree::new(1.),
            seed: rng.gen(),

            sheep_num,
            wolf_num,

            sheep: Vec::with_capacity(sheep_num),
            wolves: Vec::with_capacity(wolf_num),
        }
    }

    #[wasm_bindgen]
    pub fn test(&self) -> u32 {
        self.seed
    }

    fn traverse_quadtree(node: &QuadTree, mut result: SerializedQuadTree) -> SerializedQuadTree {
        result.locations.push((node.position.x, node.position.y));
        result.sizes.push(node.size);
        result.children.push(node.children.clone());

        log(&format!("{:#?}", node.child_nodes.len()));
        if node.child_nodes.len() > 0 {
            for child in &node.child_nodes {
                result = Self::traverse_quadtree(child, result);
            }
        }

        result
    }

    #[wasm_bindgen]
    pub fn get_quadtree(&self) -> JsValue {
        println!("Getting quadtree as serialized object...");
        let mut result = SerializedQuadTree {
            locations: Vec::new(),
            sizes: Vec::new(),
            children: Vec::new(),
        };

        result = Self::traverse_quadtree(&self.quad, result);

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}

#[wasm_bindgen]
pub struct Test {
    string: String,
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, genetic-algorithm!");
}
