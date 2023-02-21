use std::{collections::HashMap, f32::MIN, fmt::Display};

use crate::genes::{self, Genotype};

use uuid::Uuid;

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
const HUNGER_RATE: f32 = 0.05;
const BITE_SIZE: f32 = 10.;

#[derive(Clone)]
pub struct Agent {
    pub kind: AgentType,
    pub position: (f32, f32),
    pub acceleration: (f32, f32),
    pub id: Uuid,
    pub health: f32,
    pub hunger: f32,
    pub life: f32,
    pub dead: bool,
    pub state: State,
}

impl Agent {
    pub fn new(kind: AgentType, position: (f32, f32), id: Uuid) -> Agent {
        Agent {
            kind,
            position,
            acceleration: (0., 0.),
            id,
            health: 100.,
            hunger: MIN_HUNGER,
            life: 100.,
            dead: false,
            state: State::Idle,
        }
    }

    pub fn update(
        &mut self,
        nearby_agents: Vec<(Uuid, (f32, f32))>,
        agents: &HashMap<Uuid, Agent>,
    ) -> HashMap<Uuid, Agent> {
        self.position.0 + self.acceleration.0;
        self.position.1 + self.acceleration.1;

        self.acceleration.0 *= 0.9;
        self.acceleration.1 *= 0.9;
        match self.kind {
            AgentType::Sheep(genotype) => {
                match self.state {
                    State::Idle => {
                        for id in nearby_agents.iter() {
                            match agents.get(id).unwrap().kind {
                                AgentType::Wolf(_) => {
                                    self.state = State::Fleeing;
                                }
                                _ => {}
                            }
                        }

                        if self.hunger < MIN_HUNGER {
                            // Get nearby food
                            for id in nearby_agents.iter() {
                                match agents.get(id).unwrap().kind {
                                    AgentType::Grass() => {
                                        self.state = State::Hunting(*id);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    State::Hunting(target) => {
                        // Check if target is still nearby
                        match nearby_agents.iter().find(|a| **a == target) {
                            Some(a) => {
                                let prey = agents.get_mut(a).unwrap();
                                // Found prey, continuing predator routine
                                self.acceleration.0 += prey.position.0 - self.position.0;
                                self.acceleration.1 += prey.position.1 - self.position.1;
                                if (prey.position.0 - self.position.0).abs() < 1.
                                    && (prey.position.1 - self.position.1).abs() < 1.
                                {
                                    // Eat prey
                                    let (bitten, remaining) = prey.eat(BITE_SIZE);
                                    self.hunger += bitten;
                                    prey.health = remaining;
                                    if prey.health <= 0. {
                                        prey.dead = true;
                                    }
                                    if self.hunger > MIN_HUNGER {
                                        self.state = State::Idle;
                                    }
                                }
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
                                    match agents.get(id).unwrap().kind {
                                        AgentType::Wolf(_) => {
                                            let pos = agents.get(id).unwrap().position;
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
                }
                self.hunger -= HUNGER_RATE * genotype.hunger_rate;
            }
            AgentType::Wolf(genotype) => {}
            AgentType::Grass() => {}
        }

        /* self.health -= 1.; */
        if self.health <= 0. {
            self.dead = true;
        }
    }

    pub fn get_closest_food(&self, agent_list: Vec<Agent>) -> Uuid {
        agent_list.iter().find(|a| a.kind.to_int() == 2).unwrap().id
    }

    /// Takes a bite from the agent and returns the bite taken and the remaining health;
    pub fn eat(&self, bite: f32) -> (f32, f32) {
        if let AgentType::Grass() = self.kind {
            if self.health - bite < 0. {
                return (self.health % bite, self.health - bite);
            } else {
                return (self.health - bite, self.health - bite);
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
}
