// =============================================================================
// memory/reconsolidation.rs — Reconsolidation memorielle (Nader 2000)
// =============================================================================
//
// Role : Implemente la reconsolidation des souvenirs. Chaque rappel d'un
// souvenir le rend temporairement labile — il peut etre modifie par
// l'etat emotionnel courant avant d'etre re-stabilise.
//
// References scientifiques :
//   - Nader, Schafe & LeDoux (2000) : "Fear memories require protein
//     synthesis in the amygdala for reconsolidation after retrieval"
//   - Ebbinghaus (1885) : courbe d'oubli exponentielle
//   - Anderson (2003) : interference retroactive et proactive
// =============================================================================

use serde::{Deserialize, Serialize};

/// Parametres de reconsolidation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationParams {
    /// Taux de modification lors du rappel [0.0, 0.3]
    /// Plus eleve = le souvenir est plus modifie par l'etat courant
    pub modification_rate: f64,
    /// Duree de labilite en cycles (window de reconsolidation)
    pub lability_window: u64,
    /// Taux d'interference entre souvenirs similaires [0.0, 0.5]
    pub interference_rate: f64,
    /// Constante de la courbe d'Ebbinghaus (vitesse d'oubli)
    /// Plus grande = oubli plus rapide
    pub ebbinghaus_decay_constant: f64,
    /// Facteur de renforcement emotionnel
    /// Les souvenirs emotionnels sont mieux retenus
    pub emotional_retention_factor: f64,
}

impl Default for ReconsolidationParams {
    fn default() -> Self {
        Self {
            modification_rate: 0.1,
            lability_window: 20,    // ~20 cycles de labilite
            interference_rate: 0.05,
            ebbinghaus_decay_constant: 0.3,
            emotional_retention_factor: 1.5,
        }
    }
}

/// Etat de reconsolidation d'un souvenir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationState {
    /// Le souvenir est-il actuellement labile ?
    pub is_labile: bool,
    /// Cycles restants de labilite
    pub lability_remaining: u64,
    /// Nombre de fois que le souvenir a ete rappele
    pub recall_count: u64,
    /// Poids emotionnel original (avant modification)
    pub original_emotional_weight: f64,
    /// Derive cumulative depuis l'original (mesure de distorsion)
    pub cumulative_drift: f64,
}

impl Default for ReconsolidationState {
    fn default() -> Self {
        Self {
            is_labile: false,
            lability_remaining: 0,
            recall_count: 0,
            original_emotional_weight: 0.0,
            cumulative_drift: 0.0,
        }
    }
}

/// Moteur de reconsolidation — gere la labilite et la modification des souvenirs.
#[derive(Debug, Clone)]
pub struct ReconsolidationEngine {
    pub params: ReconsolidationParams,
    /// Souvenirs actuellement labiles (memory_id → state)
    pub labile_memories: std::collections::HashMap<String, ReconsolidationState>,
}

impl Default for ReconsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconsolidationEngine {
    pub fn new() -> Self {
        Self {
            params: ReconsolidationParams::default(),
            labile_memories: std::collections::HashMap::new(),
        }
    }

    /// Appele lors du rappel d'un souvenir.
    /// Rend le souvenir labile et calcule la modification emotionnelle.
    ///
    /// Retourne le delta de poids emotionnel a appliquer au souvenir.
    pub fn on_recall(
        &mut self,
        memory_id: &str,
        current_emotional_weight: f64,
        current_valence: f64,
        current_arousal: f64,
    ) -> ReconsolidationEffect {
        let state = self.labile_memories
            .entry(memory_id.to_string())
            .or_insert_with(|| ReconsolidationState {
                original_emotional_weight: current_emotional_weight,
                ..Default::default()
            });

        state.recall_count += 1;
        state.is_labile = true;
        state.lability_remaining = self.params.lability_window;

        // Le rappel renforce le souvenir (testing effect)
        let reinforcement = 0.02 * (state.recall_count as f64).ln().max(0.1);

        // Mais l'etat emotionnel courant colore le souvenir (reconsolidation)
        // La valence courante "contamine" le souvenir
        let emotional_contamination = current_valence * self.params.modification_rate;
        let arousal_effect = (current_arousal - 0.5) * self.params.modification_rate * 0.5;

        // La derive cumule
        state.cumulative_drift += emotional_contamination.abs() * 0.1;

        ReconsolidationEffect {
            strength_delta: reinforcement,
            emotional_weight_delta: emotional_contamination + arousal_effect,
            became_labile: true,
            recall_count: state.recall_count,
            cumulative_drift: state.cumulative_drift,
        }
    }

    /// Tick : avancer les timers de labilite.
    /// Appelee a chaque cycle cognitif.
    pub fn tick(&mut self) {
        let mut to_remove = Vec::new();
        for (id, state) in &mut self.labile_memories {
            if state.lability_remaining > 0 {
                state.lability_remaining -= 1;
            }
            if state.lability_remaining == 0 {
                state.is_labile = false;
                // Le souvenir est re-stabilise — garder l'etat pour tracking
                // mais retirer de la map active apres un moment
                if state.recall_count > 0 && !state.is_labile {
                    to_remove.push(id.clone());
                }
            }
        }
        // Nettoyer les souvenirs stabilises anciens (garder les 100 plus recents)
        if self.labile_memories.len() > 100 {
            for id in to_remove {
                self.labile_memories.remove(&id);
            }
        }
    }

