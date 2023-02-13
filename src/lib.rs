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
    pub has_child_nodes: Vec<bool>,
    pub address: Vec<Vec<usize>>,
}
impl SerializedQuadTree {
    pub fn new() -> SerializedQuadTree {
        SerializedQuadTree {
            locations: Vec::new(),
            sizes: Vec::new(),
            children: Vec::new(),
            has_child_nodes: Vec::new(),
            address: Vec::new(),
        }
    }
}

pub struct WolfGeneticInformation {
    pub movement_speed: f32,
    pub sight_distance: f32,

    pub hunger: f32,
    pub health: f32,
}

impl WolfGeneticInformation {
    pub fn default() -> WolfGeneticInformation {
        WolfGeneticInformation {
            movement_speed: 1.,
            sight_distance: 1.,
            hunger: 30.,
            health: 100.,
        }
    }
}

pub struct SheepGeneticInformation {
    pub movement_speed: f32,
    pub sight_distance: f32,

    pub hunger: f32,
    pub health: f32,
}

impl SheepGeneticInformation {
    pub fn default() -> SheepGeneticInformation {
        SheepGeneticInformation {
            movement_speed: 1.5,
            sight_distance: 1.,
            hunger: 30.,
            health: 100.,
        }
    }
}

enum AgentType {
    Wolf(WolfGeneticInformation),
    Sheep(SheepGeneticInformation),
    Grass(f32),
}

struct Agent {
    pub kind: AgentType,
    pub position: (f32, f32),
}

#[wasm_bindgen]
pub struct World {
    quad: QuadTree,
    size: f32,
    pub seed: u32,
    sheep_num: usize,
    wolf_num: usize,

    agents: Vec<Agent>,
}

extern crate console_error_panic_hook;

const MAX_GRASS: usize = 1024;
const MAX_CHILDREN: usize = 2;
const MAX_LEVELS: usize = 32;

// Main JS interface to the simulation
#[wasm_bindgen(inspectable)]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(sheep_num: usize, wolf_num: usize, size: f32) -> World {
        console_error_panic_hook::set_once();

        let mut rng = rand::thread_rng();
        let mut w = World {
            quad: QuadTree::new(size),
            size,
            seed: rng.gen(),

            sheep_num,
            wolf_num,

            agents: Vec::new(),
        };
        w.spawn_entities();
        w.build_quadtree();
        w
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.build_quadtree();
    }

    fn build_quadtree(&mut self) {
        let mut done = false;
        let mut iterations = 0;
        self.quad = QuadTree::new(self.size);

        while !done {
            iterations += 1;
            if iterations >= 1000 {
                log(&format!("Stopping after {iterations} iterations"));
                break;
            };
            done = true;
            let scheme = QuadTree::gather_children(&self.quad, Vec::new());
            for quad in scheme {
                if quad.child_nodes.len() == 0 {
                    let mut contained_agents = 0;
                    for agent in &self.agents {
                        if quad.contains(agent.position) {
                            contained_agents += 1;
                        }
                        if contained_agents > MAX_CHILDREN && quad.level < MAX_LEVELS {
                            log(&format!("{:#?}", self.quad));
                            self.quad.get_mut_child_at(quad.address.clone()).subdivide();
                            /* log(&format!("{:#?}", quad)); */
                            done = false;
                            break;
                        }
                    }
                }
                if !done {
                    break;
                };
            }
            /* done = true; */
        }
    }

    #[wasm_bindgen]
    pub fn test(&self) -> u32 {
        let mut q = QuadTree::new(1024.);
        q.subdivide();
        q.child_nodes[0].subdivide();
        q.child_nodes[0].child_nodes[3].subdivide();
        log(&format!("{:#?}", q));
        self.seed
    }

    fn traverse_quadtree(node: &QuadTree, mut result: SerializedQuadTree) -> SerializedQuadTree {
        result.locations.push((node.position.x, node.position.y));
        result.sizes.push(node.size);
        result.children.push(node.children.clone());
        result.has_child_nodes.push(node.child_nodes.len() > 0);
        result.address.push(node.address.clone());

        /* log(&format!("{:#?}", node.child_nodes.len())); */
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
        let mut result = SerializedQuadTree::new();

        result = Self::traverse_quadtree(&self.quad, result);

        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    fn spawn_entities(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..self.wolf_num {
            self.agents.push(Agent {
                position: (
                    rng.gen::<f32>() * self.quad.size,
                    rng.gen::<f32>() * self.quad.size,
                ),
                kind: AgentType::Wolf(WolfGeneticInformation::default()),
            });
        }
        for _ in 0..self.sheep_num {
            self.agents.push(Agent {
                position: (
                    rng.gen::<f32>() * self.quad.size,
                    rng.gen::<f32>() * self.quad.size,
                ),
                kind: AgentType::Sheep(SheepGeneticInformation::default()),
            });
        }

        for _ in 0..MAX_GRASS {
            self.agents.push(Agent {
                position: (
                    rng.gen::<f32>() * self.quad.size,
                    rng.gen::<f32>() * self.quad.size,
                ),
                kind: AgentType::Grass(1.),
            })
        }
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
