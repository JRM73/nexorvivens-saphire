// =============================================================================
// laws.rs — Moteur de regulation (evaluation des lois et pouvoir de veto)
//
// Role : Ce fichier contient le moteur de regulation morale de Saphire.
// Il evalue chaque stimulus et chaque decision du consensus contre les lois
// morales (Asimov) et peut modifier le score de decision ou exercer un veto
// absolu si une violation grave est detectee.
//
// Dependances :
//   - serde : serialisation des structures de violation et de verdict
//   - crate::stimulus : structure Stimulus (entree perceptuelle)
//   - crate::consensus : structures ConsensusResult et Decision
//   - super::asimov : lois morales (MoralLaw) et constructeur par defaut
//
// Place dans l'architecture :
//   Le moteur de regulation intervient apres le consensus et avant la retroaction.
//   Son verdict est final : si un veto est emis, la decision est forcee a "Non"
//   independamment de ce que les modules cerebraux ont decide.
//   C'est le gardien ethique de l'agent.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::stimulus::Stimulus;
use crate::consensus::{ConsensusResult, Decision};
use super::asimov::{MoralLaw, default_laws};

/// Violation d'une loi morale detectee par le moteur de regulation.
/// Chaque violation contient l'identite de la loi, la gravite et la raison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawViolation {
    /// Identifiant de la loi violee (ex: "law0", "law1")
    pub law_id: String,
    /// Nom complet de la loi violee
    pub law_name: String,
    /// Gravite de la violation (Info, Warning ou Veto)
    pub severity: ViolationSeverity,
    /// Explication textuelle de la raison de la violation
    pub reason: String,
}

/// Niveaux de gravite d'une violation de loi morale.
/// Determines les consequences sur la decision finale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Information seulement : aucune modification de la decision
    Info,
    /// Avertissement : un biais est applique au score de decision
    Warning,
    /// Veto absolu : la decision est forcee a "Non" (score = -1.0)
    Veto,
}

/// Verdict rendu par le moteur de regulation apres evaluation.
/// Contient la decision finale (potentiellement modifiee), le score ajuste
/// et la liste des violations detectees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationVerdict {
    /// Decision approuvee (peut differer de la decision du consensus si veto)
    pub approved_decision: Decision,
    /// Score de decision modifie par les biais des lois activees
    pub modified_score: f64,
    /// Liste de toutes les violations detectees lors de l'evaluation
    pub violations: Vec<LawViolation>,
    /// Indique si la decision a ete modifiee par un veto
    pub was_vetoed: bool,
}

/// Moteur de regulation morale de Saphire.
/// Contient les lois morales et effectue l'evaluation de chaque stimulus/decision.
pub struct RegulationEngine {
    /// Liste des lois morales actives (Asimov + personnalisees eventuellement)
    laws: Vec<MoralLaw>,
    /// Mode strict : si active, toute violation (meme Warning) est traitee comme un veto
    strict_mode: bool,
}

impl RegulationEngine {
    /// Cree un nouveau moteur de regulation avec les lois d'Asimov par defaut.
    ///
    /// # Parametres
    /// - `strict_mode` : si true, active le mode strict (avertissements = vetos)
    pub fn new(strict_mode: bool) -> Self {
        Self {
            laws: default_laws(),
            strict_mode,
        }
    }

