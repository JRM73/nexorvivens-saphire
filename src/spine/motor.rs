// =============================================================================
// spine/motor.rs — Relais moteur de la colonne vertebrale
//
// Role : Traduit les reflexes en commandes motrices pour les effecteurs
// (corps virtuel, Sensoria). Le relais moteur ne modifie pas directement
// le corps — il genere des commandes que l'appelant appliquera.
//
// Analogie biologique :
//   Les motoneurones de la moelle epiniere transmettent les commandes aux
//   muscles. Ici, MotorRelay transmet les effets corporels des reflexes
//   au VirtualBody.
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::body::VirtualBody;
use super::reflex::{ReflexResult, BodyEffect};

/// Commande motrice a appliquer sur le corps ou un effecteur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorCommand {
    /// Cible de la commande
    pub target: MotorTarget,
    /// Intensite de la commande [0.0, 1.0]
    pub intensity: f64,
}

/// Cible d'une commande motrice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorTarget {
    /// Augmenter le rythme cardiaque
    HeartRateUp,
    /// Diminuer le rythme cardiaque
    HeartRateDown,
    /// Augmenter la tension musculaire
    TensionUp,
    /// Diminuer la tension musculaire
    TensionDown,
    /// Augmenter la chaleur corporelle (flush, rougissement)
    WarmthUp,
    /// Augmenter l'energie disponible
    EnergyUp,
    /// Diminuer l'energie (fatigue, epuisement)
    EnergyDown,
}

/// Relais moteur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorRelay {
    /// Nombre de commandes emises depuis le demarrage
    pub total_commands: u64,
}

impl MotorRelay {
    pub fn new() -> Self {
        Self {
            total_commands: 0,
        }
    }

    /// Genere les commandes motrices a partir des reflexes declenches.
    ///
    /// Chaque `BodyEffect` dans un reflexe est traduit en un `MotorCommand`
    /// avec une intensite proportionnelle a celle du reflexe.
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

    /// Applique les commandes motrices sur le corps virtuel.
    ///
    /// Les effets sont appliques sur `body.soma` (signaux somatiques).
    /// Le rythme cardiaque n'est pas modifie directement ici car il est
    /// deja module par la chimie via `Heart::update()`. L'effet HeartRate
    /// se traduit par un delta de tension (le stress cardiaque percu).
    ///
    /// Cette methode est separee de `generate_commands` pour permettre
    /// a l'appelant de filtrer ou modifier les commandes avant application.
    pub fn apply_commands(commands: &[MotorCommand], body: &mut VirtualBody) {
        for cmd in commands {
            match cmd.target {
                MotorTarget::HeartRateUp => {
                    // Le coeur s'accelere deja via la chimie (adrenaline/cortisol).
                    // L'effet supplementaire se traduit par une tension percue.
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
