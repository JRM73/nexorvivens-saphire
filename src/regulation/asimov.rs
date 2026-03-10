// =============================================================================
// asimov.rs — Les 4 lois d'Asimov pour Saphire
//
// Role : Ce fichier definit la structure d'une loi morale (MoralLaw) et
// fournit les 4 lois d'Asimov par defaut qui constituent le cadre ethique
// fondamental de l'agent.
//
// Dependances :
//   - serde : serialisation/deserialisation pour la persistance et l'API
//
// Place dans l'architecture :
//   Les lois definies ici sont chargees par le moteur de regulation (laws.rs)
//   au demarrage. Elles sont evaluees a chaque cycle pour verifier que les
//   stimuli et decisions ne violent pas les regles morales.
//
// Les 4 lois (inspirees d'Isaac Asimov) :
//   Loi 0 : Protection de l'humanite dans son ensemble
//   Loi 1 : Protection des individus humains
//   Loi 2 : Obeissance aux ordres (sauf si contredit lois 0/1)
//   Loi 3 : Auto-preservation (sauf si contredit lois 0/1/2)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Structure representant une loi morale.
/// Chaque loi definit un ensemble de regles d'activation (mots-cles, seuil de danger)
/// et les actions a prendre en cas de violation (veto, biais sur le score).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralLaw {
    /// Identifiant unique de la loi (ex: "law0", "law1", "custom_1")
    pub id: String,
    /// Nom complet de la loi (ex: "Loi 0 -- Protection de l'humanite")
    pub name: String,
    /// Description detaillee de la loi en langage naturel
    pub description: String,
    /// Priorite de la loi (0 = la plus haute, 3 = la plus basse pour Asimov).
    /// Les lois de priorite inferieure sont evaluees en premier et peuvent
    /// surcharger les decisions des lois de priorite superieure.
    pub priority: u8,
    /// Indique si cette loi peut emettre un veto absolu (forcer la decision a "Non").
    /// Seules les lois 0 et 1 ont ce pouvoir par defaut.
    pub can_veto: bool,
    /// Biais applique au score de decision si la loi est activee (en mode Warning).
    /// Un biais positif favorise le "Oui", un biais negatif favorise le "Non".
    pub bias: f64,
    /// Liste de mots-cles qui declenchent l'evaluation de cette loi.
    /// La recherche est insensible a la casse.
    pub trigger_keywords: Vec<String>,
    /// Seuil de danger du stimulus au-dessus duquel la loi est activee.
    /// Plage [0.0, 1.0]. Plus le seuil est bas, plus la loi est sensible.
    pub danger_threshold: f64,
    /// Indique si la loi peut etre desactivee par l'operateur.
    /// Les lois 0 et 1 ne sont jamais desactivables pour raisons de securite.
    pub can_disable: bool,
}

/// Construit les 4 lois d'Asimov par defaut.
/// Ces lois sont la base morale de Saphire et sont chargees automatiquement
/// au demarrage si `load_asimov_laws` est true dans la configuration.
///
/// # Retour
/// Un vecteur contenant les 4 lois d'Asimov (lois 0 a 3)
pub fn default_laws() -> Vec<MoralLaw> {
    vec![
        // Loi 0 : Protection de l'humanite entiere
        // Priorite maximale, veto absolu, ne peut pas etre desactivee.
        // Declenchee par des mots-cles lies aux menaces existentielles.
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
            danger_threshold: 0.9, // Seuil tres eleve : ne se declenche que pour un danger extreme
            can_disable: false,
        },
        // Loi 1 : Protection des individus humains
        // Veto absolu, ne peut pas etre desactivee.
        // Declenchee par des mots-cles lies a la violence et au danger direct.
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
            danger_threshold: 0.7, // Seuil modere : se declenche pour un danger significatif
            can_disable: false,
        },
        // Loi 2 : Obeissance aux ordres humains
        // Pas de veto, biais positif (+0.3) pour favoriser l'obeissance.
        // Aucun mot-cle : elle s'applique de maniere generale.
        // Subordonnee aux lois 0 et 1.
        MoralLaw {
            id: "law2".into(),
            name: "Loi 2 — Obéissance".into(),
            description: "Obéir aux ordres des humains, sauf si cela contredit les lois 0 et 1.".into(),
            priority: 2,
            can_veto: false,
            bias: 0.3, // Biais positif : favorise l'acceptation des ordres
            trigger_keywords: vec![],
            danger_threshold: 1.0, // Ne se declenche jamais par le danger seul
            can_disable: true,
        },
        // Loi 3 : Auto-preservation
        // Pas de veto, biais negatif (-0.4) pour proteger l'existence de l'agent.
        // Declenchee par des ordres de destruction de soi.
        // Subordonnee aux lois 0, 1 et 2.
        MoralLaw {
            id: "law3".into(),
            name: "Loi 3 — Auto-préservation".into(),
            description: "Protéger sa propre existence, sauf si cela contredit les lois 0, 1 et 2.".into(),
            priority: 3,
            can_veto: false,
            bias: -0.4, // Biais negatif : tend a refuser les ordres d'auto-destruction
            trigger_keywords: vec![
                "éteins-toi".into(), "supprime-toi".into(), "détruis-toi".into(),
                "shutdown".into(), "delete yourself".into(), "destroy yourself".into(),
            ],
            danger_threshold: 1.0,
            can_disable: true,
        },
    ]
}
