// =============================================================================
// cognition/map_sync.rs — MAP: Adaptive Proprioception Modulator
// =============================================================================
//
// Synchronizes the Sensorium (5 senses) with the BrainNetwork (12 regions)
// and the Connectome (neural graph) in real time.
//
// Problem solved: the Sensorium updates early in the cycle (phase_senses)
// but the BrainNetwork only reacts much later (process_stimulus).
// The MAP bridges this gap by immediately propagating sensory data
// to the brain network and the connectome.
//
// "Network tension" measures the gap between what the senses perceive
// and how the brain reacts — an indicator of internal coherence.
// =============================================================================

use std::collections::VecDeque;

/// Brain region indices mapped to senses
/// (same constants as in brain_regions.rs)
const TEMPORAL: usize = 9;
const INSULA: usize = 4;
const OFC: usize = 8;

/// Anatomical mapping: each sense is associated with a dominant region
const SENSE_TO_REGION: [usize; 5] = [
    TEMPORAL,  // reading -> temporal cortex (ventral stream)
    TEMPORAL,  // listening -> temporal cortex (auditory cortex)
    INSULA, // touch -> insula (interoception)    OFC, // taste -> orbitofrontal cortex (gustatory)    OFC,       // ambiance -> orbitofrontal cortex (olfactory)
];

/// Result of a MAP synchronization
#[derive(Debug, Clone)]
pub struct MapSyncResult {
    /// Network tension: average gap between perception and brain response
    pub network_tension: f64,
    /// Dominant region after synchronization
    pub dominant_region: String,
    /// Overall workspace strength after synchronization
    pub workspace_strength: f64,
}

/// Adaptive Proprioception Modulator
///
/// Synchronizes three systems in real time:
/// 1. Sensorium -> BrainNetwork (sensory intensity propagation)
/// 2. BrainNetwork -> Connectome (active label propagation)
/// 3. Network tension measurement (perception/reaction gap)
pub struct MapSync {
    /// Module enabled or not
    pub enabled: bool,
    /// Network tension: gap between Sensorium and BrainNetwork
    pub network_tension: f64,
    /// Last synchronized cycle
    pub last_sync_cycle: u64,
    /// Tension history (max 50, for trend analysis)
    pub tension_history: VecDeque<f64>,
    /// Dominant region after last synchronization
    pub dominant_region: String,
    /// Workspace strength after last synchronization
    pub workspace_strength: f64,
}

impl MapSync {
    /// Creates a new active MAP
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            network_tension: 0.0,
            last_sync_cycle: 0,
            tension_history: VecDeque::with_capacity(50),
            dominant_region: String::new(),
            workspace_strength: 0.0,
        }
    }

    /// Computes the network tension: average gap between sensory intensities
    /// and activations of corresponding brain regions.
    ///
    /// Tension is low (~5-15%) in normal regime and rises during transitions.
    pub fn compute_tension(
        &mut self,
        sensory_input: &[f64; 5],
        brain_network: &crate::neuroscience::brain_regions::BrainNetwork,
    ) -> MapSyncResult {
        // Compute the average gap between each sense and its mapped region
        let mut total_diff = 0.0;
        let mut count = 0;
        for (i, &intensity) in sensory_input.iter().enumerate() {
            let region_idx = SENSE_TO_REGION[i];
            if region_idx < brain_network.regions.len() {
                let activation = brain_network.regions[region_idx].activation;
                total_diff += (intensity - activation).abs();
                count += 1;
            }
        }

        let tension = if count > 0 { total_diff / count as f64 } else { 0.0 };

        // Store in history
        self.tension_history.push_back(tension);
        if self.tension_history.len() > 50 {
            self.tension_history.pop_front();
        }

        self.network_tension = tension;
        self.dominant_region = brain_network.workspace_region_name().to_string();
        self.workspace_strength = brain_network.workspace_strength;

        MapSyncResult {
            network_tension: tension,
            dominant_region: self.dominant_region.clone(),
            workspace_strength: self.workspace_strength,
        }
    }

    /// Proprioception line for the LLM prompt
    pub fn proprioception_line(&self) -> String {
        if !self.enabled {
            return String::new();
        }
        let tension_desc = if self.network_tension > 0.5 { "forte" }
            else if self.network_tension > 0.25 { "moderee" }
            else { "faible" };
        let focus_desc = if self.workspace_strength > 0.6 { "concentree" }
            else if self.workspace_strength > 0.3 { "diffuse" }
            else { "dispersee" };
        format!(
            "Tension cerebrale {} ({:.0}%), pensee {} ({})",
            tension_desc,
            self.network_tension * 100.0,
            focus_desc,
            self.dominant_region,
        )
    }

    /// JSON data for WebSocket broadcast
    pub fn to_broadcast_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "network_tension": self.network_tension,
            "dominant_region": self.dominant_region,
            "workspace_strength": self.workspace_strength,
            "last_sync_cycle": self.last_sync_cycle,
        })
    }
}