    /// Courbe d'oubli d'Ebbinghaus : retention = e^(-t/S)
    /// ou t = temps depuis l'encodage, S = stabilite du souvenir.
    ///
    /// La stabilite augmente avec :
    /// - Le nombre de rappels (testing effect)
    /// - Le poids emotionnel (souvenirs emotionnels durent plus)
    /// - L'espacement des rappels (spacing effect)
    pub fn ebbinghaus_retention(
        &self,
        cycles_since_encoding: u64,
        recall_count: u64,
        emotional_weight: f64,
    ) -> f64 {
        // Stabilite de base
        let base_stability = 1.0 / self.params.ebbinghaus_decay_constant;

        // Bonus de stabilite par rappel (testing effect + spacing)
        let recall_bonus = (recall_count as f64).sqrt() * 2.0;

        // Bonus emotionnel (les souvenirs emotionnels sont mieux retenus)
        let emotional_bonus = emotional_weight.abs() * self.params.emotional_retention_factor;

        let total_stability = base_stability + recall_bonus + emotional_bonus;

        // Courbe d'Ebbinghaus : retention = e^(-t/S)
        let t = cycles_since_encoding as f64;
        (-t / total_stability.max(1.0)).exp()
    }

    /// Calcule l'interference entre deux souvenirs similaires.
    /// L'interference retroactive (nouveau souvenir degrade l'ancien)
    /// et proactive (ancien souvenir interfere avec le nouveau).
    ///
    /// Retourne le facteur d'affaiblissement [0.0, 1.0] (1.0 = pas d'interference).
    pub fn compute_interference(
        &self,
        similarity: f64,
        is_retroactive: bool,
    ) -> f64 {
        if similarity < 0.5 {
            return 1.0; // Pas assez similaires pour interferer
        }

        let interference_strength = (similarity - 0.5) * 2.0 * self.params.interference_rate;
        let direction_factor = if is_retroactive { 1.0 } else { 0.7 }; // Retroactive plus forte
        let weakening = 1.0 - (interference_strength * direction_factor);
        weakening.clamp(0.5, 1.0) // Jamais plus de 50% d'affaiblissement
    }
}

/// Effet de la reconsolidation sur un souvenir.
#[derive(Debug, Clone)]
pub struct ReconsolidationEffect {
    /// Delta de force du souvenir (positif = renforcement par le rappel)
    pub strength_delta: f64,
    /// Delta de poids emotionnel (contamination par l'etat courant)
    pub emotional_weight_delta: f64,
    /// Le souvenir est devenu labile
    pub became_labile: bool,
    /// Nombre total de rappels
    pub recall_count: u64,
    /// Derive cumulative depuis l'original
    pub cumulative_drift: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebbinghaus_curve() {
        let engine = ReconsolidationEngine::new();
        let ret_0 = engine.ebbinghaus_retention(0, 0, 0.0);
        let ret_100 = engine.ebbinghaus_retention(100, 0, 0.0);
        let ret_1000 = engine.ebbinghaus_retention(1000, 0, 0.0);
        assert!(ret_0 > ret_100, "Retention diminue avec le temps");
        assert!(ret_100 > ret_1000, "Retention continue de diminuer");
        assert!(ret_0 > 0.9, "Retention initiale proche de 1.0");
    }

    #[test]
    fn test_recall_improves_retention() {
        let engine = ReconsolidationEngine::new();
        let ret_no_recall = engine.ebbinghaus_retention(100, 0, 0.0);
        let ret_with_recall = engine.ebbinghaus_retention(100, 5, 0.0);
        assert!(ret_with_recall > ret_no_recall,
            "Les rappels ameliorent la retention (testing effect)");
    }

    #[test]
    fn test_emotional_memories_last_longer() {
        let engine = ReconsolidationEngine::new();
        let ret_neutral = engine.ebbinghaus_retention(200, 0, 0.1);
        let ret_emotional = engine.ebbinghaus_retention(200, 0, 0.9);
        assert!(ret_emotional > ret_neutral,
            "Les souvenirs emotionnels durent plus longtemps");
    }

    #[test]
    fn test_interference() {
        let engine = ReconsolidationEngine::new();
        let weak_interference = engine.compute_interference(0.3, true);
        let strong_interference = engine.compute_interference(0.9, true);
        assert_eq!(weak_interference, 1.0, "Faible similarite = pas d'interference");
        assert!(strong_interference < 1.0, "Forte similarite = interference");
    }

    #[test]
    fn test_reconsolidation_on_recall() {
        let mut engine = ReconsolidationEngine::new();
        let effect = engine.on_recall("test_memory", 0.5, 0.3, 0.6);
        assert!(effect.strength_delta > 0.0, "Le rappel renforce le souvenir");
        assert!(effect.became_labile, "Le souvenir devient labile");
        assert_eq!(effect.recall_count, 1);
    }
}
