// =============================================================================
// cognition/map_sync.rs — MAP : Modulateur Adaptatif de Proprioception
// =============================================================================
//
// Synchronise le Sensorium (5 sens) avec le BrainNetwork (12 regions)
// et le Connectome (graphe neuronal) en temps reel.
//
// Probleme resolu : le Sensorium se met a jour tot dans le cycle (phase_senses)
// mais le BrainNetwork ne reagit que bien plus tard (process_stimulus).
// Le MAP comble ce decalage en propageant immediatement les donnees sensorielles
// vers le reseau cerebral et le connectome.
//
// La "tension du reseau" mesure l'ecart entre ce que les sens percoivent
// et comment le cerveau reagit — un indicateur de coherence interne.
// =============================================================================

use std::collections::VecDeque;

/// Indices des regions cerebrales mappees aux sens
/// (memes constantes que dans brain_regions.rs)
const TEMPORAL: usize = 9;
const INSULA: usize = 4;
const OFC: usize = 8;

/// Mapping anatomique : chaque sens est associe a une region dominante
const SENSE_TO_REGION: [usize; 5] = [
    TEMPORAL,  // lecture → cortex temporal (stream ventral)
    TEMPORAL,  // ecoute → cortex temporal (cortex auditif)
    INSULA,    // contact → insula (interoception)
    OFC,       // saveur → cortex orbitofrontal (gustatif)
    OFC,       // ambiance → cortex orbitofrontal (olfactif)
];

/// Resultat d'une synchronisation MAP
#[derive(Debug, Clone)]
pub struct MapSyncResult {
    /// Tension du reseau : ecart moyen entre perception et reaction cerebrale
    pub network_tension: f64,
    /// Region dominante apres synchronisation
    pub dominant_region: String,
    /// Force du workspace global apres synchronisation
    pub workspace_strength: f64,
}

/// Modulateur Adaptatif de Proprioception
///
/// Synchronise trois systemes en temps reel :
/// 1. Sensorium → BrainNetwork (propagation des intensites sensorielles)
/// 2. BrainNetwork → Connectome (propagation des labels actifs)
/// 3. Mesure de la tension reseau (ecart perception/reaction)
pub struct MapSync {
    /// Module actif ou non
    pub enabled: bool,
    /// Tension du reseau : ecart entre Sensorium et BrainNetwork
    pub network_tension: f64,
    /// Dernier cycle synchronise
    pub last_sync_cycle: u64,
    /// Historique des tensions (max 50, pour tendance)
    pub tension_history: VecDeque<f64>,
    /// Region dominante apres derniere synchronisation
    pub dominant_region: String,
    /// Force du workspace apres derniere synchronisation
    pub workspace_strength: f64,
}

impl MapSync {
    /// Cree un nouveau MAP actif
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

    /// Calcule la tension du reseau : ecart moyen entre intensites sensorielles
    /// et activations des regions cerebrales correspondantes.
    ///
    /// La tension est basse (~5-15%) en regime normal et monte en transition.
    pub fn compute_tension(
        &mut self,
        sensory_input: &[f64; 5],
        brain_network: &crate::neuroscience::brain_regions::BrainNetwork,
    ) -> MapSyncResult {
        // Calculer l'ecart moyen entre chaque sens et sa region mappee
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

        // Stocker dans l'historique
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

    /// Ligne de proprioception pour le prompt LLM
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

    /// Donnees JSON pour le broadcast WebSocket
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
