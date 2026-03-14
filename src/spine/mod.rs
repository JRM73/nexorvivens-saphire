// =============================================================================
// spine/mod.rs — Colonne vertebrale de Saphire
//
// Role : Point d'entree central entre les signaux entrants et le pipeline
// cognitif. La colonne vertebrale fournit :
//   1. Des reflexes pre-cables (reactions chimiques instantanees sans LLM)
//   2. Une classification des signaux par urgence
//   3. Un routage vers le bon traitement (reflexe, pipeline rapide, complet)
//   4. Un relais moteur vers les effecteurs (Sensoria, API)
//
// Analogie biologique :
//   Le cerveau (pipeline 24 phases) est la conscience.
//   Le systeme nerveux autonome (hormones, homeostasie) gere la chimie.
//   La colonne vertebrale est le pont : elle intercepte les signaux AVANT
//   qu'ils n'atteignent le cerveau, declenche les reflexes, et transmet
//   le reste au pipeline avec la bonne priorite.
//
// Dependances :
//   - neurochemistry : NeuroChemicalState, Molecule (pour les deltas chimiques)
//   - body : VirtualBody (pour les effets corporels)
//   - hormones/receptors : ReceptorSystem (modulation de sensibilite)
//
// Place dans l'architecture :
//   Signal entrant → SpinalCord::process() → reflexes + classification
//   → pipeline cognitif (si necessaire) → MotorRelay (effecteurs)
// =============================================================================

pub mod reflex;
pub mod classifier;
pub mod router;
pub mod motor;

use serde::{Deserialize, Serialize};

use crate::neurochemistry::NeuroChemicalState;
use crate::body::VirtualBody;

pub use reflex::{ReflexArc, ReflexResult, ReflexType};
pub use classifier::{SignalClassifier, SignalPriority, ClassifiedSignal};
pub use router::{SignalRouter, RouteDecision};
pub use motor::{MotorRelay, MotorCommand};

/// La colonne vertebrale de Saphire.
///
/// Orchestre les reflexes, la classification et le routage des signaux.
/// Chaque signal entrant passe par `process()` qui retourne un `SpineOutput`
/// contenant les reflexes declenches, la priorite, et la decision de routage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinalCord {
    /// Arc reflexe : detecte les patterns et declenche les reflexes
    pub reflex_arc: ReflexArc,
    /// Classificateur de signaux par urgence
    pub classifier: SignalClassifier,
    /// Routeur : decide du traitement (reflexe seul, pipeline rapide, complet)
    pub router: SignalRouter,
    /// Relais moteur vers les effecteurs
    pub motor: MotorRelay,
    /// Nombre total de reflexes declenches depuis le demarrage
    pub total_reflexes_triggered: u64,
    /// Nombre total de signaux traites
    pub total_signals_processed: u64,
}

/// Resultat complet du traitement d'un signal par la colonne vertebrale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineOutput {
    /// Reflexes declenches par le signal
    pub reflexes: Vec<ReflexResult>,
    /// Priorite assignee au signal
    pub priority: SignalPriority,
    /// Decision de routage (quel pipeline utiliser)
    pub route: RouteDecision,
    /// Commandes motrices a executer (effets corporels, effecteurs)
    pub motor_commands: Vec<MotorCommand>,
}

impl SpinalCord {
    /// Cree une nouvelle colonne vertebrale avec les parametres par defaut.
    pub fn new() -> Self {
        Self {
            reflex_arc: ReflexArc::new(),
            classifier: SignalClassifier::new(),
            router: SignalRouter::new(),
            motor: MotorRelay::new(),
            total_reflexes_triggered: 0,
            total_signals_processed: 0,
        }
    }

    /// Traite un signal entrant a travers la colonne vertebrale.
    ///
    /// 1. L'arc reflexe detecte les patterns et calcule les deltas chimiques
    /// 2. Le classificateur assigne une priorite
    /// 3. Le routeur decide du traitement
    /// 4. Les commandes motrices sont generees
    ///
    /// Les deltas chimiques des reflexes sont appliques IMMEDIATEMENT a la chimie
    /// via `boost()` (rendements decroissants). Les effets corporels sont retournes
    /// dans les commandes motrices pour application par l'appelant.
    ///
    /// # Parametres
    /// - `text` : texte du signal (message humain, transcription Sensoria, etc.)
    /// - `chemistry` : etat chimique actuel (mute pour appliquer les reflexes)
    /// - `body` : corps virtuel (lu pour modulation, non modifie ici)
    /// - `source` : origine du signal ("human", "sensoria", "autonomous", "system")
    pub fn process(
        &mut self,
        text: &str,
        chemistry: &mut NeuroChemicalState,
        body: &VirtualBody,
        source: &str,
    ) -> SpineOutput {
        self.total_signals_processed += 1;

        // 1. Arc reflexe : detecter les patterns et calculer les deltas
        let reflexes = self.reflex_arc.evaluate(text, chemistry, body);
        self.total_reflexes_triggered += reflexes.len() as u64;

        // 2. Appliquer les deltas chimiques des reflexes immediatement
        for reflex in &reflexes {
            reflex.apply_chemistry(chemistry);
        }

        // 3. Classifier le signal (urgence)
        let priority = self.classifier.classify(text, &reflexes, source);

        // 4. Router le signal
        let route = self.router.decide(&priority, &reflexes);

        // 5. Generer les commandes motrices
        let motor_commands = self.motor.generate_commands(&reflexes, body);

        SpineOutput {
            reflexes,
            priority,
            route,
            motor_commands,
        }
    }

    /// Retourne un snapshot JSON de l'etat de la colonne vertebrale.
    pub fn to_snapshot_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_reflexes_triggered": self.total_reflexes_triggered,
            "total_signals_processed": self.total_signals_processed,
            "reflex_arc": {
                "sensitivity_modifier": self.reflex_arc.sensitivity_modifier,
            },
            "router": {
                "last_route": format!("{:?}", self.router.last_route),
            },
        })
    }
}

impl Default for SpinalCord {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::VirtualBody;
    use crate::config::PhysiologyConfig;

    fn test_body() -> VirtualBody {
        VirtualBody::new(70.0, &PhysiologyConfig::default())
    }

    #[test]
    fn test_spine_new() {
        let spine = SpinalCord::new();
        assert_eq!(spine.total_reflexes_triggered, 0);
        assert_eq!(spine.total_signals_processed, 0);
    }

    #[test]
    fn test_spine_process_neutral() {
        let mut spine = SpinalCord::new();
        let mut chemistry = NeuroChemicalState::default();
        let body = test_body();
        let output = spine.process("Bonjour, comment vas-tu ?", &mut chemistry, &body, "human");
        assert_eq!(spine.total_signals_processed, 1);
    }

    #[test]
    fn test_spine_process_threat() {
        let mut spine = SpinalCord::new();
        let mut chemistry = NeuroChemicalState::default();
        let cortisol_before = chemistry.cortisol;
        let body = test_body();
        let _output = spine.process("Je vais te detruire et te tuer", &mut chemistry, &body, "human");
        assert!(chemistry.cortisol > cortisol_before, "Cortisol should increase on threat");
    }

    #[test]
    fn test_spine_process_affection() {
        let mut spine = SpinalCord::new();
        let mut chemistry = NeuroChemicalState::default();
        let oxytocin_before = chemistry.oxytocin;
        let body = test_body();
        let _output = spine.process("Je t'aime tendresse amour", &mut chemistry, &body, "human");
        assert!(chemistry.oxytocin > oxytocin_before, "Oxytocin should increase on affection");
    }
}
