mod quadtree;
use nickname::NameGen;
use quadtree::QuadTree;

mod genes;
use genes::Genotype;

mod agent;
use agent::{Agent, AgentType};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use rand::{rngs::ThreadRng, Rng};

use uuid::Uuid;

use std::{collections::HashMap, hash::Hash};

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
    pub children: Vec<Vec<(f32, f32)>>,
    pub children_type: Vec<Vec<u8>>,
    pub has_child_nodes: Vec<bool>,
    pub child_nodes: Vec<usize>,
    pub address: Vec<Vec<usize>>,
}
impl SerializedQuadTree {
    pub fn new() -> SerializedQuadTree {
        SerializedQuadTree {
            locations: Vec::new(),
            sizes: Vec::new(),
            children: Vec::new(),
            children_type: Vec::new(),
            has_child_nodes: Vec::new(),
            child_nodes: Vec::new(),
            address: Vec::new(),
        }
    }
}

#[wasm_bindgen]
pub struct World {
    wolf_quad: QuadTree,
    sheep_quad: QuadTree,
    grass_quad: QuadTree,
    namer: NameGen,
    size: f32,
    pub seed: u32,
    rng: ThreadRng,
    sheep_num: usize,
    wolf_num: usize,

    agents: HashMap<Uuid, Agent>,
}

extern crate console_error_panic_hook;

const MAX_GRASS: usize = 8;
const MAX_CHILDREN: usize = 4;
const MAX_LEVELS: usize = 6;

