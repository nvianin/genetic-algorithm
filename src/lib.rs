mod quadtree;
use quadtree::QuadTree;
mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct World {
    quad: QuadTree,
}
#[wasm_bindgen]
impl World {
    pub fn new() -> World {
        World {
            quad: QuadTree::new(1.),
        }
    }
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, genetic-algorithm!");
}
