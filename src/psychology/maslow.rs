// =============================================================================
// psychology/maslow.rs — Pyramide de Maslow (5 niveaux de besoins)
//
// Modelise la hierarchie des besoins :
//   1. Physiologique (energie, vitalite, chimie stable)
//   2. Securite (cortisol bas, resilience, instinct de survie)
//   3. Appartenance (oxytocine, conversation, pas de solitude)
//   4. Estime (lecons confirmees, desirs accomplis, conscience)
//   5. Actualisation (phi, flow, emotions positives, ethique)
//
// Le niveau actif est le plus haut dont le precedent est satisfait.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Un indicateur contribuant a la satisfaction d'un niveau.
#[derive(Debug, Clone, Serialize)]
pub struct MaslowIndicator {
    /// Nom de l'indicateur
    pub name: String,
    /// Valeur actuelle (0.0 - 1.0)
    pub value: f64,
    /// Poids dans le calcul de satisfaction
    pub weight: f64,
}

/// Un niveau de la pyramide de Maslow.
#[derive(Debug, Clone, Serialize)]
pub struct MaslowLevel {
    /// Nom du niveau
    pub name: String,
    /// Satisfaction actuelle (0.0 - 1.0)
    pub satisfaction: f64,
    /// Seuil de satisfaction pour considerer le niveau comme "acquis"
    pub threshold: f64,
    /// Indicateurs qui contribuent a ce niveau
    pub indicators: Vec<MaslowIndicator>,
}

/// Pyramide de Maslow complete pour Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct MaslowPyramid {
    /// Les 5 niveaux de besoins
    pub levels: Vec<MaslowLevel>,
    /// Index du niveau actuellement actif (0-4)
    pub current_active_level: usize,
}

impl MaslowPyramid {
    /// Cree une pyramide initiale avec des valeurs de base.
    pub fn new() -> Self {
        Self {
            levels: vec![
                MaslowLevel {
                    name: "Physiologique".into(),
                    satisfaction: 0.5,
                    threshold: 0.6,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Securite".into(),
                    satisfaction: 0.5,
                    threshold: 0.5,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Appartenance".into(),
                    satisfaction: 0.3,
                    threshold: 0.5,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Estime".into(),
                    satisfaction: 0.2,
                    threshold: 0.5,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Actualisation".into(),
                    satisfaction: 0.0,
                    threshold: 1.0, // Jamais completement atteint
                    indicators: Vec::new(),
                },
            ],
            current_active_level: 0,
        }
    }

    /// Recalcule la satisfaction de chaque niveau et le niveau actif.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Niveau 1 : Physiologique ────────────────────
        {
            let level = &mut self.levels[0];
            let chimie_stable = 1.0 - (input.cortisol - 0.2).abs().min(0.5) * 2.0;
            level.indicators = vec![
                MaslowIndicator { name: "Energie".into(), value: input.body_energy, weight: 0.35 },
                MaslowIndicator { name: "Vitalite".into(), value: input.body_vitality, weight: 0.35 },
                MaslowIndicator { name: "Chimie stable".into(), value: chimie_stable.max(0.0), weight: 0.3 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Niveau 2 : Securite ─────────────────────────
        {
            let level = &mut self.levels[1];
            level.indicators = vec![
                MaslowIndicator { name: "Cortisol bas".into(), value: 1.0 - input.cortisol, weight: 0.35 },
                MaslowIndicator { name: "Resilience".into(), value: input.healing_resilience, weight: 0.35 },
                MaslowIndicator { name: "Instinct survie".into(), value: input.survival_drive, weight: 0.3 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Niveau 3 : Appartenance ─────────────────────
        {
            let level = &mut self.levels[2];
            let conversation_val = if input.in_conversation { 1.0 } else { 0.0 };
            let no_loneliness = if input.has_loneliness { 0.0 } else { 1.0 };
            level.indicators = vec![
                MaslowIndicator { name: "Oxytocine".into(), value: input.oxytocin, weight: 0.35 },
                MaslowIndicator { name: "En conversation".into(), value: conversation_val, weight: 0.35 },
                MaslowIndicator { name: "Pas de solitude".into(), value: no_loneliness, weight: 0.3 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Niveau 4 : Estime ───────────────────────────
        {
            let level = &mut self.levels[3];
            let lessons_val = (input.learning_confirmed_count as f64 / 10.0).min(1.0);
            let desires_val = if input.desires_active_count > 0 {
                input.desires_fulfilled_count as f64
                    / (input.desires_active_count + input.desires_fulfilled_count).max(1) as f64
            } else {
                0.3
            };
            level.indicators = vec![
                MaslowIndicator { name: "Lecons confirmees".into(), value: lessons_val, weight: 0.35 },
                MaslowIndicator { name: "Desirs accomplis".into(), value: desires_val, weight: 0.3 },
                MaslowIndicator { name: "Conscience".into(), value: input.consciousness_level, weight: 0.35 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Niveau 5 : Actualisation ────────────────────
        {
            let level = &mut self.levels[4];
            let emotions_pos = if input.emotion_valence > 0.3 { input.emotion_valence } else { 0.0 };
            let ethics_val = (input.ethics_active_count as f64 / 5.0).min(1.0);
            let flow_val = if input.in_flow { 1.0 } else { 0.0 };
            level.indicators = vec![
                MaslowIndicator { name: "Phi (IIT)".into(), value: input.phi, weight: 0.3 },
                MaslowIndicator { name: "Etat de flow".into(), value: flow_val, weight: 0.25 },
                MaslowIndicator { name: "Emotions positives".into(), value: emotions_pos, weight: 0.2 },
                MaslowIndicator { name: "Ethique".into(), value: ethics_val, weight: 0.25 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Determiner le niveau actif ──────────────────
        // Le niveau actif est le plus haut ou le precedent est satisfait
        self.current_active_level = 0;
        for i in 1..5 {
            if self.levels[i - 1].satisfaction >= self.levels[i - 1].threshold {
                self.current_active_level = i;
            } else {
                break;
            }
        }
    }

    /// Description concise pour le prompt LLM.
    pub fn describe(&self) -> String {
        let level = &self.levels[self.current_active_level];
        if self.current_active_level == 0 && level.satisfaction > 0.5 {
            return String::new(); // Etat banal, pas besoin de le decrire
        }
        format!(
            "Maslow : niveau {} ({}) satisfaction {:.0}%",
            self.current_active_level + 1,
            level.name,
            level.satisfaction * 100.0
        )
    }
}
