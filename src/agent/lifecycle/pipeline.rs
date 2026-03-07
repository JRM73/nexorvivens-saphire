// =============================================================================
// lifecycle/pipeline.rs — Pipeline cerebral simplifie (version lite)
// =============================================================================
//
// Version allegee pour le papier ArXiv.
// Les modules supprimes (hormones, besoins, phobies, conditions, sentiments,
// connectome, neural, tuning, plugins, senses, sleep, etc.) sont retires.
//
// Pipeline lite :
//   VAGUE 1 — PERCEPTION
//     1. Snapshot chimie avant traitement
//     2. Calcul de l'emotion (36 emotions)
//
//   VAGUE 2 — TRAITEMENT
//     3. 3 modules cerebraux (reptilien, limbique, neocortex)
//     4. Consensus pondere (seuils depuis config)
//     5. Penalite physiologique (corps virtuel)
//     6. Evaluation de la conscience (IIT/Phi)
//
//   VAGUE 3 — REPONSE
//     7. Regulation ethique (Asimov)
//     8. Retroaction chimique
//     9. Surveillance chimique, logs, trace, return
// =============================================================================

use crate::consensus::{self, ConsensusThresholds, Decision};
use crate::emotions::EmotionalState;
use crate::modules::BrainModule;
use crate::stimulus::Stimulus;
use crate::logging::{LogLevel, LogCategory};
use crate::logging::trace::CognitiveTrace;

use super::SaphireAgent;
use super::ProcessResult;

