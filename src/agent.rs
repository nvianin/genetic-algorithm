use std::{collections::HashMap, f32::MIN, fmt::Display};

use crate::{
    genes::{self, Genotype},
    normalize_vector,
};

use rand::rngs::ThreadRng;
use uuid::Uuid;

use noise::{NoiseFn, OpenSimplex};

#[derive(Clone)]
pub enum AgentType {
    Wolf(Genotype),
    Sheep(Genotype),
    Grass(),
}

impl AgentType {
    pub fn to_int(&self) -> u8 {
        match self {
            AgentType::Wolf(_) => 0,
            AgentType::Sheep(_) => 1,
            AgentType::Grass() => 2,
        }
    }

    pub fn to_name(&self) -> String {
        match self {
            AgentType::Wolf(_) => "Wolf".to_string(),
            AgentType::Sheep(_) => "Sheep".to_string(),
            AgentType::Grass() => "Grass".to_string(),
        }
    }
}

impl Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AgentType::Wolf(_) => "Wolf",
                AgentType::Sheep(_) => "Sheep",
                AgentType::Grass() => "Grass",
            }
        )
    }
}

const MIN_HUNGER: f32 = 30.;
const SATIETY: f32 = 40.;
const HUNGER_RATE: f32 = 0.001;
const BITE_SIZE: f32 = 10.;
const WANDER_SPEED: f32 = 0.1;
const STARVING_DAMAGE: f32 = 0.1;
/* const MAX_WANDER_SPEED: f32 = 0.1; */

#[derive(Clone)]
pub struct Agent {
    pub kind: AgentType,
    pub position: (f32, f32),
    pub acceleration: (f32, f32),
    pub direction: f32,
    pub id: Uuid,
    pub health: f32,
    pub hunger: f32,
    pub life: f32,
    pub dead: bool,
    pub state: State,
    pub seed: f64,
}

impl Agent {
    pub fn new(kind: AgentType, position: (f32, f32), id: Uuid, seed: f64) -> Agent {
        let genes = match kind {
            AgentType::Wolf(genes) => Some(genes),
            AgentType::Sheep(genes) => Some(genes),
            AgentType::Grass() => None,
        };
        let mut health_mult = 1.;
        match genes {
            Some(genes) => {
                health_mult = genes.health_scale;
            }
            None => health_mult = 100.,
        }
        Agent {
            kind,
            position,
            acceleration: (0., 0.),
            direction: 0.,
            id,
            health: health_mult,
            hunger: MIN_HUNGER,
            life: 0.,
            dead: false,
            state: State::Idle,
            seed: seed * 10000.,
        }
    }

