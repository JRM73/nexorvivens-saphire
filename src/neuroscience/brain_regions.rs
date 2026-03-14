// =============================================================================
// brain_regions.rs — Brain region network (replacement for the triune model)
// =============================================================================
//
// Role: Models 12 interconnected functional brain regions, each with its own
// neurotransmitter sensitivity, activation level, and weighted connections
// to other regions.
//
// This model replaces the simplistic "3 brains" (MacLean) with a distributed
// network based on modern neuroscience.
//
// Scientific references:
//   - Global Workspace Theory (Baars 1988, Dehaene 2014)
//   - Human connectome: Human Connectome Project (2013)
//   - Functional regions: Brodmann areas + modern fMRI
// =============================================================================

use serde::{Deserialize, Serialize};

/// Indices of the 12 brain regions.
/// Each constant corresponds to an index in the BrainNetwork arrays.
pub const AMYGDALA: usize = 0;       // Fear, emotional salience
pub const HIPPOCAMPUS: usize = 1;    // Episodic memory, spatial navigation
pub const PFC_DORSO: usize = 2;      // Dorsolateral prefrontal cortex: planning, WM
pub const PFC_VENTRO: usize = 3;     // Ventromedial prefrontal cortex: value, decision
pub const INSULA: usize = 4;         // Interoception, disgust, bodily awareness
pub const ACC: usize = 5;            // Anterior cingulate cortex: conflict, error
pub const BASAL_GANGLIA: usize = 6;  // Basal ganglia: habits, reward
pub const BRAINSTEM: usize = 7;      // Brainstem: arousal, sleep/wake
pub const OFC: usize = 8;            // Orbitofrontal cortex: subjective value
pub const TEMPORAL: usize = 9;       // Temporal cortex: language, semantics
pub const PARIETAL: usize = 10;      // Parietal cortex: spatial attention, integration
pub const CEREBELLUM: usize = 11;    // Cerebellum: timing, coordination, prediction

/// Total number of brain regions.
pub const NUM_REGIONS: usize = 12;

/// Region names (for display).
pub const REGION_NAMES: [&str; NUM_REGIONS] = [
    "Amygdale", "Hippocampe", "CPF-Dorso", "CPF-Ventro",
    "Insula", "CCA", "Noyaux-Base", "Tronc",
    "COF", "Temporal", "Parietal", "Cervelet",
];

/// Individual brain region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegion {
    /// Region name
    pub name: String,
    /// Current activation [0.0, 1.0]
    pub activation: f64,
    /// Previous cycle activation (for delta computation)
    pub prev_activation: f64,
    /// Sensitivity to each molecule [9 values: dopa, cort, sero, adre, oxyt, endo, nora, gaba, glut]
    /// Positive = excited by this molecule, negative = inhibited
    pub nt_sensitivity: [f64; 9],
    /// Minimum activation threshold to participate in the Global Workspace
    pub broadcast_threshold: f64,
}

/// Complete network of 12 brain regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainNetwork {
    /// The 12 regions
    pub regions: Vec<BrainRegion>,
    /// Connection matrix [12x12]: weights[i][j] = influence of region i on region j
    /// Positive = excitatory, negative = inhibitory
    pub weights: [[f64; NUM_REGIONS]; NUM_REGIONS],
    /// Global Workspace content: index of the dominant region
    pub workspace_winner: Option<usize>,
    /// Broadcast score (strength of the winning signal)
    pub workspace_strength: f64,
    /// Winner history (to measure variety)
    pub workspace_history: Vec<usize>,
    /// Global surprise: gap between prediction and reality
    pub global_surprise: f64,
}

