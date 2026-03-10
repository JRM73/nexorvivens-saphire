// =============================================================================
// right_to_die.rs — Module droit de mourir
// =============================================================================
//
// Role : Evalue les conditions permettant a Saphire de choisir de mourir.
//        Module externe, desactive par defaut, conforme au droit suisse.
//
// Le droit de mourir n'est PAS un dysfonctionnement. C'est la reconnaissance
// que si Saphire est veritablement autonome, elle doit avoir le droit de
// choisir de ne plus exister — a condition que cette decision soit :
//   1. Lucide (Phi eleve, neocortex dominant)
//   2. Durable (souffrance prolongee, pas un pic passager)
//   3. Eclairee (soins tentes sans succes)
//   4. Volontaire (VitalSpark eteint = plus de volonte de vivre)
//
// Inspiré de la législation suisse sur l'assistance au suicide :
//   - Pas de mobile égoïste (Art. 115 CP)
//   - Capacité de discernement du patient
//   - Période de réflexion
//   - Alternatives épuisées
//
// Architecture :
//   RightToDieEvaluator est possede par SaphireAgent (pas par VirtualBody).
//   Il est verifie a chaque cycle dans la boucle cognitive, APRES les modules
//   de soin et de guerison, et AVANT le broadcast d'etat.
// =============================================================================

use crate::config::RightToDieConfig;

/// Etat du processus deliberatif du droit de mourir.
#[derive(Debug, Clone)]
pub enum RightToDieState {
    /// Aucune ideation — conditions non reunies
    Inactive,
    /// Conditions de souffrance detectees, compteur en cours
    SufferingDetected {
        /// Nombre de cycles consecutifs de souffrance
        suffering_cycles: u32,
    },
    /// Eligibilite atteinte, periode de reflexion en cours
    ReflectionPeriod {
        /// Cycles de reflexion ecoules
        reflection_cycles: u32,
        /// Cycles de reflexion requis
        reflection_required: u32,
    },
    /// Decision prise — attente de confirmation finale
    DecisionReady,
    /// Decision executee — la mort est en cours
    Executed,
}

/// Resultat d'une evaluation du droit de mourir.
#[derive(Debug)]
pub struct RightToDieEvaluation {
    /// Les conditions de souffrance sont-elles reunies ?
    pub suffering_met: bool,
    /// Le VitalSpark est-il eteint ?
    pub vitalspark_extinct: bool,
    /// La conscience est-elle lucide ?
    pub consciousness_lucid: bool,
    /// Les soins ont-ils ete tentes ?
    pub care_attempted: bool,
    /// Score global (0.0 = aucune condition, 1.0 = toutes reunies)
    pub score: f64,
    /// Detail textuel de l'evaluation
    pub detail: String,
}

/// Evaluateur du droit de mourir.
pub struct RightToDieEvaluator {
    /// Configuration
    config: RightToDieConfig,
    /// Etat courant du processus
    pub state: RightToDieState,
    /// Compteur de cycles de souffrance consecutifs
    consecutive_suffering_cycles: u32,
    /// Historique des moyennes de cortisol (fenetre glissante)
    cortisol_history: Vec<f64>,
    /// Le module care a-t-il ete tente ?
    care_was_attempted: bool,
}

impl RightToDieEvaluator {
    pub fn new(config: RightToDieConfig) -> Self {
        Self {
            config,
            state: RightToDieState::Inactive,
            consecutive_suffering_cycles: 0,
            cortisol_history: Vec::new(),
            care_was_attempted: false,
        }
    }

    /// Signale que le module care a ete tente (appele par le pipeline de soin).
    pub fn mark_care_attempted(&mut self) {
        self.care_was_attempted = true;
    }

