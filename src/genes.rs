#[derive(Clone)]
pub struct WolfGeneticInformation {
    pub movement_speed: f32,
    pub sight_distance: f32,
}

impl WolfGeneticInformation {
    pub fn default() -> WolfGeneticInformation {
        WolfGeneticInformation {
            movement_speed: 1.,
            sight_distance: 1.,
        }
    }
}

#[derive(Clone)]
pub struct SheepGeneticInformation {
    pub movement_speed: f32,
    pub sight_distance: f32,
}

impl SheepGeneticInformation {
    pub fn default() -> SheepGeneticInformation {
        SheepGeneticInformation {
            movement_speed: 1.5,
            sight_distance: 1.,
        }
    }
}
