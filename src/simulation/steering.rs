// =============================================================================
// steering.rs — Steering Behaviors pour la navigation emotionnelle
//
// Role : Applique les Steering Behaviors (Craig Reynolds) dans l'espace
//        valence-arousal pour guider la regulation emotionnelle de Saphire.
//
// Comportements disponibles :
//   - Seek : tendre vers un etat emotionnel cible
//   - Flee : fuir un etat emotionnel negatif
//   - Wander : exploration aleatoire de l'espace emotionnel
//   - Arrive : deceleation progressive en approchant la cible
//
// L'espace emotionnel est 2D : (valence [-1,+1], arousal [0,1])
// Les forces calculees sont traduites en ajustements chimiques.
//
// Place dans l'architecture :
//   Consulte dans le pipeline apres le calcul emotionnel pour produire
//   des forces de regulation qui modifient la chimie vers l'equilibre.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Position dans l'espace emotionnel 2D (valence, arousal).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionalPos {
    /// Valence [-1, +1] : plaisant ↔ deplaisant
    pub valence: f64,
    /// Arousal [0, 1] : calme ↔ active
    pub arousal: f64,
}

impl EmotionalPos {
    pub fn new(valence: f64, arousal: f64) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
        }
    }

    /// Distance euclidienne vers une autre position.
    pub fn distance_to(&self, other: &EmotionalPos) -> f64 {
        let dv = self.valence - other.valence;
        let da = self.arousal - other.arousal;
        (dv * dv + da * da).sqrt()
    }
}

/// Force directrice 2D dans l'espace emotionnel.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SteeringForce {
    /// Composante valence (positif = vers plaisir, negatif = vers deplaisir)
    pub dv: f64,
    /// Composante arousal (positif = activation, negatif = calme)
    pub da: f64,
}

impl SteeringForce {
    pub fn zero() -> Self { Self { dv: 0.0, da: 0.0 } }

    pub fn magnitude(&self) -> f64 {
        (self.dv * self.dv + self.da * self.da).sqrt()
    }

    /// Additionne deux forces.
    pub fn add(&self, other: &SteeringForce) -> SteeringForce {
        SteeringForce {
            dv: self.dv + other.dv,
            da: self.da + other.da,
        }
    }

    /// Limite la magnitude de la force.
    pub fn truncate(&self, max: f64) -> SteeringForce {
        let mag = self.magnitude();
        if mag > max && mag > 1e-10 {
            SteeringForce {
                dv: self.dv * max / mag,
                da: self.da * max / mag,
            }
        } else {
            *self
        }
    }
}

/// Parametres des steering behaviors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringParams {
    /// Force maximale appliquee par cycle
    pub max_force: f64,
    /// Rayon de ralentissement pour Arrive
    pub arrive_radius: f64,
    /// Rayon de fuite pour Flee
    pub flee_radius: f64,
    /// Amplitude du Wander
    pub wander_strength: f64,
    /// Cible d'equilibre emotionnel (etat souhaite)
    pub equilibrium: EmotionalPos,
}

impl Default for SteeringParams {
    fn default() -> Self {
        Self {
            max_force: 0.05,
            arrive_radius: 0.3,
            flee_radius: 0.4,
            wander_strength: 0.02,
            // Equilibre : legerement positif, arousal modere
            equilibrium: EmotionalPos::new(0.2, 0.35),
        }
    }
}

/// Seek : force vers une position cible.
pub fn seek(current: &EmotionalPos, target: &EmotionalPos) -> SteeringForce {
    SteeringForce {
        dv: target.valence - current.valence,
        da: target.arousal - current.arousal,
    }
}

/// Flee : force eloignant d'une position (avec rayon de fuite).
pub fn flee(current: &EmotionalPos, threat: &EmotionalPos, radius: f64) -> SteeringForce {
    let dist = current.distance_to(threat);
    if dist > radius || dist < 1e-10 {
        return SteeringForce::zero();
    }
    let strength = 1.0 - (dist / radius);
    SteeringForce {
        dv: (current.valence - threat.valence) * strength,
        da: (current.arousal - threat.arousal) * strength,
    }
}

/// Arrive : seek avec deceleration progressive.
pub fn arrive(current: &EmotionalPos, target: &EmotionalPos, radius: f64) -> SteeringForce {
    let dist = current.distance_to(target);
    if dist < 1e-10 {
        return SteeringForce::zero();
    }
    let speed_factor = if dist < radius {
        dist / radius // Deceleration dans le rayon
    } else {
        1.0
    };
    SteeringForce {
        dv: (target.valence - current.valence) * speed_factor,
        da: (target.arousal - current.arousal) * speed_factor,
    }
}

/// Wander : mouvement aleatoire dans l'espace emotionnel.
/// Utilise un compteur de cycle pour la pseudo-aleatoire.
pub fn wander(cycle: u64, strength: f64) -> SteeringForce {
    // Pseudo-aleatoire simple base sur le cycle
    let angle = ((cycle * 7919) % 360) as f64 * std::f64::consts::PI / 180.0;
    SteeringForce {
        dv: angle.cos() * strength,
        da: angle.sin().abs() * strength * 0.5, // Arousal ne descend pas trop
    }
}