// Main JS interface to the simulation
#[wasm_bindgen(inspectable)]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(sheep_num: usize, wolf_num: usize, size: f32) -> World {
        console_error_panic_hook::set_once();

        let namer = NameGen::new();

        let mut rng = rand::thread_rng();
        let mut w = World {
            wolf_quad: QuadTree::new(size),
            sheep_quad: QuadTree::new(size),
            grass_quad: QuadTree::new(size),
            namer,
            size,
            seed: rng.gen(),
            rng: rand::thread_rng(),

            sheep_num,
            wolf_num,

            agents: HashMap::with_capacity(sheep_num + wolf_num + MAX_GRASS),
        };
        w.spawn_entities();
        w.build_quadtree_good();
        w
    }

    #[wasm_bindgen]
    pub fn step(&mut self, optimized: bool) {
        self.build_quadtree_good();

        self.update_agents(optimized);
    }

    fn update_agents(&mut self, optimized: bool) {
        let mut old_agents = self.agents.clone();
        let mut to_remove = Vec::new();
        for (id, agent) in old_agents.iter_mut() {
            let mut current_agent = self.agents.get_mut(id).unwrap();
            match &agent.kind {
                AgentType::Wolf(_) => {}
                AgentType::Sheep(genotype) => {
                    // Try to avoid wolves
                    let mut nearby_wolves = self
                        .wolf_quad
                        .get_children_in_radius(agent.position, genotype.sight_distance);
                    let mut direction = (0., 0.);
                    if nearby_wolves.len() > 0 {
                        for wolf in nearby_wolves.iter() {
                            direction.0 += agent.position.0 - wolf.1 .0;
                            direction.1 += agent.position.1 - wolf.1 .1;
                        }
                        direction.0 /= nearby_wolves.len() as f32;
                        direction.1 /= nearby_wolves.len() as f32;

                        current_agent.acceleration.0 += direction.0 * 0.01;
                        current_agent.acceleration.1 += direction.1 * 0.01;
                    }
                }
                AgentType::Grass() => {}
            }

            match agent.kind {
                AgentType::Sheep(genotype) => {
                    let mut nearby_agents = Vec::new();
                    nearby_agents.append(
                        &mut self
                            .wolf_quad
                            .get_children_in_radius(agent.position, genotype.sight_distance),
                    );
                    nearby_agents.append(
                        &mut self
                            .grass_quad
                            .get_children_in_radius(agent.position, genotype.sight_distance),
                    );

                    // Pass a non mutable reeference, return mutations to the agents hashmaps
                    let modified_agents = agent.update(nearby_agents, &self.agents, genotype);
                    for (id, agent) in modified_agents {
                        self.agents.insert(id, agent);
                    }
                    /* if let AgentType::Sheep(_) = agent.kind {
                        log(&format!("{:?}, {:?}", agent.position, agent.acceleration));
                    } */
                }
                AgentType::Wolf(genotype) => {
                    let mut nearby_agents = Vec::new();
                    nearby_agents.append(
                        &mut self
                            .sheep_quad
                            .get_children_in_radius(agent.position, genotype.sight_distance),
                    );
                    let modified_agents = agent.update(nearby_agents, &self.agents, genotype);
                    for (id, agent) in modified_agents {
                        self.agents.insert(id, agent);
                    }
                }
                AgentType::Grass() => {}
            }
            if agent.dead {
                to_remove.push(agent.id);
            }
        }
        // Delete marked agents4

        for dead in &to_remove {
            self.agents.remove(dead);
        }
    }

    fn build_quadtree_good(&mut self) {
        self.wolf_quad = QuadTree::new(self.size);
        self.sheep_quad = QuadTree::new(self.size);
        self.grass_quad = QuadTree::new(self.size);

        for (id, agent) in self.agents.iter() {
            match agent.kind {
                AgentType::Wolf(_) => {
                    self.wolf_quad
                        .insert((agent.id, agent.position), &self.agents, &self.namer);
                }
                AgentType::Sheep(_) => {
                    self.sheep_quad
                        .insert((agent.id, agent.position), &self.agents, &self.namer);
                }
                AgentType::Grass() => {
                    self.grass_quad
                        .insert((agent.id, agent.position), &self.agents, &self.namer);
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn test(&self) -> u32 {
        let namer = NameGen::new();
        let agent_list = HashMap::new();

        let mut q = QuadTree::new(1024.);
        q.subdivide(&namer, &agent_list);
        q.child_nodes[0].subdivide(&namer, &agent_list);
        q.child_nodes[0].child_nodes[3].subdivide(&namer, &agent_list);
        log(&format!("{:#?}", q));
        self.seed
    }

    fn traverse_quadtree(
        &self,
        node: &QuadTree,
        mut result: SerializedQuadTree,
    ) -> SerializedQuadTree {
        result.locations.push((node.position.0, node.position.1));
        result.sizes.push(node.size);

        let mut children_position = Vec::new();
        let mut children_type = Vec::new();
        for child in &node.children {
            children_position.push(child.1);
            children_type.push(self.agents.get(&child.0).unwrap().kind.to_int());
        }
        result.children.push(children_position);
        result.children_type.push(children_type);

        result.has_child_nodes.push(node.child_nodes.len() > 0);
        result.address.push(node.address.clone());

        /* log(&format!("{:#?}", node.child_nodes.len())); */
        if node.child_nodes.len() > 0 {
            for child in &node.child_nodes {
                result = self.traverse_quadtree(child, result);
            }
        }

        result
    }

    #[wasm_bindgen]
    pub fn get_quadtree(&self) -> JsValue {
        let result = self.sheep_quad.get_all();
        /* log(&format!("{:?}", result)); */

        /* let mut result = SerializedQuadTree::new();
        result = self.traverse_quadtree(&self.quad, result); */

        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    fn spawn_entities(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..self.wolf_num {
            let id = Uuid::new_v4();
            self.agents.insert(
                id,
                Agent::new(
                    AgentType::Wolf(Genotype::new(&mut self.rng)),
                    (rng.gen::<f32>() * self.size, rng.gen::<f32>() * self.size),
                    id,
                ),
            );
        }

        for _ in 0..self.sheep_num {
            let id = Uuid::new_v4();
            self.agents.insert(
                id,
                Agent::new(
                    AgentType::Sheep(Genotype::new(&mut self.rng)),
                    (rng.gen::<f32>() * self.size, rng.gen::<f32>() * self.size),
                    id,
                ),
            );
        }

        for _ in 0..MAX_GRASS {
            let id = Uuid::new_v4();
            self.agents.insert(
                id,
                Agent::new(
                    AgentType::Grass(),
                    (rng.gen::<f32>() * self.size, rng.gen::<f32>() * self.size),
                    id,
                ),
            );
        }
    }

    #[wasm_bindgen]
    pub fn get_agents(&self) -> JsValue {
        let mut result = SerializedAgents::new();

        for agent in self.agents.values() {
            result.ids.push(agent.id.to_string());
            result.positions.push(agent.position);
            result.types.push(agent.kind.to_int());
            match agent.kind {
                AgentType::Sheep(genotype) => {
                    result.genotypes.push(genotype.to_hashmap());
                }
                AgentType::Wolf(genotype) => {
                    result.genotypes.push(genotype.to_hashmap());
                }
                AgentType::Grass() => {
                    result.genotypes.push(HashMap::new());
                }
            }
            result.states.push(agent.state.to_int());
            result.vitals.push((agent.health, agent.hunger))
        }

        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    #[wasm_bindgen]
    pub fn activate(&self, mouse_x: f32, mouse_y: f32) -> JsValue {
        /* return serde_wasm_bindgen::to_value(
            &self
                .quad
                .get_child_at(vec![0])
                .contains((mouse_x, mouse_y)),
        )
        .unwrap(); */

        match self
            .sheep_quad
            .find_quad_containing_point((mouse_x, mouse_y))
        {
            Some(q) => serde_wasm_bindgen::to_value(q).unwrap(),
            None => wasm_bindgen::JsValue::NULL,
        }
    }

    #[wasm_bindgen]
    pub fn get_agents_in_radius(&self, x: f32, y: f32, radius: f32) -> JsValue {
        let mut result = SerializedAgents::new();
        let agent_indexes = self.sheep_quad.get_children_in_radius((x, y), radius);
        for agent in agent_indexes {
            result.ids.push(agent.0.to_string());
            result.positions.push(agent.1);
            let a = self.agents.get(&agent.0).unwrap();
            result.types.push(a.kind.to_int());
            result.vitals.push((a.health, a.hunger));
        }

        /* log(&format!("{:#?}", &result.positions.len())); */
        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializedAgents {
    pub ids: Vec<String>,
    pub positions: Vec<(f32, f32)>,
    pub types: Vec<u8>,
    pub genotypes: Vec<HashMap<String, f32>>,
    pub states: Vec<u8>,
    pub vitals: Vec<(f32, f32)>, // Health, hunger
}
impl SerializedAgents {
    pub fn new() -> SerializedAgents {
        SerializedAgents {
            ids: Vec::new(),
            positions: Vec::new(),
            types: Vec::new(),
            genotypes: Vec::new(),
            states: Vec::new(),
            vitals: Vec::new(),
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

fn vector_length(vec: (f32, f32)) -> f32 {
    (vec.0.powi(2) + vec.1.powi(2)).sqrt()
}

fn normalize_vector(vec: (f32, f32)) -> (f32, f32) {
    let length = vector_length(vec);
    (vec.0 / length, vec.1 / length)
}
