// =============================================================================
// neocortex.rs — Module néocortex : analyse rationnelle, clarté mentale
// =============================================================================
//
// Rôle : Ce fichier implémente le module néocortex de Saphire, inspiré du
// néocortex dans le modèle du cerveau triunique de Paul MacLean. Il gère
// l'analyse rationnelle, le calcul coût/bénéfice et la prise de décision
// logique. C'est le module le plus « réfléchi » mais il est vulnérable
// au stress qui dégrade sa clarté mentale.
//
// Dépendances :
//   - crate::neurochemistry::NeuroChemicalState : état chimique (cortisol et
//     adrénaline dégradent la clarté, noradrénaline l'améliore)
//   - crate::stimulus::Stimulus : entrée sensorielle (reward, danger, social, urgency)
//   - super::BrainModule, ModuleSignal : trait et type de sortie communs
//
// Place dans l'architecture :
//   Troisième des 3 modules cérébraux. Son signal reflète une analyse
//   rationnelle pondérée. Son poids dans le consensus augmente quand la
//   sérotonine et la noradrénaline sont élevées (calme + focus), et
//   diminue sous stress (cortisol, adrénaline élevés).
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// Le néocortex — pensée rationnelle, analyse coût/bénéfice.
/// Module le plus évolué, il analyse le stimulus de manière logique
/// en pesant les bénéfices contre les risques. Sa performance dépend
/// de la « clarté mentale », elle-même influencée par la neurochimie.
pub struct NeocortexModule;

impl BrainModule for NeocortexModule {
    /// Retourne le nom du module : "Néocortex".
    fn name(&self) -> &str {
        "Néocortex"
    }

    /// Traite un stimulus du point de vue rationnel (analyse coût/bénéfice).
    ///
    /// Algorithme :
    /// 1. Calcul de la clarté mentale : pénalisée par le stress (cortisol +
    ///    adrénaline), améliorée par le focus (noradrénaline).
    /// 2. Analyse coût/bénéfice : récompense et social comme bénéfices,
    ///    danger et urgence comme coûts, le tout multiplié par la clarté.
    /// 3. Confiance proportionnelle à la clarté mentale.
    ///
    /// La clarté mentale est bornée entre 0.1 (minimum vital, même en
    /// stress extrême le néocortex fonctionne un peu) et 1.5 (focus optimal).
    ///
    /// # Paramètres
    /// - `stimulus` : entrée sensorielle avec ses scores perceptuels.
    /// - `chemistry` : état chimique (cortisol et adrénaline dégradent la clarté,
    ///   noradrénaline l'améliore).
    ///
    /// # Retour
    /// Un `ModuleSignal` avec signal, confiance et raisonnement explicatif.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Pénalité de stress : le cortisol et l'adrénaline réduisent la
        // clarté mentale. Cela simule l'effet bien documenté du stress
        // sur les fonctions exécutives (difficulté à réfléchir sous pression).
        let stress_penalty = (chemistry.cortisol + chemistry.adrenaline) * 0.5;

        // Bonus de focus : la noradrénaline améliore la concentration et
        // la vigilance cognitive (effet stimulant sur l'attention).
        let focus = chemistry.noradrenaline * 0.4;

        // Clarté mentale résultante : 1.0 = normale, < 1.0 = dégradée par
        // le stress, > 1.0 = améliorée par le focus. Bornée entre 0.1
        // (minimum vital) et 1.5 (performance optimale).
        let clarity = (1.0 - stress_penalty + focus).clamp(0.1, 1.5);

        // Analyse rationnelle coût/bénéfice :
        //   Bénéfices : récompense (poids 0.7) + social (poids 0.3)
        //   Coûts : danger (poids 0.6) + urgence (poids 0.2)
        // Le résultat est multiplié par la clarté mentale : sous stress,
        // l'analyse est atténuée ; avec du focus, elle est amplifiée.
        let raw = (stimulus.reward * 0.7
            + stimulus.social * 0.3
            - stimulus.danger * 0.6
            - stimulus.urgency * 0.2)
            * clarity;

        // tanh() borne naturellement le signal entre -1 et +1
        let signal = raw.tanh();

        // Confiance proportionnelle à la clarté mentale : le néocortex
        // a confiance en son jugement uniquement quand il peut bien réfléchir.
        // clarity/1.5 normalise entre 0 et 1, borné entre 0.2 et 0.95.
        let confidence = (clarity / 1.5).clamp(0.2, 0.95);

        // Raisonnement détaillé avec état de la clarté mentale
        let reasoning = format!(
            "Analyse rationnelle (clarté={:.2}) : bénéfice={:.2}, risque={:.2}, social={:.2}. {}",
            clarity,
            stimulus.reward,
            stimulus.danger,
            stimulus.social,
            if clarity < 0.5 {
                "⚠ Clarté mentale dégradée par le stress."
            } else if clarity > 1.0 {
                "Focus élevé, analyse optimale."
            } else {
                "Clarté mentale normale."
            }
        );

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
