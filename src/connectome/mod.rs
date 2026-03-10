// connectome/ — Stub for the lite edition

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Module(String),
    Molecule(String),
    Emotion(String),
    Memory(String),
    Sense(String),
    Concept,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Excitatory,
    Inhibitory,
    Modulatory,
    Associative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub average_strength: f64,
    pub total_synaptic_strength: f64,
    pub plasticity: f64,
    pub strongest_edge: Option<String>,
    pub most_connected_node: Option<String>,
}

impl Default for ConnectomeMetrics {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            total_edges: 0,
            average_strength: 0.0,
            total_synaptic_strength: 0.0,
            plasticity: 1.0,
            strongest_edge: None,
            most_connected_node: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connectome {
    pub learning_rate: f64,
    pub pruning_threshold: f64,
    pub max_edges: usize,
    pub pruning_interval_cycles: u64,
    pub plasticity: f64,
}

impl Connectome {
    pub fn new(
        learning_rate: f64,
        pruning_threshold: f64,
        max_edges: usize,
        pruning_interval_cycles: u64,
    ) -> Self {
        Self {
            learning_rate,
            pruning_threshold,
            max_edges,
            pruning_interval_cycles,
            plasticity: 1.0,
        }
    }

    pub fn tick(&mut self, _labels: &[&str]) {
        // stub
    }

    pub fn metrics(&self) -> ConnectomeMetrics {
        ConnectomeMetrics::default()
    }

    pub fn add_node(&mut self, _label: &str, _node_type: NodeType) -> usize {
        0 // stub
    }

    pub fn add_edge(&mut self, _from: usize, _to: usize, _strength: f64, _edge_type: EdgeType) {
        // stub
    }

    pub fn associative_chain(&self, _from: &str, _to: &str, _max_depth: usize) -> Option<Vec<(String, f64)>> {
        None // stub
    }

    pub fn spreading_activation(&self, _seed: &str, _depth: usize) -> Vec<(String, f64)> {
        Vec::new() // stub
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "learning_rate": self.learning_rate,
            "plasticity": self.plasticity,
            "metrics": self.metrics(),
        })
    }
}