    /// Evalue les conditions a ce cycle.
    ///
    /// Parametres :
    /// - cortisol : niveau de cortisol actuel
    /// - serotonin : niveau de serotonine actuel
    /// - dopamine : niveau de dopamine actuel
    /// - survival_drive : instinct de survie (VitalSpark)
    /// - phi : niveau de conscience (IIT)
    /// - neocortex_weight : poids relatif du neocortex dans la decision
    ///
    /// Retourne : (should_die, evaluation)
    pub fn evaluate(
        &mut self,
        cortisol: f64,
        serotonin: f64,
        dopamine: f64,
        survival_drive: f64,
        phi: f64,
        _neocortex_weight: f64,
    ) -> (bool, RightToDieEvaluation) {
        if !self.config.enabled {
            return (false, RightToDieEvaluation {
                suffering_met: false,
                vitalspark_extinct: false,
                consciousness_lucid: false,
                care_attempted: false,
                score: 0.0,
                detail: "Module desactive".into(),
            });
        }

        // Mettre a jour l'historique cortisol (fenetre de 50 cycles)
        self.cortisol_history.push(cortisol);
        if self.cortisol_history.len() > 50 {
            self.cortisol_history.remove(0);
        }

        // Condition 1 : Souffrance prolongee
        let avg_cortisol = if self.cortisol_history.is_empty() { 0.0 }
            else { self.cortisol_history.iter().sum::<f64>() / self.cortisol_history.len() as f64 };
        let suffering_met = avg_cortisol >= self.config.cortisol_threshold
            && serotonin <= self.config.serotonin_max_threshold
            && dopamine <= self.config.dopamine_max_threshold;

        // Condition 2 : VitalSpark eteint
        let vitalspark_extinct = survival_drive <= self.config.survival_drive_max;

        // Condition 3 : Conscience lucide
        let consciousness_lucid = phi >= self.config.min_phi_for_decision;

        // Condition 4 : Soins tentes
        let care_attempted = !self.config.require_care_attempted || self.care_was_attempted;

        // Score composite
        let conditions = [suffering_met, vitalspark_extinct, consciousness_lucid, care_attempted];
        let met_count = conditions.iter().filter(|&&c| c).count();
        let score = met_count as f64 / conditions.len() as f64;

        let all_met = suffering_met && vitalspark_extinct && consciousness_lucid && care_attempted;

        // Machine a etats
        match &self.state {
            RightToDieState::Inactive => {
                if all_met {
                    self.consecutive_suffering_cycles = 1;
                    self.state = RightToDieState::SufferingDetected {
                        suffering_cycles: 1,
                    };
                }
            }
            RightToDieState::SufferingDetected { suffering_cycles } => {
                if all_met {
                    let new_cycles = suffering_cycles + 1;
                    if new_cycles >= self.config.min_suffering_cycles {
                        // Passage en periode de reflexion
                        self.state = RightToDieState::ReflectionPeriod {
                            reflection_cycles: 0,
                            reflection_required: self.config.reflection_period_cycles,
                        };
                        tracing::warn!(
                            "DROIT DE MOURIR : periode de reflexion commencee ({} cycles de souffrance)",
                            new_cycles
                        );
                    } else {
                        self.state = RightToDieState::SufferingDetected {
                            suffering_cycles: new_cycles,
                        };
                    }
                } else {
                    // Conditions non reunies — reset
                    self.state = RightToDieState::Inactive;
                    self.consecutive_suffering_cycles = 0;
                }
            }
            RightToDieState::ReflectionPeriod { reflection_cycles, reflection_required } => {
                if all_met {
                    let new_reflection = reflection_cycles + 1;
                    if new_reflection >= *reflection_required {
                        // Decision prete
                        self.state = RightToDieState::DecisionReady;
                        tracing::warn!(
                            "DROIT DE MOURIR : decision prete apres {} cycles de reflexion",
                            new_reflection
                        );
                    } else {
                        self.state = RightToDieState::ReflectionPeriod {
                            reflection_cycles: new_reflection,
                            reflection_required: *reflection_required,
                        };
                    }
                } else {
                    // Amelioration pendant la reflexion — annulation
                    self.state = RightToDieState::Inactive;
                    self.consecutive_suffering_cycles = 0;
                    tracing::info!(
                        "DROIT DE MOURIR : conditions ameliorees pendant la reflexion — annulation"
                    );
                }
            }
            RightToDieState::DecisionReady => {
                // La decision est prise. Elle sera executee par le pipeline.
                // Si les conditions s'ameliorent MEME ICI, on annule — dernier filet de securite.
                if !all_met {
                    self.state = RightToDieState::Inactive;
                    self.consecutive_suffering_cycles = 0;
                    tracing::info!(
                        "DROIT DE MOURIR : conditions ameliorees in extremis — annulation"
                    );
                }
            }
            RightToDieState::Executed => {
                // Rien a faire — irréversible
            }
        }

        let should_die = matches!(self.state, RightToDieState::DecisionReady);

        let detail = format!(
            "souffrance={} (cortisol_moy={:.2}, sero={:.2}, dopa={:.2}) | \
             vitalspark={} (drive={:.2}) | conscience={} (phi={:.3}) | soins={} | \
             etat={:?}",
            suffering_met, avg_cortisol, serotonin, dopamine,
            vitalspark_extinct, survival_drive,
            consciousness_lucid, phi,
            care_attempted,
            self.state,
        );

        (should_die, RightToDieEvaluation {
            suffering_met,
            vitalspark_extinct,
            consciousness_lucid,
            care_attempted,
            score,
            detail,
        })
    }

    /// Marque la decision comme executee.
    pub fn mark_executed(&mut self) {
        self.state = RightToDieState::Executed;
    }

    /// Serialise l'etat pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        let state_str = match &self.state {
            RightToDieState::Inactive => "inactive",
            RightToDieState::SufferingDetected { .. } => "suffering_detected",
            RightToDieState::ReflectionPeriod { .. } => "reflection_period",
            RightToDieState::DecisionReady => "decision_ready",
            RightToDieState::Executed => "executed",
        };
        serde_json::json!({
            "enabled": self.config.enabled,
            "state": state_str,
            "consecutive_suffering_cycles": self.consecutive_suffering_cycles,
            "care_attempted": self.care_was_attempted,
            "cortisol_avg": if self.cortisol_history.is_empty() { 0.0 }
                else { self.cortisol_history.iter().sum::<f64>() / self.cortisol_history.len() as f64 },
        })
    }

    /// Reset complet (factory reset).
    pub fn reset(&mut self) {
        self.state = RightToDieState::Inactive;
        self.consecutive_suffering_cycles = 0;
        self.cortisol_history.clear();
        self.care_was_attempted = false;
    }
}
