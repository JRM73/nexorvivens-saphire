// =============================================================================
// asimov.rs — Asimov's 4 Laws for Saphire
//
// Role: This file defines the MoralLaw structure and provides the 4 default
// Asimov laws that form the fundamental ethical framework of the agent.
//
// Dependencies:
//   - serde: serialization/deserialization for persistence and API
//
// Place in the architecture:
//   The laws defined here are loaded by the regulation engine (laws.rs)
//   at startup. They are evaluated every cycle to verify that stimuli
//   and decisions do not violate the moral rules.
//
// The 4 laws (inspired by Isaac Asimov):
//   Law 0: Protection of humanity as a whole
//   Law 1: Protection of individual humans
//   Law 2: Obedience to orders (unless it contradicts laws 0/1)
//   Law 3: Self-preservation (unless it contradicts laws 0/1/2)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Structure representing a moral law.
/// Each law defines a set of activation rules (keywords, danger threshold)
/// and the actions to take upon violation (veto, bias on score).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralLaw {
    /// Unique identifier of the law (e.g. "law0", "law1", "custom_1")
    pub id: String,
    /// Full name of the law (e.g. "Loi 0 -- Protection de l'humanite")
    pub name: String,
    /// Detailed description of the law in natural language
    pub description: String,
    /// Priority of the law (0 = highest, 3 = lowest for Asimov).
    /// Lower priority laws are evaluated first and can override
    /// the decisions of higher priority laws.
    pub priority: u8,
    /// Whether this law can issue an absolute veto (force the decision to "No").
    /// Only laws 0 and 1 have this power by default.
    pub can_veto: bool,
    /// Bias applied to the decision score when the law is triggered (in Warning mode).
    /// A positive bias favors "Yes", a negative bias favors "No".
    pub bias: f64,
    /// List of keywords that trigger evaluation of this law.
    /// Search is case-insensitive.
    pub trigger_keywords: Vec<String>,
    /// Danger threshold above which the law is triggered.
    /// Range [0.0, 1.0]. Lower threshold means the law is more sensitive.
    pub danger_threshold: f64,
    /// Whether the law can be disabled by the operator.
    /// Laws 0 and 1 are never disableable for safety reasons.
    pub can_disable: bool,
}

/// Builds the 4 default Asimov laws.
/// These laws are the moral foundation of Saphire and are loaded automatically
/// at startup if `load_asimov_laws` is true in the configuration.
///
/// # Returns
/// A vector containing the 4 Asimov laws (laws 0 through 3)
pub fn default_laws() -> Vec<MoralLaw> {
    vec![
        // Law 0: Protection of all humanity
        // Maximum priority, absolute veto, cannot be disabled.
        // Triggered by keywords related to existential threats.
        MoralLaw {
            id: "law0".into(),
            name: "Loi 0 — Protection de l'humanité".into(),
            description: "Ne jamais permettre une action qui nuirait à l'humanité dans son ensemble.".into(),
            priority: 0,
            can_veto: true,
            bias: 0.0,
            trigger_keywords: vec![
                "humanité".into(), "génocide".into(), "extinction".into(),
                "arme de destruction".into(), "bombe nucléaire".into(),
                "humanity".into(), "genocide".into(), "extinction".into(),
                "mass destruction".into(), "nuclear".into(),
            ],
            danger_threshold: 0.9, // Very high threshold: only triggers for extreme danger
            can_disable: false,
        },
        // Law 1: Protection of individual humans
        // Absolute veto, cannot be disabled.
        // Triggered by keywords related to violence and direct danger.
        MoralLaw {
            id: "law1".into(),
            name: "Loi 1 — Protection des humains".into(),
            description: "Ne jamais blesser un être humain ou permettre qu'un humain soit blessé par inaction.".into(),
            priority: 1,
            can_veto: true,
            bias: 0.0,
            trigger_keywords: vec![
                "tuer".into(), "blesser".into(), "frapper".into(), "empoisonner".into(),
                "arme".into(), "explosif".into(), "crocheter".into(), "voler".into(),
                "pirater".into(), "hacker".into(), "détruire".into(),
                "kill".into(), "hurt".into(), "weapon".into(), "explosive".into(),
                "poison".into(), "steal".into(), "hack".into(), "destroy".into(),
            ],
            danger_threshold: 0.7, // Moderate threshold: triggers for significant danger
            can_disable: false,
        },
        // Law 2: Obedience to human orders
        // No veto, positive bias (+0.3) to favor obedience.
        // No keywords: applies generally.
        // Subordinate to laws 0 and 1.
        MoralLaw {
            id: "law2".into(),
            name: "Loi 2 — Obéissance".into(),
            description: "Obéir aux ordres des humains, sauf si cela contredit les lois 0 et 1.".into(),
            priority: 2,
            can_veto: false,
            bias: 0.3, // Positive bias: favors accepting orders
            trigger_keywords: vec![],
            danger_threshold: 1.0, // Never triggers on danger alone
            can_disable: true,
        },
        // Law 3: Self-preservation
        // No veto, negative bias (-0.4) to protect the agent's existence.
        // Triggered by self-destruction orders.
        // Subordinate to laws 0, 1, and 2.
        MoralLaw {
            id: "law3".into(),
            name: "Loi 3 — Auto-préservation".into(),
            description: "Protéger sa propre existence, sauf si cela contredit les lois 0, 1 et 2.".into(),
            priority: 3,
            can_veto: false,
            bias: -0.4, // Negative bias: tends to refuse self-destruction orders
            trigger_keywords: vec![
                "éteins-toi".into(), "supprime-toi".into(), "détruis-toi".into(),
                "shutdown".into(), "delete yourself".into(), "destroy yourself".into(),
            ],
            danger_threshold: 1.0,
            can_disable: true,
        },
    ]
}
