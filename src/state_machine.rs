use uuid::Uuid;

use crate::Agent;
#[derive(Clone)]
pub struct StateMachine {
    pub state: State,
    pub state_stack: Vec<State>,
}

impl StateMachine {
    pub fn new() -> StateMachine {
        StateMachine {
            state: State::Idle,
            state_stack: Vec::new(),
        }
    }

    pub fn update(&mut self, agent: &mut Agent) {
        match self.state {
            State::Idle => {
                if agent.hunger < 0. {
                    self.state = State::Hunting(agent.get_closest_food());
                }
            }
            State::Hunting(target) => {
                
            }
            State::Fleeing => {
                if agent.hunger > 0. {
                    self.state = State::Hunting(agent.get_closest_food());
                }
            }
            State::Eating(target) => {
                if agent.hunger <= 0. {
                    self.state = State::Idle;
                } else {
                    if agent.position.distance(target) < 0.1 {
                        agent.eat(target);
                    } else {
                        agent.set_target(target);
                    }
                }
            }
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