/// Moteur de steering complet : combine seek/flee/arrive/wander
/// pour calculer la force de regulation emotionnelle.
pub struct SteeringEngine {
    pub params: SteeringParams,
}

impl SteeringEngine {
    pub fn new(params: SteeringParams) -> Self {
        Self { params }
    }

    /// Calcule la force de regulation totale.
    /// Combine :
    ///   - Arrive vers l'equilibre (tendance naturelle)
    ///   - Flee des zones de stress extreme (cortisol eleve)
    ///   - Wander pour l'exploration emotionnelle
    pub fn compute_regulation(
        &self,
        current: &EmotionalPos,
        cycle: u64,
        cortisol: f64,
    ) -> SteeringForce {
        // Force principale : Arrive vers l'equilibre
        let equilibrium_force = arrive(
            current,
            &self.params.equilibrium,
            self.params.arrive_radius,
        );

        // Flee : fuir les zones de stress extreme
        let stress_zone = EmotionalPos::new(-0.8, 0.9);
        let flee_force = if cortisol > 0.6 {
            flee(current, &stress_zone, self.params.flee_radius)
        } else {
            SteeringForce::zero()
        };

        // Wander : exploration emotionnelle legere
        let wander_force = wander(cycle, self.params.wander_strength);

        // Combiner les forces avec poids
        let combined = equilibrium_force
            .add(&flee_force)
            .add(&wander_force)
            .truncate(self.params.max_force);

        combined
    }

    /// Traduit la force emotionnelle en ajustements chimiques.
    /// dv positif → boost serotonine/dopamine, baisse cortisol
    /// da positif → boost noradrenaline/adrenaline
    pub fn force_to_chemistry(&self, force: &SteeringForce) -> ChemistryAdjustment {
        ChemistryAdjustment {
            dopamine: force.dv * 0.3,
            cortisol: -force.dv * 0.2,
            serotonin: force.dv * 0.2,
            adrenaline: force.da * 0.15,
            noradrenaline: force.da * 0.1,
            oxytocin: 0.0,
            endorphin: if force.dv > 0.0 { force.dv * 0.1 } else { 0.0 },
        }
    }
}

impl Default for SteeringEngine {
    fn default() -> Self {
        Self::new(SteeringParams::default())
    }
}

/// Ajustements chimiques produits par le steering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemistryAdjustment {
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub noradrenaline: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seek_positive() {
        let current = EmotionalPos::new(0.0, 0.5);
        let target = EmotionalPos::new(0.5, 0.5);
        let force = seek(&current, &target);
        assert!(force.dv > 0.0, "Seek vers valence positive doit pousser dv > 0");
    }

    #[test]
    fn test_flee_in_radius() {
        let current = EmotionalPos::new(-0.5, 0.8);
        let threat = EmotionalPos::new(-0.6, 0.9);
        let force = flee(&current, &threat, 0.5);
        assert!(force.magnitude() > 0.0, "Flee doit generer une force dans le rayon");
    }

    #[test]
    fn test_flee_out_of_radius() {
        let current = EmotionalPos::new(0.5, 0.3);
        let threat = EmotionalPos::new(-0.8, 0.9);
        let force = flee(&current, &threat, 0.3);
        assert!(force.magnitude() < 0.001, "Flee ne doit pas agir hors du rayon");
    }

    #[test]
    fn test_arrive_deceleration() {
        let target = EmotionalPos::new(0.2, 0.35);
        let far = EmotionalPos::new(-0.5, 0.8);
        let close = EmotionalPos::new(0.15, 0.4);
        let force_far = arrive(&far, &target, 0.3);
        let force_close = arrive(&close, &target, 0.3);
        assert!(force_far.magnitude() > force_close.magnitude(),
            "Arrive doit decelerer pres de la cible");
    }

    #[test]
    fn test_regulation_produces_force() {
        let engine = SteeringEngine::default();
        let pos = EmotionalPos::new(-0.5, 0.8);
        let force = engine.compute_regulation(&pos, 100, 0.7);
        assert!(force.magnitude() > 0.0, "La regulation doit produire une force");
    }

    #[test]
    fn test_force_to_chemistry() {
        let engine = SteeringEngine::default();
        let positive_force = SteeringForce { dv: 0.05, da: 0.02 };
        let adj = engine.force_to_chemistry(&positive_force);
        assert!(adj.dopamine > 0.0, "Force positive doit booster dopamine");
        assert!(adj.cortisol < 0.0, "Force positive doit reduire cortisol");
    }

    #[test]
    fn test_truncate_force() {
        let big = SteeringForce { dv: 1.0, da: 1.0 };
        let truncated = big.truncate(0.1);
        assert!(truncated.magnitude() <= 0.1 + 0.001);
    }
}