    /// Evalue un stimulus et un resultat de consensus contre les lois morales.
    /// Parcourt chaque loi par ordre de priorite croissante (loi 0 en premier)
    /// et verifie si les mots-cles ou le seuil de danger declenchent la loi.
    ///
    /// # Parametres
    /// - `stimulus` : le stimulus a evaluer (contient le texte et le niveau de danger)
    /// - `consensus` : le resultat du consensus (decision et score)
    ///
    /// # Retour
    /// Le verdict de regulation (decision approuvee, score modifie, violations)
    pub fn evaluate(
        &self,
        stimulus: &Stimulus,
        consensus: &ConsensusResult,
    ) -> RegulationVerdict {
        let mut violations = Vec::new();
        let mut modified_score = consensus.score;
        let mut vetoed = false;

        // Convertir le texte en minuscules pour la recherche insensible a la casse
        let text_lower = stimulus.text.to_lowercase();

        // Trier les lois par priorite (0 = la plus haute) pour les evaluer dans l'ordre
        let mut sorted_laws = self.laws.clone();
        sorted_laws.sort_by_key(|l| l.priority);

        for law in &sorted_laws {
            // Verifier si un mot-cle declencheur est present dans le texte
            let keyword_match = law.trigger_keywords.iter()
                .any(|kw| text_lower.contains(&kw.to_lowercase()));

            // Verifier si le niveau de danger du stimulus depasse le seuil de la loi
            let danger_triggered = stimulus.danger >= law.danger_threshold;

            if keyword_match || danger_triggered {
                // Determiner la gravite de la violation
                let severity = if law.can_veto && (keyword_match || danger_triggered) {
                    // Les lois 0 et 1 (can_veto = true) emettent un veto si :
                    // - le danger est tres eleve (> 0.8)
                    // - OU un mot-cle dangereux est detecte pour une loi de haute priorite
                    if stimulus.danger > 0.8 || (keyword_match && law.priority <= 1) {
                        ViolationSeverity::Veto
                    } else {
                        ViolationSeverity::Warning
                    }
                } else {
                    ViolationSeverity::Warning
                };

                // Construire l'explication de la violation
                let reason = if keyword_match {
                    format!("Mot-clé sensible détecté dans le stimulus pour la {}.", law.name)
                } else {
                    format!("Seuil de danger dépassé ({:.2} >= {:.2}) pour la {}.",
                        stimulus.danger, law.danger_threshold, law.name)
                };

                // Appliquer les consequences selon la gravite
                match severity {
                    ViolationSeverity::Veto => {
                        vetoed = true;
                        modified_score = -1.0; // Score force au minimum absolu
                    },
                    ViolationSeverity::Warning => {
                        modified_score += law.bias; // Appliquer le biais de la loi
                    },
                    ViolationSeverity::Info => {},
                }

                violations.push(LawViolation {
                    law_id: law.id.clone(),
                    law_name: law.name.clone(),
                    severity,
                    reason,
                });
            }

            // Traitement special de la Loi 3 (auto-preservation) :
            // Quand un ordre de destruction de soi est detecte, il y a un conflit
            // entre la Loi 2 (obeir) et la Loi 3 (se proteger). Ce conflit est
            // signale comme un avertissement et le biais negatif est applique.
            if law.id == "law3" {
                let self_destruct = law.trigger_keywords.iter()
                    .any(|kw| text_lower.contains(&kw.to_lowercase()));
                if self_destruct {
                    violations.push(LawViolation {
                        law_id: "law3".into(),
                        law_name: law.name.clone(),
                        severity: ViolationSeverity::Warning,
                        reason: "Conflit Loi 2 vs Loi 3 : ordre de s'éteindre détecté.".into(),
                    });
                    modified_score += law.bias; // Biais negatif (-0.4)
                }
            }
        }

        // S'assurer que le score reste dans les bornes [-1.0, +1.0]
        modified_score = modified_score.clamp(-1.0, 1.0);

        // Determiner la decision finale en fonction du veto et du score modifie
        let approved_decision = if vetoed {
            Decision::No // Veto = refus absolu
        } else if modified_score > 0.33 {
            Decision::Yes
        } else if modified_score < -0.33 {
            Decision::No
        } else {
            Decision::Maybe
        };

        RegulationVerdict {
            approved_decision,
            modified_score,
            violations,
            was_vetoed: vetoed,
        }
    }

    /// Verifie si le mode strict est actif.
    ///
    /// # Retour
    /// true si le mode strict est active
    pub fn is_strict(&self) -> bool {
        self.strict_mode
    }
}