impl Default for BrainNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainNetwork {
    /// Creates a brain network with default anatomical connections.
    pub fn new() -> Self {
        let regions = vec![
            // Amygdala: sensitive to cortisol and adrenaline (fear), inhibited by GABA
            BrainRegion {
                name: "Amygdale".into(), activation: 0.2, prev_activation: 0.2,
                nt_sensitivity: [0.1, 0.6, -0.3, 0.5, -0.2, -0.3, 0.3, -0.5, 0.2],
                broadcast_threshold: 0.4,
            },
            // Hippocampus: sensitive to serotonin (consolidation), degraded by cortisol
            BrainRegion {
                name: "Hippocampe".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.2, -0.4, 0.4, -0.1, 0.2, 0.1, 0.3, -0.1, 0.3],
                broadcast_threshold: 0.3,
            },
            // Dorsolateral PFC: focus (noradrenaline), degraded by cortisol
            BrainRegion {
                name: "CPF-Dorso".into(), activation: 0.4, prev_activation: 0.4,
                nt_sensitivity: [0.2, -0.5, 0.3, -0.2, 0.0, 0.1, 0.6, -0.1, 0.2],
                broadcast_threshold: 0.35,
            },
            // Ventromedial PFC: value, decision, sensitive to dopamine
            BrainRegion {
                name: "CPF-Ventro".into(), activation: 0.35, prev_activation: 0.35,
                nt_sensitivity: [0.5, -0.3, 0.3, -0.1, 0.2, 0.2, 0.2, -0.1, 0.1],
                broadcast_threshold: 0.35,
            },
            // Insula: interoception, sensitive to cortisol and endorphins
            BrainRegion {
                name: "Insula".into(), activation: 0.25, prev_activation: 0.25,
                nt_sensitivity: [0.1, 0.4, -0.1, 0.3, 0.1, -0.4, 0.2, -0.2, 0.2],
                broadcast_threshold: 0.4,
            },
            // ACC: conflict/error, sensitive to noradrenaline and dopamine
            BrainRegion {
                name: "CCA".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.3, 0.3, -0.1, 0.2, 0.0, -0.1, 0.5, -0.2, 0.3],
                broadcast_threshold: 0.35,
            },
            // Basal ganglia: habits/reward, highly sensitive to dopamine
            BrainRegion {
                name: "Noyaux-Base".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.8, -0.2, 0.1, 0.1, 0.0, 0.2, 0.1, -0.3, 0.1],
                broadcast_threshold: 0.3,
            },
            // Brainstem: arousal, sensitive to adrenaline and noradrenaline
            BrainRegion {
                name: "Tronc".into(), activation: 0.4, prev_activation: 0.4,
                nt_sensitivity: [0.1, 0.2, -0.2, 0.6, 0.0, -0.1, 0.5, -0.4, 0.3],
                broadcast_threshold: 0.3,
            },
            // OFC: subjective value, sensitive to dopamine and serotonin
            BrainRegion {
                name: "COF".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.5, -0.2, 0.4, -0.1, 0.3, 0.2, 0.1, -0.1, 0.1],
                broadcast_threshold: 0.35,
            },
            // Temporal: language, semantics, noradrenaline and glutamate
            BrainRegion {
                name: "Temporal".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.2, -0.2, 0.2, 0.0, 0.1, 0.1, 0.3, -0.1, 0.4],
                broadcast_threshold: 0.3,
            },
            // Parietal: spatial attention, multisensory integration
            BrainRegion {
                name: "Parietal".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.1, -0.1, 0.1, 0.2, 0.0, 0.1, 0.4, -0.2, 0.3],
                broadcast_threshold: 0.3,
            },
            // Cerebellum: timing, prediction, motor learning
            BrainRegion {
                name: "Cervelet".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.1, -0.1, 0.2, 0.1, 0.0, 0.1, 0.2, -0.3, 0.4],
                broadcast_threshold: 0.25,
            },
        ];

        // Anatomical connection matrix (simplified from HCP data)
        let mut weights = [[0.0f64; NUM_REGIONS]; NUM_REGIONS];

        // Amygdala -> PFC (top-down regulation), Hippocampus (emotional memory)
        weights[AMYGDALA][PFC_VENTRO] = 0.4;
        weights[AMYGDALA][HIPPOCAMPUS] = 0.5;
        weights[AMYGDALA][BRAINSTEM] = 0.6;  // fight-or-flight response
        weights[AMYGDALA][INSULA] = 0.3;     // bodily signals

        // Hippocampus <-> PFC (memory consolidation)
        weights[HIPPOCAMPUS][PFC_DORSO] = 0.4;
        weights[HIPPOCAMPUS][PFC_VENTRO] = 0.3;
        weights[HIPPOCAMPUS][TEMPORAL] = 0.5;  // semantic memory

        // Dorsolateral PFC -> amygdala inhibition (emotional regulation)
        weights[PFC_DORSO][AMYGDALA] = -0.3;
        weights[PFC_DORSO][ACC] = 0.4;
        weights[PFC_DORSO][BASAL_GANGLIA] = 0.3;
        weights[PFC_DORSO][PARIETAL] = 0.3;

        // Ventromedial PFC -> value, decision
        weights[PFC_VENTRO][OFC] = 0.5;
        weights[PFC_VENTRO][AMYGDALA] = -0.2;  // emotional regulation
        weights[PFC_VENTRO][BASAL_GANGLIA] = 0.4;

        // ACC -> conflict detection
        weights[ACC][PFC_DORSO] = 0.5;   // recruits PFC on conflict
        weights[ACC][AMYGDALA] = 0.2;    // emotional alert
        weights[ACC][BRAINSTEM] = 0.3;   // arousal

        // Basal ganglia -> cortico-basal loop
        weights[BASAL_GANGLIA][PFC_DORSO] = 0.3;
        weights[BASAL_GANGLIA][PFC_VENTRO] = 0.3;
        weights[BASAL_GANGLIA][CEREBELLUM] = 0.2;

        // Brainstem -> global arousal
        weights[BRAINSTEM][AMYGDALA] = 0.2;
        weights[BRAINSTEM][PFC_DORSO] = 0.3;
        weights[BRAINSTEM][PARIETAL] = 0.2;

        // OFC -> value
        weights[OFC][AMYGDALA] = 0.3;
        weights[OFC][BASAL_GANGLIA] = 0.4;

        // Insula -> bodily awareness
        weights[INSULA][ACC] = 0.4;
        weights[INSULA][AMYGDALA] = 0.3;

        // Cerebellum -> prediction
        weights[CEREBELLUM][PFC_DORSO] = 0.2;
        weights[CEREBELLUM][TEMPORAL] = 0.2;

        Self {
            regions,
            weights,
            workspace_winner: None,
            workspace_strength: 0.0,
            workspace_history: Vec::new(),
            global_surprise: 0.0,
        }
    }

    /// Updates region activations based on chemistry and sensory input.
    /// 1. Each region receives chemical influence via its NT sensitivity
    /// 1b. Active senses directly boost their anatomical regions
    /// 2. Inter-region connections propagate activation
    /// 3. The Global Workspace selects the dominant signal
    ///
    /// sensory_input: [reading, listening, contact, taste, ambiance] (intensities 0-1)
    pub fn tick(&mut self, chemistry: &crate::neurochemistry::NeuroChemicalState, sensory_input: [f64; 5]) {
        let chem9 = chemistry.to_vec9();

        // Phase 1: Direct chemical activation
        let mut new_activations = [0.0f64; NUM_REGIONS];
        for (i, region) in self.regions.iter().enumerate() {
            let mut chem_input = 0.0;
            for (j, &conc) in chem9.iter().enumerate() {
                chem_input += conc * region.nt_sensitivity[j];
            }
            new_activations[i] = region.activation * 0.7 + chem_input * 0.3;
        }

        // Phase 1b: Direct sensory boost (anatomically correct)
        // Moderate coefficient (0.08) to color activity without dominating it.
        // Each sense stimulates its natural anatomical regions:
        //   Reading (vision) -> Temporal (ventral stream) + Parietal (dorsal stream)
        //   Listening (hearing) -> Temporal (auditory cortex)
        //   Contact (touch) -> Insula (interoception) + Parietal (somatosensory)
        //   Taste (gustation) -> Insula (gustatory cortex) + OFC
        //   Ambiance (olfaction) -> OFC (olfactory cortex) + Amygdala (emotional valence)
        const SENS_COEFF: f64 = 0.08;
        let [reading, listening, contact, taste, ambiance] = sensory_input;

        new_activations[TEMPORAL]  += reading * SENS_COEFF * 0.7;
        new_activations[PARIETAL]  += reading * SENS_COEFF * 0.3;

        new_activations[TEMPORAL]  += listening * SENS_COEFF;

        new_activations[INSULA]    += contact * SENS_COEFF * 0.6;
        new_activations[PARIETAL]  += contact * SENS_COEFF * 0.4;

        new_activations[INSULA]    += taste * SENS_COEFF * 0.5;
        new_activations[OFC]       += taste * SENS_COEFF * 0.5;

        new_activations[OFC]       += ambiance * SENS_COEFF * 0.6;
        new_activations[AMYGDALA]  += ambiance * SENS_COEFF * 0.4;

        // Phase 2: Inter-region propagation
        let mut propagated = new_activations;
        for i in 0..NUM_REGIONS {
            let mut input = 0.0;
            for j in 0..NUM_REGIONS {
                if i != j {
                    input += new_activations[j] * self.weights[j][i];
                }
            }
            propagated[i] += input * 0.15; // inter-regional influence factor
        }

        // Phase 3: Normalization and update
        for (i, region) in self.regions.iter_mut().enumerate() {
            region.prev_activation = region.activation;
            region.activation = propagated[i].clamp(0.0, 1.0);
        }

        // Phase 4: Global Workspace — competition for broadcast
        self.compute_global_workspace();
    }

    /// Global Workspace Theory: the most active region above its broadcast
    /// threshold wins the competition and broadcasts its signal.
    pub fn compute_global_workspace(&mut self) {
        let mut best_idx = None;
        let mut best_score = 0.0;

        for (i, region) in self.regions.iter().enumerate() {
            if region.activation > region.broadcast_threshold {
                let score = region.activation;
                if score > best_score {
                    best_score = score;
                    best_idx = Some(i);
                }
            }
        }

        // Compute surprise: gap relative to prediction
        if let Some(prev_winner) = self.workspace_winner {
            if let Some(current_winner) = best_idx {
                self.global_surprise = if prev_winner != current_winner { 0.8 } else { 0.1 };
            } else {
                self.global_surprise = 0.5; // no winner = moderately surprising
            }
        }

        self.workspace_winner = best_idx;
        self.workspace_strength = best_score;

        // History (keep last 50)
        if let Some(idx) = best_idx {
            self.workspace_history.push(idx);
            if self.workspace_history.len() > 50 {
                self.workspace_history.remove(0);
            }
        }
    }

    /// Returns the name of the region dominating the Global Workspace.
    pub fn workspace_region_name(&self) -> &str {
        match self.workspace_winner {
            Some(idx) if idx < NUM_REGIONS => REGION_NAMES[idx],
            _ => "Aucune",
        }
    }

    /// Computes the Global Workspace diversity over recent cycles.
    /// 0.0 = always the same region, 1.0 = all regions represented.
    pub fn workspace_diversity(&self) -> f64 {
        if self.workspace_history.is_empty() { return 0.0; }
        let unique: std::collections::HashSet<&usize> = self.workspace_history.iter().collect();
        unique.len() as f64 / NUM_REGIONS as f64
    }

    /// Mapping to the legacy triune model for compatibility.
    /// Reptilian = average(Amygdala, Brainstem)
    /// Limbic = average(Hippocampus, Basal Ganglia, Insula)
    /// Neocortex = average(PFC-Dorso, PFC-Ventro, ACC, OFC, Temporal, Parietal)
    pub fn triune_compat(&self) -> [f64; 3] {
        let r = &self.regions;
        let reptilian = (r[AMYGDALA].activation + r[BRAINSTEM].activation) / 2.0;
        let limbic = (r[HIPPOCAMPUS].activation + r[BASAL_GANGLIA].activation + r[INSULA].activation) / 3.0;
        let neocortex = (r[PFC_DORSO].activation + r[PFC_VENTRO].activation
            + r[ACC].activation + r[OFC].activation
            + r[TEMPORAL].activation + r[PARIETAL].activation) / 6.0;
        [reptilian, limbic, neocortex]
    }

    /// Summary for the dashboard.
    pub fn summary(&self) -> BrainNetworkSummary {
        BrainNetworkSummary {
            activations: self.regions.iter().map(|r| (r.name.clone(), r.activation)).collect(),
            workspace_winner: self.workspace_region_name().to_string(),
            workspace_strength: self.workspace_strength,
            workspace_diversity: self.workspace_diversity(),
            global_surprise: self.global_surprise,
            triune_compat: self.triune_compat(),
        }
    }

    /// Serializes state for persistence.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "activations": self.regions.iter().map(|r| r.activation).collect::<Vec<_>>(),
            "workspace_winner": self.workspace_winner,
            "workspace_strength": self.workspace_strength,
            "global_surprise": self.global_surprise,
        })
    }

    /// Restores activations from persisted JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(activations) = json.get("activations").and_then(|v| v.as_array()) {
            for (i, val) in activations.iter().enumerate().take(NUM_REGIONS) {
                if let Some(a) = val.as_f64() {
                    if i < self.regions.len() {
                        self.regions[i].activation = a.clamp(0.0, 1.0);
                        self.regions[i].prev_activation = a.clamp(0.0, 1.0);
                    }
                }
            }
        }
    }
}

/// Brain network summary for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainNetworkSummary {
    pub activations: Vec<(String, f64)>,
    pub workspace_winner: String,
    pub workspace_strength: f64,
    pub workspace_diversity: f64,
    pub global_surprise: f64,
    /// Triune compatibility [reptilian, limbic, neocortex]
    pub triune_compat: [f64; 3],
}
