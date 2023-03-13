use rand::{rngs::ThreadRng, Rng};
use std::{collections::HashMap};

#[derive(Clone, Copy, PartialEq)]
pub struct Genotype {
    // Genotype Variables
    pub body_size: f32,
    pub sight_distance: f32,
    pub muscle_mass: f32,
    pub reproduction_chance: f32,
    // Derived Variables
    pub hunger_rate: f32,
    pub health_scale: f32,
    pub movement_speed: f32,
    pub gestation_duration: f32,
}

const BODY_SIZE_RANGE: std::ops::Range<f32> = 5.0..10.0;
const SIGHT_DISTANCE_RANGE: std::ops::Range<f32> = 50.0..200.;
const MUSCLE_MASS_RANGE: std::ops::Range<f32> = 1.0..10.0;
const REPRODUCTION_CHANCE_RANGE: std::ops::Range<f32> = 0.0..1.0;

const MUTATION_RATE: f32 = 0.05;

impl Genotype {
    pub fn new(thread_rng: &mut ThreadRng) -> Genotype {
        let body_size = thread_rng.gen_range(BODY_SIZE_RANGE);
        let sight_distance = thread_rng.gen_range(SIGHT_DISTANCE_RANGE);
        let muscle_mass = thread_rng.gen_range(MUSCLE_MASS_RANGE);
        let reproduction_chance = thread_rng.gen_range(REPRODUCTION_CHANCE_RANGE);

        let mut genotype = Genotype {
            body_size,
            sight_distance,
            muscle_mass,
            reproduction_chance,
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
        let reproduction_chance = if thread_rng.gen_bool(0.5) {
            self.reproduction_chance
        } else {
            other.reproduction_chance
        };

        let mut new_genotype = Genotype {
            body_size,
            sight_distance,
            muscle_mass,
            reproduction_chance,
            // Derived variables
            hunger_rate: 0.,
            health_scale: 0.,
            movement_speed: 0.,
            gestation_duration: 0.,
        };
        new_genotype.derive_genotype();
        new_genotype.mutate(thread_rng);

        new_genotype
    }

    pub fn mutate(&mut self, thread_rng: &mut ThreadRng) {
        self.body_size += (thread_rng.gen::<f32>() * 2. - 1.) * MUTATION_RATE;
        self.sight_distance += (thread_rng.gen::<f32>() * 2. - 1.) * MUTATION_RATE;
        self.muscle_mass += (thread_rng.gen::<f32>() * 2. - 1.) * MUTATION_RATE;
        self.reproduction_chance += (thread_rng.gen::<f32>() * 2. - 1.) * MUTATION_RATE;

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
            self.reproduction_chance,
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
