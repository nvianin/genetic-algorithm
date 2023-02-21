use rand::{rngs::ThreadRng, Rng};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Genotype {
    // Genotype Variables
    pub body_size: f32,
    pub sight_distance: f32,
    pub muscle_mass: f32,
    // Derived Variables
    pub hunger_rate: f32,
    pub health_scale: f32,
    pub movement_speed: f32,
    pub gestation_duration: f32,
}
impl Genotype {
    pub fn new(thread_rng: &mut ThreadRng) -> Genotype {
        let body_size = thread_rng.gen_range(0.1..0.9);
        let sight_distance = thread_rng.gen_range(0.1..0.9);
        let muscle_mass = thread_rng.gen_range(0.1..0.9);
        Genotype {
            body_size,
            sight_distance,
            muscle_mass,
            // Derived variables
            hunger_rate: body_size * muscle_mass,
            health_scale: body_size * muscle_mass,
            movement_speed: muscle_mass / body_size,
            gestation_duration: body_size * muscle_mass,
        }
    }

    pub fn to_hashmap(&self) -> HashMap<String, f32> {
        let mut map = HashMap::new();
        map.insert("body_size".to_string(), self.body_size);
        map.insert("sight_distance".to_string(), self.sight_distance);
        map.insert("muscle_mass".to_string(), self.muscle_mass);

        map.insert("hunger_rate".to_string(), self.hunger_rate);
        map.insert("health_scale".to_string(), self.health_scale);
        map.insert("movement_speed".to_string(), self.movement_speed);
        map.insert("gestation_duration".to_string(), self.gestation_duration);

        map
    }
}
