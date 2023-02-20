mod quadtree;
use nickname::NameGen;
use quadtree::QuadTree;

mod genes;
use genes::{SheepGeneticInformation, WolfGeneticInformation};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use rand::Rng;

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

#[derive(Clone)]
pub enum AgentType {
    Wolf(WolfGeneticInformation),
    Sheep(SheepGeneticInformation),
    Grass(f32), // growth stage normalized 0-1
}

impl AgentType {
    pub fn to_int(&self) -> u8 {
        match self {
            AgentType::Wolf(_) => 0,
            AgentType::Sheep(_) => 1,
            AgentType::Grass(_) => 2,
        }
    }
}

#[derive(Clone)]
pub struct Agent {
    pub kind: AgentType,
    pub position: (f32, f32),
    pub acceleration: (f32, f32),
    pub id: Uuid,
    pub health: f32,
    pub hunger: f32,
}

impl Agent {
    pub fn new(kind: AgentType, position: (f32, f32), id: Uuid) -> Agent {
        Agent {
            kind,
            position,
            acceleration: (0., 0.),
            id,
            health: 100.,
            hunger: 30.,
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
        let old_agents = self.agents.clone();
        for agent in self.agents.values_mut() {
            match &mut agent.kind {
                AgentType::Wolf(_) => {}
                AgentType::Sheep(genotype) => {
                    // Try to avoid wolves
                    let mut nearby_wolves = self.wolf_quad.get_children_in_radius(
                        agent.position,
                        genotype.sight_distance,
                        &old_agents,
                    );
                    let mut direction = (0., 0.);
                    if nearby_wolves.len() > 0 {
                        for wolf in nearby_wolves.iter() {
                            direction.0 += agent.position.0 - old_agents[wolf].position.0;
                            direction.1 += agent.position.1 - old_agents[wolf].position.1;
                        }
                        direction.0 /= nearby_wolves.len() as f32;
                        direction.1 /= nearby_wolves.len() as f32;

                        agent.acceleration.0 += direction.0 * 0.01;
                        agent.acceleration.1 += direction.1 * 0.01;
                    }
                }
                AgentType::Grass(_) => {}
            }

            match agent.kind {
                AgentType::Sheep(_) | AgentType::Wolf(_) => {
                    // Move the agent
                    agent.position.0 += agent.acceleration.0;
                    agent.position.1 += agent.acceleration.1;

                    agent.acceleration.0 *= 0.9;
                    agent.acceleration.1 *= 0.9;
                    log(&format!("{:?}, {:?}", agent.position, agent.acceleration));
                }
                AgentType::Grass(_) => {}
            }
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
                        .insert(agent.position, id, &self.agents, &self.namer);
                }
                AgentType::Sheep(_) => {
                    self.sheep_quad
                        .insert(agent.position, id, &self.agents, &self.namer);
                }
                AgentType::Grass(_) => {
                    self.grass_quad
                        .insert(agent.position, id, &self.agents, &self.namer);
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
            children_position.push(self.agents[child].position);
            children_type.push(self.agents[child].kind.to_int());
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
                    AgentType::Wolf(WolfGeneticInformation::default()),
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
                    AgentType::Sheep(SheepGeneticInformation::default()),
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
                    AgentType::Grass(1.),
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
        let agent_indexes = self
            .sheep_quad
            .get_children_in_radius((x, y), radius, &self.agents);
        for id in agent_indexes {
            result.ids.push(id.to_string());
            result.positions.push(self.agents[&id].position);
            result.types.push(self.agents[&id].kind.to_int());
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
}
impl SerializedAgents {
    pub fn new() -> SerializedAgents {
        SerializedAgents {
            ids: Vec::new(),
            positions: Vec::new(),
            types: Vec::new(),
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
