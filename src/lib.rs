mod quadtree;
use quadtree::QuadTree;

mod genes;
use genes::Genotype;

mod agent;
use agent::{Agent, AgentType, State};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use rand::{rngs::ThreadRng, Rng};

use uuid::Uuid;

use std::{collections::HashMap, thread::current};

use noise::{NoiseFn, OpenSimplex};

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
    
    size: f32,
    pub seed: u32,
    rng: ThreadRng,
    sheep_num: usize,
    wolf_num: usize,
    noise: noise::OpenSimplex,
    agents: HashMap<Uuid, Agent>,
    to_remove: Vec<Uuid>,
    optimized_query: bool,
}

extern crate console_error_panic_hook;

const MAX_GRASS: usize = 1024;
const MAX_CHILDREN: usize = 16;
const MAX_LEVELS: usize = 6;
const NOISE_SCALING: f64 = 0.01;

// Main JS interface to the simulation
#[wasm_bindgen(inspectable)]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(sheep_num: usize, wolf_num: usize, size: f32) -> World {
        console_error_panic_hook::set_once();

        let mut rng = rand::thread_rng();
        let seed = rng.gen();
        let mut w = World {
            wolf_quad: QuadTree::new(size),
            sheep_quad: QuadTree::new(size),
            grass_quad: QuadTree::new(size),
    
            size,
            seed,
            rng: rand::thread_rng(),

            sheep_num,
            wolf_num,

            noise: OpenSimplex::new(seed),

            agents: HashMap::with_capacity(sheep_num + wolf_num + MAX_GRASS),
            to_remove: Vec::new(),
            optimized_query: true,
        };
        w.spawn_entities();
        w.build_quadtree_good();
        w
    }

    #[wasm_bindgen]
    pub fn step(&mut self, optimized: bool, time: f32) {
        self.build_quadtree_good();

        self.update_agents(optimized, time);
    }

    fn update_agents(&mut self, optimized: bool, time: f32) {
        /* log(&self.agents.len().to_string()); */
        let mut old_agents = self.agents.clone();
        for (id, agent) in old_agents.iter_mut() {
            /* log(&format!(
                "{}:{:?}",
                agent.kind.to_name(),
                agent.state.to_string()
            )); */
            let mut current_agent = self.agents.get(id).unwrap().clone();
            if current_agent.dead {
                self.to_remove.push(*id);
                continue;
            };

            match agent.kind {
                AgentType::Sheep(genotype) => {
                    let mut nearby_agents = Vec::new();
                    nearby_agents = self.wolf_quad.get_children_in_radius(
                        (agent.position.0, agent.position.1),
                        genotype.sight_distance,
                        nearby_agents,
                    );
                    nearby_agents = self.grass_quad.get_children_in_radius(
                        (agent.position.0, agent.position.1),
                        genotype.sight_distance,
                        nearby_agents,
                    );

                    // Pass a non mutable reeference, return mutations to the agents hashmaps
                    let modified_agents = current_agent.update(
                        nearby_agents,
                        &self.agents,
                        genotype,
                        &self.noise,
                        &mut self.rng,
                        time,
                    );
                    for (id, agent) in modified_agents {
                        self.agents.insert(id, agent);
                    }
                    /* if let AgentType::Sheep(_) = agent.kind {
                        log(&format!("{:?}, {:?}", agent.position, agent.acceleration));
                    } */
                }
                AgentType::Wolf(genotype) => {
                    let mut nearby_agents = Vec::new();
                    nearby_agents = self.sheep_quad.get_children_in_radius(
                        (agent.position.0, agent.position.1),
                        genotype.sight_distance,
                        nearby_agents,
                    );
                    let modified_agents = current_agent.update(
                        nearby_agents,
                        &self.agents,
                        genotype,
                        &self.noise,
                        &mut self.rng,
                        time,
                    );
                    for (id, agent) in modified_agents {
                        self.agents.insert(id, agent);
                    }
                }
                AgentType::Grass() => {
                    if current_agent.health <= 0. {
                        self.to_remove.push(*id);
                        current_agent.dead = true;
                        current_agent.state = State::Dead;
                    }
                    current_agent.health =
                        (current_agent.health + agent::PLANT_GROWTH_RATE).min(100.);
                }
            }

            self.agents.insert(agent.id, current_agent);
        }

        for to_remove in self.to_remove.iter() {
            /* log(&format!("Removing {:?}", to_remove)); */
            self.agents.remove(to_remove);
        }
        self.to_remove.clear();
    }

    fn build_quadtree_good(&mut self) {
        self.wolf_quad = QuadTree::new(self.size);
        self.sheep_quad = QuadTree::new(self.size);
        self.grass_quad = QuadTree::new(self.size);

        for (id, agent) in self.agents.iter() {
            match agent.kind {
                AgentType::Wolf(_) => {
                    self.wolf_quad
                        .insert((*id, (agent.position.0, agent.position.1)), &self.agents);
                }
                AgentType::Sheep(_) => {
                    self.sheep_quad
                        .insert((*id, (agent.position.0, agent.position.1)), &self.agents);
                }
                AgentType::Grass() => {
                    self.grass_quad
                        .insert((*id, (agent.position.0, agent.position.1)), &self.agents);
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn test(&self) -> u32 {
        let agent_list = HashMap::new();

        let mut q = QuadTree::new(1024.);
        q.subdivide(&agent_list);
        q.child_nodes[0].subdivide(&agent_list);
        q.child_nodes[0].child_nodes[3].subdivide(&agent_list);
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
                    rng.gen::<f64>(),
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
                    rng.gen::<f64>(),
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
                    rng.gen::<f64>(),
                ),
            );
        }
    }

    #[wasm_bindgen]
    pub fn get_agents(&mut self) -> JsValue {
        let mut result = SerializedAgents::new();

        for agent in self.agents.values() {
            result.ids.push(agent.id_string.clone());
            result.seeds.push(agent.seed);
            result.positions.push((
                agent.position.0,
                agent.position.1,
                self.get_noise(
                    agent.position.0 as f64 * 0.01,
                    agent.position.1 as f64 * 0.01,
                ) as f32
                    * 2.
                    - 1.,
            ));
            result.accelerations.push(agent.acceleration);
            result.types.push(agent.kind.to_int());
            match agent.kind {
                AgentType::Sheep(genotype) => {
                    result.genotypes.push(genotype.to_vec());
                }
                AgentType::Wolf(genotype) => {
                    result.genotypes.push(genotype.to_vec());
                }
                AgentType::Grass() => {
                    result.genotypes.push(Vec::new());
                }
            }
            result.states.push(agent.state.to_int());
            result.vitals.push((agent.health, agent.hunger))
        }

        // Delete marked agents after having sent them in a dead state
        for dead in &mut self.to_remove {
            self.agents.remove(dead);
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
        let mut agents_in_radius = Vec::new();
        agents_in_radius = self
            .sheep_quad
            .get_children_in_radius((x, y), radius, agents_in_radius);

        agents_in_radius = self
            .wolf_quad
            .get_children_in_radius((x, y), radius, agents_in_radius);

        agents_in_radius = self
            .grass_quad
            .get_children_in_radius((x, y), radius, agents_in_radius);

        for agent in agents_in_radius {
            result.ids.push(agent.0.to_string());
            result.positions.push((agent.1 .0, agent.1 .1, 0.));
            let a = self.agents.get(&agent.0);
            match a {
                Some(a) => {
                    result.types.push(a.kind.to_int());
                    result.vitals.push((a.health, a.hunger));
                }
                None => {}
            }
        }

        /* log(&format!("{:#?}", &result.positions.len())); */
        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_noise(&self, x: f64, y: f64) -> f64 {
        self.noise.get([x, y])
    }

    pub fn noise_scale() -> f64 {
        0.01
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializedAgents {
    pub ids: Vec<String>,
    pub seeds: Vec<f64>,
    pub positions: Vec<(f32, f32, f32)>,
    pub accelerations: Vec<(f32, f32)>,
    pub types: Vec<u8>,
    pub genotypes: Vec<Vec<f32>>,
    pub states: Vec<u8>,
    pub vitals: Vec<(f32, f32)>, // Health, hunger
}
impl SerializedAgents {
    pub fn new() -> SerializedAgents {
        SerializedAgents {
            ids: Vec::new(),
            seeds: Vec::new(),
            positions: Vec::new(),
            accelerations: Vec::new(),
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
