// =============================================================================
// reptilian.rs — Module reptilien : survie, danger, réflexes
// =============================================================================
//
// Rôle : Ce fichier implémente le module reptilien de Saphire, inspiré du
// complexe R (cerveau reptilien) dans le modèle du cerveau triunique de
// Paul MacLean. Il gère les réactions instinctives de survie :
// détection du danger, réaction de fuite, vigilance face à l'inconnu.
//
// Dépendances :
//   - crate::neurochemistry::NeuroChemicalState : état chimique (amplifie la menace)
//   - crate::stimulus::Stimulus : entrée sensorielle (danger, urgence, familiarité)
//   - super::BrainModule, ModuleSignal : trait et type de sortie communs
//
// Place dans l'architecture :
//   Premier des 3 modules cérébraux. Son signal (généralement négatif face
//   au danger) est combiné avec les signaux limbique et néocortex dans
//   consensus.rs. Son poids augmente quand le cortisol et l'adrénaline
//   sont élevés.
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// Le cerveau reptilien — réagit au danger et à la survie.
/// Module le plus primitif, il répond rapidement et instinctivement.
/// Son signal est presque toujours négatif (rejet) en présence de danger.
pub struct ReptilianModule;

impl BrainModule for ReptilianModule {
    /// Retourne le nom du module : "Reptilien".
    fn name(&self) -> &str {
        "Reptilien"
    }

    /// Traite un stimulus du point de vue de la survie et du danger.
    ///
    /// Algorithme :
    /// 1. Calcul de la menace perçue : danger du stimulus amplifié par le
    ///    cortisol (stress ambiant) et l'adrénaline (état d'alerte).
    /// 2. Instinct de survie : réaction face à l'inconnu (faible familiarité)
    ///    combinée avec l'urgence.
    /// 3. Signal brut = -menace + survie*0.3, passé par tanh() pour borner.
    /// 4. Confiance élevée quand le danger ou l'urgence sont clairs.
    ///
    /// # Paramètres
    /// - `stimulus` : entrée sensorielle avec ses scores perceptuels.
    /// - `chemistry` : état chimique (cortisol et adrénaline amplifient la menace).
    ///
    /// # Retour
    /// Un `ModuleSignal` avec signal, confiance et raisonnement explicatif.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Menace perçue : le danger brut est amplifié par le cortisol et
        // l'adrénaline ambiants. Cela simule l'hypervigilance : un individu
        // stressé perçoit les menaces de manière amplifiée.
        let threat = stimulus.danger
            * (1.0 + chemistry.cortisol + chemistry.adrenaline * 2.0);

        // Instinct de survie face à l'inconnu : un stimulus peu familier
        // et urgent déclenche une réaction défensive.
        let survival = (1.0 - stimulus.familiarity) * stimulus.urgency;

        // Signal brut : la menace pousse vers le rejet (négatif), tandis
        // que l'instinct de survie a un léger effet positif (agir vite).
        // tanh() borne naturellement le résultat entre -1 et +1.
        let raw = -threat + survival * 0.3;
        let signal = raw.tanh();

        // Confiance : le reptilien est très confiant quand la situation est
        // clairement dangereuse ou urgente (domaine d'expertise).
        // En l'absence de danger, sa confiance est faible (il n'a pas d'avis).
        let confidence = if stimulus.danger > 0.5 || stimulus.urgency > 0.7 {
            0.9 // danger clair = haute confiance
        } else if stimulus.danger > 0.2 {
            0.6 // danger modéré = confiance moyenne
        } else {
            0.3 // pas de danger = faible confiance
        };

        // Raisonnement textuel : explication de la réponse du reptilien
        let reasoning = if threat > 1.0 {
            format!("DANGER ÉLEVÉ détecté (menace={:.2}). Instinct de fuite activé.", threat)
        } else if threat > 0.5 {
            format!("Menace modérée (menace={:.2}). Vigilance accrue.", threat)
        } else if survival > 0.5 {
            format!("Situation inconnue urgente (survie={:.2}). Prudence.", survival)
        } else {
            "Pas de danger immédiat. Le reptilien est calme.".to_string()
        };

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
