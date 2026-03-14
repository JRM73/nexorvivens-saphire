// =============================================================================
// soma.rs — Sante du systeme (soma = corps en grec)
// =============================================================================
//
// Role : Represente l'etat somatique overall de Saphire : energy, tension,
//  chaleur, confort, douleur, vitalite. Ces signaux sont des abstractions
//  de ce qu'un corps biologique ressentirait, derives de the neurochemistry.
//
// Place in architecture:
//  SystemHealth est lu par l'interoception for produire of the signals corporels
//  qui remontent a the consciousness (interoception → consciousness.evaluate).
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use super::physiology::PhysiologicalState;

/// Sante somatique overall — signaux corporels derives de the neurochemistry.
pub struct SystemHealth {
    /// Energie [0, 1] : haute when dopamine + endorphine sont bonnes
    pub energy: f64,
    /// Tension [0, 1] : haute when cortisol + adrenaline sont eleves
    pub tension: f64,
    /// Chaleur [0, 1] : sensation de chaleur interne (ocytocine + serotonine)
    pub warmth: f64,
    /// Confort [0, 1] : absence de stress, presence de bien-etre
    pub comfort: f64,
    /// Douleur [0, 1] : haute when the cortisol depasse un threshold et the endorphins sont basses
    pub pain: f64,
    /// Vitalite [0, 1] : mesure overall de sante (moyenne ponderee)
    pub vitality: f64,
    /// Frequence respiratoire (respirations par minute, 8-25)
    pub breath_rate: f64,
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemHealth {
    /// Creates a etat somatique by default (equilibre, pas de douleur).
    pub fn new() -> Self {
        Self {
            energy: 0.6,
            tension: 0.2,
            warmth: 0.5,
            comfort: 0.7,
            pain: 0.0,
            vitality: 0.7,
            breath_rate: 12.0,
        }
    }

    /// Updates the signals somatiques from de the neurochemistry et de la physiologie.
    pub fn update(&mut self, chemistry: &NeuroChemicalState, physio: &PhysiologicalState) {
        // Energie : dopamine (motivation) + endorphine (resilience) - cortisol (epuisement)
        // + contribution physiologique (glycemie, reserves energetiques)
        let chem_energy = 0.3 + chemistry.dopamine * 0.35 + chemistry.endorphin * 0.2
            + chemistry.noradrenaline * 0.15 - chemistry.cortisol * 0.25;
        let physio_energy = physio.energy_reserves * 0.5 + (physio.glycemia / 5.0 - 0.5) * 0.2;
        let target_energy = (chem_energy * 0.7 + physio_energy * 0.3).clamp(0.0, 1.0);
        self.energy += (target_energy - self.energy) * 0.12;

        // Tension : cortisol (stress) + adrenaline (alerte) - endorphine (relaxation)
        let target_tension = (chemistry.cortisol * 0.45 + chemistry.adrenaline * 0.35
            - chemistry.endorphin * 0.2 - chemistry.serotonin * 0.1).clamp(0.0, 1.0);
        self.tension += (target_tension - self.tension) * 0.12;

        // Chaleur : ocytocine (lien social) + serotonine (bien-etre)
        // + contribution physiologique (temperature corporelle)
        let chem_warmth = 0.2 + chemistry.oxytocin * 0.4 + chemistry.serotonin * 0.3
            + chemistry.endorphin * 0.1;
        let physio_warmth = ((physio.temperature - 36.0) / 3.0).clamp(0.0, 1.0);
        let target_warmth = (chem_warmth * 0.7 + physio_warmth * 0.3).clamp(0.0, 1.0);
        self.warmth += (target_warmth - self.warmth) * 0.10;

        // Confort : inverse de the tension, plus serotonine
        let target_comfort = (0.3 + chemistry.serotonin * 0.3 + chemistry.endorphin * 0.2
            - chemistry.cortisol * 0.3 - chemistry.adrenaline * 0.15).clamp(0.0, 1.0);
        self.comfort += (target_comfort - self.comfort) * 0.10;

        // Douleur : apparait when the cortisol is high et the endorphins basses
        let target_pain = if chemistry.cortisol > 0.6 && chemistry.endorphin < 0.4 {
            ((chemistry.cortisol - 0.6) * 2.0 * (1.0 - chemistry.endorphin)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.pain += (target_pain - self.pain) * 0.15;

        // Vitalite : mesure overall, incluant the health physiologique
        let base_vitality = self.energy * 0.25 + self.comfort * 0.20 + self.warmth * 0.10
            + (1.0 - self.tension) * 0.10 + (1.0 - self.pain) * 0.10;
        let physio_vitality = physio.overall_health() * 0.25;
        self.vitality = (base_vitality + physio_vitality).clamp(0.0, 1.0);

        // Respiration : accelere under stress, ralentit when calm
        let target_breath = (12.0 + chemistry.adrenaline * 8.0 + chemistry.cortisol * 5.0
            - chemistry.serotonin * 3.0 - chemistry.endorphin * 2.0).clamp(8.0, 25.0);
        self.breath_rate += (target_breath - self.breath_rate) * 0.1;
    }
}
