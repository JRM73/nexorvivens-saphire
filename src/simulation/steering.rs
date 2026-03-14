// =============================================================================
// steering.rs — Steering Behaviors for emotional navigation
//
// Role: Applies Steering Behaviors (Craig Reynolds) in the valence-arousal
//       space to guide Saphire's emotional regulation.
//
// Available behaviors:
//   - Seek: tend toward a target emotional state
//   - Flee: flee from a negative emotional state
//   - Wander: random exploration of emotional space
//   - Arrive: progressive deceleration when approaching the target
//
// The emotional space is 2D: (valence [-1,+1], arousal [0,1])
// The computed forces are translated into chemistry adjustments.
//
// Place in the architecture:
//   Consulted in the pipeline after emotional computation to produce
//   regulation forces that modify chemistry toward equilibrium.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Position in 2D emotional space (valence, arousal).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionalPos {
    /// Valence [-1, +1]: pleasant <-> unpleasant
    pub valence: f64,
    /// Arousal [0, 1]: calm <-> activated
    pub arousal: f64,
}

impl EmotionalPos {
    pub fn new(valence: f64, arousal: f64) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
        }
    }

    /// Euclidean distance to another position.
    pub fn distance_to(&self, other: &EmotionalPos) -> f64 {
        let dv = self.valence - other.valence;
        let da = self.arousal - other.arousal;
        (dv * dv + da * da).sqrt()
    }
}

/// 2D steering force in emotional space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SteeringForce {
    /// Valence component (positive = toward pleasure, negative = toward displeasure)
    pub dv: f64,
    /// Arousal component (positive = activation, negative = calm)
    pub da: f64,
}

impl SteeringForce {
    pub fn zero() -> Self { Self { dv: 0.0, da: 0.0 } }

    pub fn magnitude(&self) -> f64 {
        (self.dv * self.dv + self.da * self.da).sqrt()
    }

    /// Adds two forces.
    pub fn add(&self, other: &SteeringForce) -> SteeringForce {
        SteeringForce {
            dv: self.dv + other.dv,
            da: self.da + other.da,
        }
    }

    /// Limits the force magnitude.
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

/// Steering behavior parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringParams {
    /// Maximum force applied per cycle
    pub max_force: f64,
    /// Slowdown radius for Arrive
    pub arrive_radius: f64,
    /// Flee radius for Flee
    pub flee_radius: f64,
    /// Wander amplitude
    pub wander_strength: f64,
    /// Emotional equilibrium target (desired state)
    pub equilibrium: EmotionalPos,
}

impl Default for SteeringParams {
    fn default() -> Self {
        Self {
            max_force: 0.05,
            arrive_radius: 0.3,
            flee_radius: 0.4,
            wander_strength: 0.02,
            // Equilibrium: slightly positive, moderate arousal
            equilibrium: EmotionalPos::new(0.2, 0.35),
        }
    }
}

/// Seek: force toward a target position.
pub fn seek(current: &EmotionalPos, target: &EmotionalPos) -> SteeringForce {
    SteeringForce {
        dv: target.valence - current.valence,
        da: target.arousal - current.arousal,
    }
}

/// Flee: force away from a position (with flee radius).
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

/// Arrive: seek with progressive deceleration.
pub fn arrive(current: &EmotionalPos, target: &EmotionalPos, radius: f64) -> SteeringForce {
    let dist = current.distance_to(target);
    if dist < 1e-10 {
        return SteeringForce::zero();
    }
    let speed_factor = if dist < radius {
        dist / radius // Deceleration within the radius
    } else {
        1.0
    };
    SteeringForce {
        dv: (target.valence - current.valence) * speed_factor,
        da: (target.arousal - current.arousal) * speed_factor,
    }
}

/// Wander: random movement in emotional space.
/// Uses a cycle counter for pseudo-randomness.
pub fn wander(cycle: u64, strength: f64) -> SteeringForce {
    // Simple pseudo-random based on cycle
    let angle = ((cycle * 7919) % 360) as f64 * std::f64::consts::PI / 180.0;
    SteeringForce {
        dv: angle.cos() * strength,
        da: angle.sin().abs() * strength * 0.5, // Arousal doesn't drop too much
    }
}