    pub fn update(
        &mut self,
        nearby_agents: Vec<(Uuid, (f32, f32))>,
        agents: &HashMap<Uuid, Agent>,
        genotype: Genotype,
        noise: &OpenSimplex,
        time: f32,
    ) -> HashMap<Uuid, Agent> {
        let mut modified_agents = HashMap::new();
        self.position.0 += (self.acceleration.0)
            .max(-genotype.movement_speed)
            .min(genotype.movement_speed);
        self.position.1 += (self.acceleration.1)
            .max(-genotype.movement_speed)
            .min(genotype.movement_speed);

        self.hunger -= HUNGER_RATE * genotype.hunger_rate;
        if self.hunger <= 0. {
            self.hunger = 0.;
            self.health -= STARVING_DAMAGE;
        }

        self.acceleration.0 *= 0.9;
        self.acceleration.1 *= 0.9;
        match self.kind {
            AgentType::Sheep(genotype) => {
                match self.state {
                    State::Idle => {
                        // Check if nearby wolf
                        for nearby_agent in nearby_agents.iter() {
                            match agents.get(&nearby_agent.0).unwrap().kind {
                                AgentType::Wolf(_) => {
                                    self.state = State::Fleeing;
                                }
                                _ => {}
                            }
                        }

                        if self.hunger < MIN_HUNGER {
                            // Get nearby food
                            for nearby_agent in nearby_agents.iter() {
                                match agents.get(&nearby_agent.0).unwrap().kind {
                                    AgentType::Grass() => {
                                        self.state = State::Hunting(nearby_agent.0);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        self.direction += (noise.get([self.seed, time as f64]) as f32) * 0.1;

                        self.acceleration.0 +=
                            self.direction.cos() * genotype.movement_speed * WANDER_SPEED;
                        self.acceleration.1 +=
                            self.direction.sin() * genotype.movement_speed * WANDER_SPEED;
                    }
                    State::Hunting(target) => {
                        // Check if target is still nearby
                        match nearby_agents.iter().find(|a| a.0 == target) {
                            Some(a) => {
                                let mut prey = agents.get(&a.0).unwrap().clone();
                                // Found prey, continuing predator routine
                                let prey_direction = normalize_vector((
                                    prey.position.0 - self.position.0,
                                    prey.position.1 - self.position.1,
                                ));
                                self.acceleration.0 += prey_direction.0;
                                self.acceleration.1 += prey_direction.1;
                                if (prey.position.0 - self.position.0).abs() < 1.
                                    && (prey.position.1 - self.position.1).abs() < 1.
                                {
                                    self.acceleration.0 = 0.;
                                    self.acceleration.1 = 0.;

                                    // Eat prey
                                    self.hunger += prey.eat(BITE_SIZE);
                                    if self.hunger > SATIETY {
                                        self.state = State::Idle;
                                    }
                                    if self.hunger > 100. {
                                        self.hunger = 100.
                                    }
                                }
                                modified_agents.insert(a.0, prey);
                            }
                            None => {
                                self.state = State::Idle;
                            }
                        }
                    }
                    State::Fleeing => {
                        match self.kind {
                            AgentType::Sheep(_) => {
                                // Get nearby wolves
                                let mut med_dir = (0., 0.);
                                let mut predator_count = 0;
                                for id in nearby_agents.iter() {
                                    match agents.get(&id.0).unwrap().kind {
                                        AgentType::Wolf(_) => {
                                            let pos = agents.get(&id.0).unwrap().position;
                                            med_dir.0 += pos.0;
                                            med_dir.1 += pos.1;
                                            predator_count += 1;
                                        }
                                        _ => {}
                                    }
                                }
                                if predator_count > 0 {
                                    med_dir.0 /= predator_count as f32;
                                    med_dir.1 /= predator_count as f32;
                                    self.acceleration.0 += med_dir.0;
                                    self.acceleration.1 += med_dir.1;
                                } else {
                                    self.state = State::Idle;
                                }
                            }
                            _ => {}
                        }
                    }
                    State::Eating(target) => {}
                    State::Dead => self.dead = true,
                }
                self.hunger -= HUNGER_RATE * genotype.hunger_rate;
            }
            AgentType::Wolf(genotype) => {}
            AgentType::Grass() => {}
        }

        /* self.health -= 1.; */
        if self.health <= 0. {
            self.dead = true;
            self.state = State::Dead;
        }

        if self.position.0 > 1024. {
            self.position.0 -= 1024.
        } else if self.position.0 < 0. {
            self.position.0 += 1024.
        }
        if self.position.1 > 1024. {
            self.position.1 -= 1024.
        } else if self.position.1 < 0. {
            self.position.1 += 1024.
        }

        modified_agents
    }

    pub fn get_closest_food(&self, agent_list: Vec<Agent>) -> Uuid {
        agent_list.iter().find(|a| a.kind.to_int() == 2).unwrap().id
    }

    pub fn eat(&mut self, bite: f32) -> f32 {
        if let AgentType::Grass() = self.kind {
            if self.health - bite < 0. {
                self.dead = true;
                return self.health % bite;
            } else {
                self.health -= bite;
                return self.health;
            }
        } else {
            panic!("Tried to eat non-food agent {}!", self.kind);
        }
    }
}

#[derive(Clone)]
pub enum State {
    Idle,
    Hunting(Uuid),
    Fleeing,
    Eating(Uuid),
    Dead,
}
impl State {
    pub fn to_int(&self) -> u8 {
        match self {
            State::Idle => 0,
            State::Hunting(_) => 1,
            State::Fleeing => 2,
            State::Eating(_) => 3,
            State::Dead => 4,
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle => write!(f, "Idle"),
            State::Hunting(_) => write!(f, "Hunting"),
            State::Fleeing => write!(f, "Fleeing"),
            State::Eating(_) => write!(f, "Eating"),
            State::Dead => write!(f, "Dead"),
        }
    }
}
