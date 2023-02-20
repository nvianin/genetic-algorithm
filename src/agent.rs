use std::fmt::Display;

use crate::genes::Genotype;
use crate::StateMachine;

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

#[derive(Clone)]
pub struct Agent {
    pub kind: AgentType,
    pub position: (f32, f32),
    pub acceleration: (f32, f32),
    pub id: Uuid,
    pub health: f32,
    pub hunger: f32,
    pub life: f32,
    pub state_machine: StateMachine,
    pub dead: bool,
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
            life: 100.,
            state_machine: StateMachine::new(),
            dead: false,
        }
    }

    pub fn update(&mut self, agent_list: Vec<Agent>) {
        self.state_machine.update(self);
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
