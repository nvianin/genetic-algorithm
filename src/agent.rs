use std::{collections::HashMap, f32::MIN, fmt::Display};

use crate::{
    genes::{self, Genotype},
    normalize_vector,
};

use rand::rngs::ThreadRng;
use uuid::Uuid;

use noise::{NoiseFn, OpenSimplex};

#[derive(Clone, PartialEq)]
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
const WANDER_SPEED: f32 = 2.;
const STARVING_DAMAGE: f32 = 0.1;
pub const PLANT_GROWTH_RATE: f32 = 0.1;
/* const MAX_WANDER_SPEED: f32 = 0.1; */

#[derive(Clone)]
// TODO: 
pub struct Agent {
    pub kind: AgentType,
    pub position: (f32, f32, f32),
    pub acceleration: (f32, f32),
    pub direction: f32,
    pub id: Uuid,
    pub id_string: String,
    pub health: f32,
    pub hunger: f32,
    pub life: f32,
    pub dead: bool,
    pub state: State,
    pub seed: f64,
    pub timeout: f32,
    pub last_time: f32,
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
            None => health_mult = seed as f32 / 10000.,
        }
        Agent {
            kind,
            position: (position.0, position.1, 0.),
            acceleration: (0., 0.),
            direction: (seed as f32 * 10000.) % (std::f32::consts::PI * 2.),
            id,
            id_string: id.to_string(),
            health: health_mult,
            hunger: MIN_HUNGER,
            life: 0.,
            dead: false,
            state: State::Idle,
            seed: seed * 10000.,
            timeout: 0.,
            last_time: 0.,
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
        if self.health <= 0. {
            self.dead = true;
            self.state = State::Dead;
        }

        self.acceleration.0 *= 0.9;
        self.acceleration.1 *= 0.9;
        match self.kind {
            AgentType::Sheep(genotype) => {
                match self.state {
                    State::Fleeing => {
                        if self.health <= 0. {
                            self.dead = true;
                            self.state = State::Dead;
                        }
                        // Get nearby wolves
                        let mut closest_distance = f32::MAX;
                        let mut closest_wolf = None;
                        for id in nearby_agents.iter() {
                            match agents.get(&id.0).unwrap().kind {
                                AgentType::Wolf(_) => {
                                    let pos = agents.get(&id.0).unwrap().position;
                                    let dist = (pos.0.powf(2.) + pos.1.powf(2.))
                                        - (self.position.0.powf(2.) + self.position.1.powf(2.));
                                    if dist < closest_distance {
                                        closest_distance = dist;
                                        closest_wolf = Some(id.0);
                                    }
                                }
                                _ => {}
                            }
                        }
                        match closest_wolf {
                            Some(id) => {
                                let wolf = agents.get(&id).unwrap();
                                let wolf_direction = normalize_vector((
                                    self.position.0 - wolf.position.0,
                                    self.position.1 - wolf.position.1,
                                ));
                                self.acceleration.0 = wolf_direction.0 * genotype.movement_speed;
                                self.acceleration.1 = wolf_direction.1 * genotype.movement_speed;
                            }
                            None => {
                                self.state = State::Idle;
                            }
                        }
                    }
                    _ => {}
                }
            }
            AgentType::Wolf(genotype) => {}
            AgentType::Grass() => {
                self.health += PLANT_GROWTH_RATE;
            }
        }

        match self.state {
            State::Idle => {
                if let AgentType::Sheep(_) = self.kind {
                    // Check if nearby wolf
                    self.wolf_fleeing_check(&nearby_agents, agents)
                }

                if self.hunger < MIN_HUNGER {
                    // Get nearby food
                    let mut closest = Uuid::nil();
                    let mut closest_distance = f32::MAX;
                    for nearby_agent in nearby_agents.iter() {
                        let agent = agents.get(&nearby_agent.0).unwrap();
                        match self.kind {
                            AgentType::Sheep(_) => match agent.kind {
                                AgentType::Grass() => {
                                    if !agent.dead {
                                        let distance = ((nearby_agent.1).0.powf(2.)
                                            + (nearby_agent.1).1.powf(2.))
                                            - (self.position.0.powf(2.) + self.position.1.powf(2.));
                                        if distance < closest_distance {
                                            closest = nearby_agent.0;
                                            closest_distance = distance;
                                        }
                                    }
                                }
                                _ => {}
                            },
                            AgentType::Wolf(_) => match agent.kind {
                                AgentType::Sheep(_) => {
                                    if !agent.dead {
                                        let distance = ((nearby_agent.1).0.powf(2.)
                                            + (nearby_agent.1).1.powf(2.))
                                            - (self.position.0.powf(2.) + self.position.1.powf(2.));
                                        if distance < closest_distance {
                                            closest = nearby_agent.0;
                                            closest_distance = distance;
                                        }
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    if closest != Uuid::nil() {
                        self.state = State::Hunting(closest);
                    }
                    return modified_agents;
                } else {
                    // If not hungry, try to reproduce
                    // TODO: Reproduction mechanics
                    for nearby_agent in nearby_agents.iter() {
                        let agent = agents.get(&nearby_agent.0).unwrap();
                    }
                }

                // Wander from time to time
                if time > self.last_time + self.timeout {
                    self.direction +=
                        (noise.get([self.seed, (time as f64) * 0.07]) as f32) * 2. - 1.;

                    self.acceleration.0 +=
                        self.direction.cos() * genotype.movement_speed * WANDER_SPEED;
                    self.acceleration.1 +=
                        self.direction.sin() * genotype.movement_speed * WANDER_SPEED;
                    self.last_time = time;
                }
            }
            State::Hunting(target) => {
                if let AgentType::Sheep(_) = self.kind {
                    self.wolf_fleeing_check(&nearby_agents, agents)
                }
                // Check if target is still nearby
                match nearby_agents.iter().find(|a| a.0 == target) {
                    Some(a) => {
                        let potential_prey = agents.get(&a.0);
                        if let None = potential_prey {
                            // Prey is no longer nearby, go back to idle
                            self.state = State::Idle;
                        } else {
                            // Found prey, continuing predator routine
                            let mut prey = potential_prey.unwrap().clone();
                            let prey_direction = normalize_vector((
                                prey.position.0 - self.position.0,
                                prey.position.1 - self.position.1,
                            ));
                            self.acceleration.0 = prey_direction.0;
                            self.acceleration.1 = prey_direction.1;
                            if (prey.position.0 - self.position.0).abs() < 4.
                                && (prey.position.1 - self.position.1).abs() < 4.
                            {
                                self.acceleration.0 = 0.;
                                self.acceleration.1 = 0.;

                                if time > self.last_time + self.timeout && !prey.dead {
                                    // Eat prey
                                    self.hunger += prey.eat(BITE_SIZE);
                                    if self.hunger > SATIETY {
                                        self.state = State::Idle;
                                    }
                                    if self.hunger > 100. {
                                        self.hunger = 100.
                                    }
                                    self.last_time = time;
                                    self.timeout = 1.;
                                }
                            }
                            modified_agents.insert(a.0, prey);
                        }
                    }
                    None => {
                        self.state = State::Idle;
                    }
                }
            }
            State::Dead => self.dead = true,
            _ => {}
        }

        /* self.health -= 1.; */

        if self.position.0 >= 1024. {
            self.acceleration.0 *= -1.;
        } else if self.position.0 <= 0. {
            self.acceleration.0 *= -1.;
        }
        if self.position.1 >= 1024. {
            self.acceleration.1 *= -1.;
        } else if self.position.1 <= 0. {
            self.acceleration.1 *= -1.;
        }

        modified_agents
    }

    pub fn get_closest_food(&self, agent_list: Vec<Agent>) -> Uuid {
        agent_list.iter().find(|a| a.kind.to_int() == 2).unwrap().id
    }

    pub fn eat(&mut self, bite: f32) -> f32 {
        match self.kind {
            AgentType::Wolf(_) => {
                panic!("Tried to eat non-food agent {}!", self.kind);
            }
            _ => {
                if self.health - bite < 0. {
                    self.dead = true;
                    let bitten = self.health % bite;
                    self.health = 0.;
                    return bitten;
                } else {
                    self.health -= bite;
                    return self.health;
                }
            }
        }
    }

    pub fn wolf_fleeing_check(
        &mut self,
        nearby_agents: &Vec<(Uuid, (f32, f32))>,
        agents: &HashMap<Uuid, Agent>,
    ) {
        for nearby_agent in nearby_agents.iter() {
            match agents.get(&nearby_agent.0).unwrap().kind {
                AgentType::Wolf(_) => {
                    self.state = State::Fleeing;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub enum State {
    Idle,
    Hunting(Uuid),
    Fleeing,
    Reproducing(Uuid),
    Dead,
}
impl State {
    pub fn to_int(&self) -> u8 {
        match self {
            State::Idle => 0,
            State::Hunting(_) => 1,
            State::Fleeing => 2,
            State::Reproducing(_) => 3,
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
            State::Reproducing(_) => write!(f, "Reproducing"),
            State::Dead => write!(f, "Dead"),
        }
    }
}