impl SaphireAgent {
    /// Traite un stimulus a travers le pipeline cerebral simplifie.
    ///
    /// Pipeline 3 vagues :
    ///   VAGUE 1 — Perception (chimie, emotion)
    ///   VAGUE 2 — Traitement (3 modules, consensus, conscience)
    ///   VAGUE 3 — Reponse (ethique, feedback chimique, logs)
    pub fn process_stimulus(&mut self, stimulus: &Stimulus) -> ProcessResult {

        // =====================================================================
        // VAGUE 1 — PERCEPTION
        // =====================================================================

        // Etape 1 : snapshot chimie avant traitement (pour la trace cognitive)
        let chemistry_before = serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
            "adrenaline": self.chemistry.adrenaline,
            "oxytocin": self.chemistry.oxytocin,
            "endorphin": self.chemistry.endorphin,
            "noradrenaline": self.chemistry.noradrenaline,
        });

        // Etape 2 : calcul de l'etat emotionnel depuis la chimie courante
        let emotion = EmotionalState::compute(&self.chemistry);

        self.mood.update(emotion.valence, emotion.arousal);
        self.last_emotion = emotion.dominant.clone();
        self.last_valence = emotion.valence;

        // =====================================================================
        // VAGUE 2 — TRAITEMENT
        // =====================================================================

        // Etape 3 : chaque module cerebral traite le stimulus independamment
        let sig_r = self.reptilian.process(stimulus, &self.chemistry);
        let sig_l = self.limbic.process(stimulus, &self.chemistry);
        let sig_n = self.neocortex.process(stimulus, &self.chemistry);
        let signals = [sig_r, sig_l, sig_n];

        // Etape 4 : consensus pondere (seuils depuis config.consensus)
        let thresholds = ConsensusThresholds {
            threshold_yes: self.config.consensus.threshold_yes,
            threshold_no: self.config.consensus.threshold_no,
        };
        let default_params = crate::tuning::TunableParams::default();
        let mut consensus_result = consensus::consensus(&signals, &self.chemistry, &thresholds, &default_params);

        // Etape 5 : penalite physiologique (corps virtuel)
        let cognitive_degradation = if self.config.body.enabled && self.config.body.physiology.enabled {
            let deg = self.body.physiology.cognitive_degradation(&self.config.body.physiology);
            if deg > 0.0 {
                consensus_result.coherence *= 1.0 - deg * 0.7;
                consensus_result.score *= 1.0 - deg * 0.3;

                if deg > 0.3 {
                    self.log(LogLevel::Warn, LogCategory::Body,
                        format!("Degradation cognitive: {:.0}% — SpO2: {:.0}%",
                            deg * 100.0, self.body.physiology.spo2),
                        serde_json::json!({
                            "degradation": deg,
                            "spo2": self.body.physiology.spo2,
                            "glycemia": self.body.physiology.glycemia,
                        }));
                }
            }
            deg
        } else {
            0.0
        };

        // Etape 6 : evaluation de la conscience (IIT + GWT simplifie)
        // GwtInput et PredictiveInput utilises avec valeurs par defaut (pas de brain_network)
        let gwt_input = crate::consciousness::GwtInput::default();
        let predictive_input = crate::consciousness::PredictiveInput::default();

        let interoception = if self.config.body.enabled {
            Some(self.body.interoception_score())
        } else {
            None
        };
        let mut consciousness = self.consciousness.evaluate(
            &self.chemistry, &consensus_result, &emotion, interoception,
            Some(&gwt_input), Some(&predictive_input),
        );

        // Appliquer la degradation cognitive sur la conscience
        if cognitive_degradation > 0.0 {
            consciousness.level *= 1.0 - cognitive_degradation * 0.5;
            consciousness.phi *= 1.0 - cognitive_degradation * 0.4;
        }

        self.last_consciousness = consciousness.level;

        // =====================================================================
        // VAGUE 3 — REPONSE
        // =====================================================================

        // Etape 7 : regulation ethique (lois d'Asimov, filtrage de securite)
        let verdict = self.regulation.evaluate(stimulus, &consensus_result);

        // Etape 8 : retroaction chimique (feedback chimique minimal)
        let chem_before_feedback = self.chemistry.clone();
        let is_human = stimulus.source == crate::stimulus::StimulusSource::Human;
        let homeostasis_rate = self.config.feedback.homeostasis_rate;
        let dopamine_boost = homeostasis_rate * 0.4;
        let cortisol_relief = homeostasis_rate * 0.3;
        let indecision_stress = homeostasis_rate * 0.2;
        match &verdict.approved_decision {
            Decision::Yes => {
                if stimulus.reward > 0.5 {
                    self.chemistry.feedback_positive(dopamine_boost);
                }
                if stimulus.social > 0.7 {
                    self.chemistry.feedback_social();
                }
            },
            Decision::No => {
                if stimulus.danger > 0.5 {
                    self.chemistry.feedback_danger_avoided(cortisol_relief);
                }
            },
            Decision::Maybe => {
                if is_human {
                    self.chemistry.apply_cortisol_penalty(0.01);
                } else {
                    self.chemistry.feedback_indecision(indecision_stress);
                }
            },
        }

        // Novelty seulement si score eleve
        if stimulus.novelty > 0.7 {
            self.chemistry.feedback_novelty();
        }

        // Coherence basse → leger stress cognitif
        self.chemistry.feedback_low_coherence(consensus_result.coherence);

        // Cap delta max ±0.10 par molecule sur le bloc feedback
        {
            let cap = 0.10;
            self.chemistry.dopamine = self.chemistry.dopamine.clamp(
                chem_before_feedback.dopamine - cap, chem_before_feedback.dopamine + cap).clamp(0.0, 1.0);
            self.chemistry.cortisol = self.chemistry.cortisol.clamp(
                chem_before_feedback.cortisol - cap, chem_before_feedback.cortisol + cap).clamp(0.0, 1.0);
            self.chemistry.serotonin = self.chemistry.serotonin.clamp(
                chem_before_feedback.serotonin - cap, chem_before_feedback.serotonin + cap).clamp(0.0, 1.0);
            self.chemistry.adrenaline = self.chemistry.adrenaline.clamp(
                chem_before_feedback.adrenaline - cap, chem_before_feedback.adrenaline + cap).clamp(0.0, 1.0);
            self.chemistry.oxytocin = self.chemistry.oxytocin.clamp(
                chem_before_feedback.oxytocin - cap, chem_before_feedback.oxytocin + cap).clamp(0.0, 1.0);
            self.chemistry.endorphin = self.chemistry.endorphin.clamp(
                chem_before_feedback.endorphin - cap, chem_before_feedback.endorphin + cap).clamp(0.0, 1.0);
            self.chemistry.noradrenaline = self.chemistry.noradrenaline.clamp(
                chem_before_feedback.noradrenaline - cap, chem_before_feedback.noradrenaline + cap).clamp(0.0, 1.0);
        }

        // Incrementer le compteur de cycles
        self.cycle_count += 1;
        self.identity.update_stats(
            &emotion.dominant,
            stimulus.source == crate::stimulus::StimulusSource::Human,
        );
        self.identity.refresh_description();

        // Auto-surveillance chimique (compteurs + alertes periodiques)
        self.check_chemical_health();

        // Logging du pipeline
        self.log(LogLevel::Debug, LogCategory::Pipeline,
            format!("Pipeline: decision={}, emotion={}, phi={:.2}, consciousness={:.2}",
                verdict.approved_decision.as_str(), emotion.dominant,
                consciousness.phi, consciousness.level),
            serde_json::json!({
                "consensus_score": consensus_result.score,
                "coherence": consensus_result.coherence,
                "emotion": emotion.dominant,
                "consciousness": consciousness.level,
                "phi": consciousness.phi,
            }));

        // Construire la trace cognitive partielle
        let trace = if self.logs_db.is_some() {
            let source_str = format!("{:?}", stimulus.source);
            let mut t = CognitiveTrace::new(self.cycle_count, &source_str, self.session_id);
            t.set_input(&stimulus.text);
            t.set_brain(serde_json::json!({
                "weights": consensus_result.weights,
                "signals_count": consensus_result.signals.len(),
            }));
            t.set_consensus(serde_json::json!({
                "score": consensus_result.score,
                "decision": verdict.approved_decision.as_str(),
                "coherence": consensus_result.coherence,
                "weights": consensus_result.weights,
            }));
            t.set_chemistry_before(chemistry_before);
            t.set_chemistry_after(serde_json::json!({
                "dopamine": self.chemistry.dopamine,
                "cortisol": self.chemistry.cortisol,
                "serotonin": self.chemistry.serotonin,
                "adrenaline": self.chemistry.adrenaline,
                "oxytocin": self.chemistry.oxytocin,
                "endorphin": self.chemistry.endorphin,
                "noradrenaline": self.chemistry.noradrenaline,
            }));
            t.set_emotion(serde_json::json!({
                "dominant": emotion.dominant,
                "valence": emotion.valence,
                "arousal": emotion.arousal,
                "dominance": 0.5,
            }));
            t.set_consciousness(serde_json::json!({
                "level": consciousness.level,
                "phi": consciousness.phi,
                "workspace_strength": consciousness.workspace_strength,
                "workspace_winner": consciousness.workspace_winner,
                "global_surprise": consciousness.global_surprise,
            }));
            t.set_regulation(serde_json::json!({
                "approved": verdict.approved_decision.as_str(),
                "violations": verdict.violations.len(),
            }));

            // Donnees du corps et du coeur
            if self.config.body.enabled {
                let body_s = self.body.status();
                t.set_heart(serde_json::json!({
                    "bpm": body_s.heart.bpm,
                    "beat_count": body_s.heart.beat_count,
                    "hrv": body_s.heart.hrv,
                    "strength": body_s.heart.strength,
                    "is_racing": body_s.heart.is_racing,
                    "is_calm": body_s.heart.is_calm,
                }));
                t.set_body(serde_json::json!({
                    "energy": body_s.energy,
                    "tension": body_s.tension,
                    "warmth": body_s.warmth,
                    "comfort": body_s.comfort,
                    "pain": body_s.pain,
                    "vitality": body_s.vitality,
                    "breath_rate": body_s.breath_rate,
                    "body_awareness": body_s.body_awareness,
                }));
            }

            // Donnees ethiques
            if self.config.ethics.enabled {
                t.set_ethics(serde_json::json!({
                    "active_personal_count": self.ethics.active_personal_count(),
                    "total_personal_count": self.ethics.total_personal_count(),
                }));
            }

            Some(t)
        } else {
            None
        };

        ProcessResult {
            consensus: consensus_result,
            emotion,
            consciousness,
            verdict,
            trace,
        }
    }

    /// Auto-surveillance de la sante chimique.
    ///
    /// Met a jour les ring-buffers a chaque cycle.
    /// Tous les 50 cycles, emet des alertes si des derives sont detectees.
    fn check_chemical_health(&mut self) {
        // Mise a jour des ring-buffers
        self.recent_emotions.push_back(self.last_emotion.clone());
        if self.recent_emotions.len() > 200 {
            self.recent_emotions.pop_front();
        }
        self.recent_valences.push_back(self.last_valence);
        if self.recent_valences.len() > 200 {
            self.recent_valences.pop_front();
        }

        // Compteurs de cycles consecutifs
        if self.chemistry.cortisol < 0.10 {
            self.cortisol_flat_cycles += 1;
        } else {
            self.cortisol_flat_cycles = 0;
        }

        if self.chemistry.dopamine > 0.85 {
            self.dopamine_ceiling_cycles += 1;
        } else {
            self.dopamine_ceiling_cycles = 0;
        }

        if self.chemistry.serotonin > 0.85 {
            self.serotonin_ceiling_cycles += 1;
        } else {
            self.serotonin_ceiling_cycles = 0;
        }

        // Emission des alertes tous les 50 cycles
        if self.cycle_count % 50 != 0 {
            return;
        }

        if self.cortisol_flat_cycles >= 100 {
            self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                format!("Cortisol anormalement bas depuis {} cycles (< 0.10)", self.cortisol_flat_cycles),
                serde_json::json!({
                    "alert": "cortisol_flat",
                    "cycles": self.cortisol_flat_cycles,
                    "cortisol": self.chemistry.cortisol,
                }));
        }

        if self.dopamine_ceiling_cycles >= 50 {
            self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                format!("Dopamine en saturation depuis {} cycles (> 0.85)", self.dopamine_ceiling_cycles),
                serde_json::json!({
                    "alert": "dopamine_saturation",
                    "cycles": self.dopamine_ceiling_cycles,
                    "dopamine": self.chemistry.dopamine,
                }));
        }

        if self.serotonin_ceiling_cycles >= 50 {
            self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                format!("Serotonine en saturation depuis {} cycles (> 0.85)", self.serotonin_ceiling_cycles),
                serde_json::json!({
                    "alert": "serotonin_saturation",
                    "cycles": self.serotonin_ceiling_cycles,
                    "serotonin": self.chemistry.serotonin,
                }));
        }

        if self.recent_emotions.len() >= 200 {
            let mut distinct = std::collections::HashSet::new();
            for e in &self.recent_emotions {
                distinct.insert(e.as_str());
            }
            if distinct.len() < 5 {
                self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                    format!("Monotonie emotionnelle: {} emotions distinctes sur 200 cycles", distinct.len()),
                    serde_json::json!({
                        "alert": "emotional_monotony",
                        "distinct_emotions": distinct.len(),
                    }));
            }
        }

        if self.recent_valences.len() >= 200 {
            let mean = self.recent_valences.iter().sum::<f64>() / self.recent_valences.len() as f64;
            let variance = self.recent_valences.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / self.recent_valences.len() as f64;
            let stddev = variance.sqrt();
            if stddev < 0.05 {
                self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                    format!("Valence figee: stddev={:.4} sur 200 cycles", stddev),
                    serde_json::json!({
                        "alert": "valence_stuck",
                        "stddev": stddev,
                        "mean_valence": mean,
                    }));
            }
        }
    }
}
