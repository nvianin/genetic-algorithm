use std::{f32::MIN, fmt::Display};

use crate::genes::Genotype;

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

    pub fn update(&mut self, nearby_agents: &mut Vec<&Agent>) {
        self.position.0 + self.acceleration.0;
        self.position.1 + self.acceleration.1;

        self.acceleration.0 *= 0.9;
        self.acceleration.1 *= 0.9;

        match self.state {
            State::Idle => {
                if self.hunger < MIN_HUNGER {
                    // Get nearby food
                    nearby_agents.iter().find(|a| match a.kind {
                        AgentType::Grass() => true,
                        _ => false,
                    });
                }
            }
            State::Hunting(target) => {
                match nearby_agents.iter_mut().find(|a| a.id == target) {
                    Some(a) => {
                        // Found prey, continuing predator routine
                        self.acceleration.0 += a.position.0 - self.position.0;
                        self.acceleration.1 += a.position.1 - self.position.1;
                        if (a.position.0 - self.position.0).abs() < 10.
                            && (a.position.1 - self.position.1).abs() < 10.
                        {
                            // Eat prey
                            self.hunger += a.eat(10.);
                            if self.hunger > MIN_HUNGER {
                                self.state = State::Idle;
                            }
                        }
                    }
                    None => {}
                }
            }
            State::Fleeing => {
                match self.kind {
                    AgentType::Sheep(_) => {
                        // Get nearby wolves
                        match nearby_agents.iter().find(|a| match a.kind {
                            AgentType::Wolf(_) => true,
                            _ => false,
                        }) {
                            Some(a) => {
                                // Found predator, fleeing
                                self.acceleration.0 += self.position.0 - a.position.0;
                                self.acceleration.1 += self.position.1 - a.position.1;
                            }
                            None => {
                                // No predators nearby, return to idle
                                self.state = State::Idle;
                            }
                        }
                    }
                    _ => {}
                }
            }
            State::Eating(target) => {}
        }
        self.hunger -= 1.;
        /* self.health -= 1.; */
        if self.health <= 0. {
            self.dead = true;
        }
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
                return self.health - bite;
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
