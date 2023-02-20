use rand::{rngs::ThreadRng, Rng};

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
}
