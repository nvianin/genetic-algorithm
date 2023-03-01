use rand::{rngs::ThreadRng, Rng};
use std::{collections::HashMap};

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
        let body_size = thread_rng.gen_range(5.0..10.0);
        let sight_distance = thread_rng.gen_range(50.0..200.);
        let muscle_mass = thread_rng.gen_range(1.0..10.0);

        let mut genotype = Genotype {
            body_size,
            sight_distance,
            muscle_mass,
            // Derived variables
            hunger_rate: 0.,
            health_scale: 0.,
            movement_speed: 0.,
            gestation_duration: 0.,
        };
        genotype.derive_genotype();

        genotype
    }

    pub fn derive_genotype(&mut self) {
        self.hunger_rate = self.body_size * self.muscle_mass;
        self.health_scale = self.body_size * self.muscle_mass;
        self.movement_speed = self.muscle_mass / self.body_size;
        self.gestation_duration = self.body_size * self.muscle_mass;
    }

    pub fn crossbreed(&mut self, other: &Genotype, thread_rng: &mut ThreadRng) -> Genotype {
        let body_size = if thread_rng.gen_bool(0.5) {
            self.body_size
        } else {
            other.body_size
        };
        let sight_distance = if thread_rng.gen_bool(0.5) {
            self.sight_distance
        } else {
            other.sight_distance
        };
        let muscle_mass = if thread_rng.gen_bool(0.5) {
            self.muscle_mass
        } else {
            other.muscle_mass
        };

        let mut new_genotype = Genotype {
            body_size,
            sight_distance,
            muscle_mass,
            // Derived variables
            hunger_rate: 0.,
            health_scale: 0.,
            movement_speed: 0.,
            gestation_duration: 0.,
        };
        new_genotype.derive_genotype();

        new_genotype
    }

    pub fn mutate(&mut self, thread_rng: &mut ThreadRng) {
        if thread_rng.gen_range(0.0..1.0) < 0.1 {
            self.body_size = thread_rng.gen_range(1.0..10.0);
        }
        if thread_rng.gen_range(0.0..1.0) < 0.1 {
            self.sight_distance = thread_rng.gen_range(50.0..200.);
        }
        if thread_rng.gen_range(0.0..1.0) < 0.1 {
            self.muscle_mass = thread_rng.gen_range(1.0..10.0);
        }
        self.derive_genotype();
    }

    pub fn many_to_vec(geneotypes: Vec<&Genotype>) -> Vec<Vec<f32>> {
        let mut result = Vec::new();

        for genotype in geneotypes {
            result.push(vec![
                genotype.body_size,
                genotype.sight_distance,
                genotype.muscle_mass,
                genotype.hunger_rate,
                genotype.health_scale,
                genotype.movement_speed,
                genotype.gestation_duration,
            ]);
        }

        result
    }

    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.body_size,
            self.sight_distance,
            self.muscle_mass,
            self.hunger_rate,
            self.health_scale,
            self.movement_speed,
            self.gestation_duration,
        ]
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
