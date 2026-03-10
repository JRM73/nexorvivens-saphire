// simulation/ — Stub for the lite edition
// Behavior trees, FSM, flocking, steering not ported.

pub mod influence_map {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct InfluenceMap;

    impl InfluenceMap {
        pub fn update_from_cognition(
            &mut self, _emotion: &str, _cortisol: f64, _dopamine: f64, _noradrenaline: f64,
        ) {}
        pub fn tick(&mut self) {}
        pub fn describe_for_prompt(&self) -> String { String::new() }
        pub fn to_status_json(&self) -> serde_json::Value { serde_json::json!({}) }
        pub fn to_json(&self) -> serde_json::Value { serde_json::json!({}) }
    }
}

pub mod cognitive_fsm {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CognitiveFsm {
        pub state: String,
    }

    impl CognitiveFsm {
        pub fn new() -> Self { Self { state: "Eveil".into() } }
        pub fn tick(
            &mut self, _cortisol: f64, _dopamine: f64, _serotonin: f64,
            _noradrenaline: f64, _endorphin: f64,
        ) {}
        pub fn describe_for_prompt(&self) -> String { String::new() }
        pub fn to_status_json(&self) -> serde_json::Value { serde_json::json!({}) }
        pub fn to_json(&self) -> serde_json::Value { serde_json::json!({}) }
    }
}

pub mod steering {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct EmotionalPos {
        pub valence: f64,
        pub arousal: f64,
    }

    impl EmotionalPos {
        pub fn new(valence: f64, arousal: f64) -> Self { Self { valence, arousal } }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct SteeringForce {
        pub dx: f64,
        pub dy: f64,
    }

    pub struct ChemistryAdjustment {
        pub dopamine: f64,
        pub cortisol: f64,
        pub serotonin: f64,
        pub adrenaline: f64,
        pub noradrenaline: f64,
        pub endorphin: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct SteeringParams {
        pub seek_weight: f64,
        pub flee_weight: f64,
        pub wander_weight: f64,
        pub equilibrium: EmotionalPos,
        pub flee_radius: f64,
        pub arrive_radius: f64,
        pub wander_strength: f64,
        pub max_force: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct SteeringEngine {
        pub params: SteeringParams,
    }

    impl SteeringEngine {
        pub fn compute_regulation(
            &self, _pos: &EmotionalPos, _cycle: u64, _cortisol: f64,
        ) -> SteeringForce {
            SteeringForce::default()
        }

        pub fn force_to_chemistry(&self, _force: &SteeringForce) -> ChemistryAdjustment {
            ChemistryAdjustment {
                dopamine: 0.0, cortisol: 0.0, serotonin: 0.0,
                adrenaline: 0.0, noradrenaline: 0.0, endorphin: 0.0,
            }
        }

        pub fn to_status_json(&self) -> serde_json::Value { serde_json::json!({}) }
    }
}