/// Complete steering engine: combines seek/flee/arrive/wander
/// to compute the emotional regulation force.
pub struct SteeringEngine {
    pub params: SteeringParams,
}

impl SteeringEngine {
    pub fn new(params: SteeringParams) -> Self {
        Self { params }
    }

    /// Computes the total regulation force.
    /// Combines:
    ///   - Arrive toward equilibrium (natural tendency)
    ///   - Flee from extreme stress zones (high cortisol)
    ///   - Wander for emotional exploration
    pub fn compute_regulation(
        &self,
        current: &EmotionalPos,
        cycle: u64,
        cortisol: f64,
    ) -> SteeringForce {
        // Main force: Arrive toward equilibrium
        let equilibrium_force = arrive(
            current,
            &self.params.equilibrium,
            self.params.arrive_radius,
        );

        // Flee: escape extreme stress zones
        let stress_zone = EmotionalPos::new(-0.8, 0.9);
        let flee_force = if cortisol > 0.6 {
            flee(current, &stress_zone, self.params.flee_radius)
        } else {
            SteeringForce::zero()
        };

        // Wander: light emotional exploration
        let wander_force = wander(cycle, self.params.wander_strength);

        // Combine forces with weights
        let combined = equilibrium_force
            .add(&flee_force)
            .add(&wander_force)
            .truncate(self.params.max_force);

        combined
    }

    /// Translates the emotional force into chemistry adjustments.
    /// Positive dv -> boosts serotonin/dopamine, lowers cortisol
    /// Positive da -> boosts noradrenaline/adrenaline
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

/// Chemistry adjustments produced by steering.
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
        assert!(force.dv > 0.0, "Seek toward positive valence should push dv > 0");
    }

    #[test]
    fn test_flee_in_radius() {
        let current = EmotionalPos::new(-0.5, 0.8);
        let threat = EmotionalPos::new(-0.6, 0.9);
        let force = flee(&current, &threat, 0.5);
        assert!(force.magnitude() > 0.0, "Flee should generate a force within radius");
    }

    #[test]
    fn test_flee_out_of_radius() {
        let current = EmotionalPos::new(0.5, 0.3);
        let threat = EmotionalPos::new(-0.8, 0.9);
        let force = flee(&current, &threat, 0.3);
        assert!(force.magnitude() < 0.001, "Flee should not act outside radius");
    }

    #[test]
    fn test_arrive_deceleration() {
        let target = EmotionalPos::new(0.2, 0.35);
        let far = EmotionalPos::new(-0.5, 0.8);
        let close = EmotionalPos::new(0.15, 0.4);
        let force_far = arrive(&far, &target, 0.3);
        let force_close = arrive(&close, &target, 0.3);
        assert!(force_far.magnitude() > force_close.magnitude(),
            "Arrive should decelerate near the target");
    }

    #[test]
    fn test_regulation_produces_force() {
        let engine = SteeringEngine::default();
        let pos = EmotionalPos::new(-0.5, 0.8);
        let force = engine.compute_regulation(&pos, 100, 0.7);
        assert!(force.magnitude() > 0.0, "Regulation should produce a force");
    }

    #[test]
    fn test_force_to_chemistry() {
        let engine = SteeringEngine::default();
        let positive_force = SteeringForce { dv: 0.05, da: 0.02 };
        let adj = engine.force_to_chemistry(&positive_force);
        assert!(adj.dopamine > 0.0, "Positive force should boost dopamine");
        assert!(adj.cortisol < 0.0, "Positive force should reduce cortisol");
    }

    #[test]
    fn test_truncate_force() {
        let big = SteeringForce { dv: 1.0, da: 1.0 };
        let truncated = big.truncate(0.1);
        assert!(truncated.magnitude() <= 0.1 + 0.001);
    }
}
