// =============================================================================
// limbic.rs — Module limbique : émotions, récompense, liens sociaux
// =============================================================================
//
// Rôle : Ce fichier implémente le module limbique de Saphire, inspiré du
// système limbique dans le modèle du cerveau triunique de Paul MacLean.
// Il gère les réactions émotionnelles : peur (amygdale), plaisir (circuit
// de récompense), attachement social (ocytocine) et résilience (endorphines).
//
// Dépendances :
//   - crate::neurochemistry::NeuroChemicalState : état chimique (dopamine,
//     ocytocine, sérotonine, endorphine influencent le traitement)
//   - crate::stimulus::Stimulus : entrée sensorielle (danger, reward, social)
//   - super::BrainModule, ModuleSignal : trait et type de sortie communs
//
// Place dans l'architecture :
//   Deuxième des 3 modules cérébraux. Son signal reflète la réaction
//   émotionnelle au stimulus. Son poids dans le consensus augmente quand
//   la dopamine, la sérotonine et l'ocytocine sont élevées.
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// Le système limbique — traitement émotionnel, empathie et circuit de récompense.
/// Ce module représente les réponses émotionnelles de Saphire, incluant :
/// - L'amygdale (réaction de peur face au danger)
/// - Le circuit de récompense (attraction vers le plaisir)
/// - Le lien social (empathie, attachement)
/// - La résilience émotionnelle (endorphines)
pub struct LimbicModule;

impl BrainModule for LimbicModule {
    /// Retourne le nom du module : "Limbique".
    fn name(&self) -> &str {
        "Limbique"
    }

    /// Traite un stimulus du point de vue émotionnel.
    ///
    /// Algorithme en 4 composantes :
    /// 1. Amygdale : réaction négative proportionnelle au danger (facteur 0.8).
    /// 2. Circuit de récompense : attraction proportionnelle à la récompense,
    ///    amplifiée par la dopamine.
    /// 3. Lien social : composante sociale amplifiée par l'ocytocine.
    /// 4. Résilience : les endorphines atténuent la douleur émotionnelle.
    ///    Le signal final intègre aussi la sérotonine (bien-être de fond).
    ///
    /// # Paramètres
    /// - `stimulus` : entrée sensorielle avec ses scores perceptuels.
    /// - `chemistry` : état chimique (dopamine, ocytocine, sérotonine,
    ///   endorphine modulent la réponse).
    ///
    /// # Retour
    /// Un `ModuleSignal` avec signal, confiance et raisonnement explicatif.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Amygdale : réaction émotionnelle instinctive au danger.
        // Le signe négatif traduit le rejet/la peur. Le facteur 0.8 est
        // légèrement inférieur à 1.0 car le limbique est moins « brutal »
        // que le reptilien face au danger.
        let amygdala = -stimulus.danger * 0.8;

        // Circuit de récompense : la dopamine amplifie l'attrait de la
        // récompense. Quand la dopamine est élevée, même une récompense
        // modeste devient très attirante (biais motivationnel).
        let reward = stimulus.reward * (1.0 + chemistry.dopamine);

        // Lien social : l'ocytocine amplifie la sensibilité aux interactions
        // sociales. Base de 0.5 pour garantir un minimum de réceptivité sociale.
        let social = stimulus.social * (0.5 + chemistry.oxytocin * 0.5);

        // Résilience : les endorphines atténuent la douleur émotionnelle
        // et ajoutent un léger biais positif (capacité à encaisser).
        let resilience = chemistry.endorphin * 0.2;

        // Signal brut : somme des 4 composantes + bien-être de fond (sérotonine).
        // tanh() borne naturellement le résultat entre -1 et +1.
        let raw = amygdala + reward + social + chemistry.serotonin * 0.3 + resilience;
        let signal = raw.tanh();

        // Confiance fixe à 0.7 : le limbique est toujours assez confiant
        // car les émotions sont par nature ressenties avec certitude
        // (on ne doute pas de ce qu'on ressent).
        let confidence = 0.7;

        // Raisonnement : construction dynamique en listant les composantes
        // significatives (au-dessus de leur seuil respectif).
        let parts: Vec<String> = [
            if amygdala.abs() > 0.2 {
                Some(format!("Amygdale réactive ({:.2})", amygdala))
            } else { None },
            if reward > 0.3 {
                Some(format!("Récompense attirante ({:.2})", reward))
            } else { None },
            if social > 0.2 {
                Some(format!("Lien social ressenti ({:.2})", social))
            } else { None },
        ].into_iter().flatten().collect();

        let reasoning = if parts.is_empty() {
            "Le limbique est neutre — pas de forte émotion.".to_string()
        } else {
            parts.join(". ") + "."
        };

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
