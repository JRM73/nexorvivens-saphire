// =============================================================================
// spine/motor.rs — Spinal cord motor relay
//
// Role: Translates reflexes into motor commands for effectors
// (virtual body, Sensoria). The motor relay does not modify the body
// directly — it generates commands that the caller will apply.
//
// Biological analogy:
//   Spinal cord motor neurons transmit commands to muscles. Here,
//   MotorRelay transmits the body effects of reflexes to the VirtualBody.
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::body::VirtualBody;
use super::reflex::{ReflexResult, BodyEffect};

/// Motor command to apply to the body or an effector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorCommand {
    /// Command target
    pub target: MotorTarget,
    /// Command intensity [0.0, 1.0]
    pub intensity: f64,
}

/// Motor command target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorTarget {
    /// Increase heart rate
    HeartRateUp,
    /// Decrease heart rate
    HeartRateDown,
    /// Increase muscle tension
    TensionUp,
    /// Decrease muscle tension
    TensionDown,
    /// Increase body warmth (flush, blushing)
    WarmthUp,
    /// Increase available energy
    EnergyUp,
    /// Decrease energy (fatigue, exhaustion)
    EnergyDown,
}

/// Motor relay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorRelay {
    /// Number of commands issued since startup
    pub total_commands: u64,
}

impl MotorRelay {
    pub fn new() -> Self {
        Self {
            total_commands: 0,
        }
    }

    /// Generates motor commands from the triggered reflexes.
    ///
    /// Each `BodyEffect` in a reflex is translated into a `MotorCommand`
    /// with an intensity proportional to the reflex intensity.
    pub fn generate_commands(
        &mut self,
        reflexes: &[ReflexResult],
        _body: &VirtualBody,
    ) -> Vec<MotorCommand> {
        let mut commands = Vec::new();

        for reflex in reflexes {
            for &(effect, delta) in &reflex.body_effects {
                let effective_delta = delta * reflex.intensity;
                let cmd = match effect {
                    BodyEffect::HeartRate => {
                        if effective_delta > 0.0 {
                            MotorCommand {
                                target: MotorTarget::HeartRateUp,
                                intensity: effective_delta,
                            }
                        } else {
                            MotorCommand {
                                target: MotorTarget::HeartRateDown,
                                intensity: effective_delta.abs(),
                            }
                        }
                    }
                    BodyEffect::Tension => {
                        if effective_delta > 0.0 {
                            MotorCommand {
                                target: MotorTarget::TensionUp,
                                intensity: effective_delta,
                            }
                        } else {
                            MotorCommand {
                                target: MotorTarget::TensionDown,
                                intensity: effective_delta.abs(),
                            }
                        }
                    }
                    BodyEffect::Warmth => {
                        MotorCommand {
                            target: MotorTarget::WarmthUp,
                            intensity: effective_delta,
                        }
                    }
                    BodyEffect::Energy => {
                        if effective_delta > 0.0 {
                            MotorCommand {
                                target: MotorTarget::EnergyUp,
                                intensity: effective_delta,
                            }
                        } else {
                            MotorCommand {
                                target: MotorTarget::EnergyDown,
                                intensity: effective_delta.abs(),
                            }
                        }
                    }
                };
                commands.push(cmd);
            }
        }

        self.total_commands += commands.len() as u64;
        commands
    }

    /// Applies motor commands to the virtual body.
    ///
    /// Effects are applied to `body.soma` (somatic signals).
    /// Heart rate is not modified directly here because it is already
    /// modulated by chemistry via `Heart::update()`. The HeartRate effect
    /// translates into a tension delta (perceived cardiac stress).
    ///
    /// This method is separated from `generate_commands` to allow the
    /// caller to filter or modify commands before application.
    pub fn apply_commands(commands: &[MotorCommand], body: &mut VirtualBody) {
        for cmd in commands {
            match cmd.target {
                MotorTarget::HeartRateUp => {
                    // Heart rate already accelerates via chemistry (adrenaline/cortisol).
                    // The additional effect translates into perceived tension.
                    body.soma.tension = (body.soma.tension + cmd.intensity * 0.5).min(1.0);
                }
                MotorTarget::HeartRateDown => {
                    body.soma.tension = (body.soma.tension - cmd.intensity * 0.3).max(0.0);
                }
                MotorTarget::TensionUp => {
                    body.soma.tension = (body.soma.tension + cmd.intensity).min(1.0);
                }
                MotorTarget::TensionDown => {
                    body.soma.tension = (body.soma.tension - cmd.intensity).max(0.0);
                }
                MotorTarget::WarmthUp => {
                    body.soma.warmth = (body.soma.warmth + cmd.intensity).min(1.0);
                }
                MotorTarget::EnergyUp => {
                    body.soma.energy = (body.soma.energy + cmd.intensity).min(1.0);
                }
                MotorTarget::EnergyDown => {
                    body.soma.energy = (body.soma.energy - cmd.intensity).max(0.0);
                }
            }
        }
    }
}

impl Default for MotorRelay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::Molecule;
    use crate::config::PhysiologyConfig;
    use super::super::reflex::{ReflexResult, ReflexType, BodyEffect};

    fn test_body() -> VirtualBody {
        VirtualBody::new(70.0, &PhysiologyConfig::default())
    }

    #[test]
    fn test_motor_generate_commands() {
        let mut motor = MotorRelay::new();
        let body = test_body();
        let reflexes = vec![ReflexResult {
            reflex_type: ReflexType::ThreatResponse,
            intensity: 0.8,
            chemistry_deltas: vec![(Molecule::Cortisol, 0.15)],
            body_effects: vec![
                (BodyEffect::HeartRate, 0.20),
                (BodyEffect::Tension, 0.15),
            ],
        }];
        let commands = motor.generate_commands(&reflexes, &body);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].target, MotorTarget::HeartRateUp);
        assert_eq!(commands[1].target, MotorTarget::TensionUp);
    }

    #[test]
    fn test_motor_apply_commands() {
        let mut body = test_body();
        let tension_before = body.soma.tension;
        let commands = vec![
            MotorCommand {
                target: MotorTarget::HeartRateUp,
                intensity: 0.15,
            },
        ];
        MotorRelay::apply_commands(&commands, &mut body);
        assert!(body.soma.tension > tension_before, "HeartRateUp should increase soma tension");
    }

    #[test]
    fn test_motor_energy_down() {
        let mut motor = MotorRelay::new();
        let body = test_body();
        let reflexes = vec![ReflexResult {
            reflex_type: ReflexType::SeparationResponse,
            intensity: 0.6,
            chemistry_deltas: vec![],
            body_effects: vec![
                (BodyEffect::Energy, -0.10),
            ],
        }];
        let commands = motor.generate_commands(&reflexes, &body);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].target, MotorTarget::EnergyDown);
    }
}
