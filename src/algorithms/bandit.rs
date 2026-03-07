// =============================================================================
// bandit.rs — UCB1 Multi-Armed Bandit avec exploration epsilon-greedy
// =============================================================================
//
// Rôle : Implémente le bandit multi-bras (MAB = Multi-Armed Bandit) avec la
//        stratégie UCB1 (Upper Confidence Bound 1) combinée à une exploration
//        epsilon-greedy. Ce mécanisme équilibre l'exploration de nouvelles
//        options et l'exploitation des options connues comme performantes.
//
// Dépendances :
//   - serde : sérialisation/désérialisation pour la persistance en base de données
//   - std::time, std::cell : pour le générateur pseudo-aléatoire local
//
// Place dans l'architecture :
//   Utilisé par le système cognitif de Saphire pour sélectionner les types
//   de pensées autonomes (curiosité, introspection, créativité...) de manière
//   optimale. Chaque type de pensée est un « bras » du bandit, et la
//   satisfaction résultante est la récompense.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Un bras du bandit — représente une option (un type de pensée, une action)
/// avec ses statistiques cumulées de sélection et de récompense.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditArm {
    /// Nom descriptif du bras (ex: "curiosité", "introspection")
    pub name: String,
    /// Nombre de fois que ce bras a été sélectionné (tiré)
    pub pulls: u64,
    /// Somme cumulative des récompenses obtenues en sélectionnant ce bras
    pub total_reward: f64,
}

/// UCB1 (Upper Confidence Bound 1) Multi-Armed Bandit avec exploration
/// epsilon-greedy.
///
/// UCB1 sélectionne le bras maximisant : moyenne + sqrt(2 * ln(T) / n_i)
/// où T = nombre total de tirages et n_i = nombre de tirages du bras i.
/// Le terme d'exploration (sqrt) favorise les bras peu explorés.
///
/// L'epsilon-greedy ajoute une probabilité epsilon de sélection purement
/// aléatoire, garantissant une exploration minimale continue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UCB1Bandit {
    /// Liste de tous les bras du bandit
    pub arms: Vec<BanditArm>,
    /// Nombre total de tirages effectués sur tous les bras
    pub total_pulls: u64,
    /// Probabilité d'exploration aléatoire pure (epsilon-greedy)
    /// Valeur par défaut : 0.25 (25% de chance de choisir au hasard)
    pub epsilon: f64,
    /// Bonus d'exploration dynamique, module par la dissonance cognitive.
    /// Ajoute au C=2.0 de base dans la formule UCB1.
    /// Valeur par defaut : 0.0 (pas de bonus). Plage typique : [0.0, 1.5].
    #[serde(default)]
    pub exploration_bonus: f64,
}

impl UCB1Bandit {
    /// Crée un bandit avec les noms des bras fournis.
    /// Chaque bras est initialisé avec 0 tirages et 0 récompense.
    ///
    /// Paramètre `arm_names` : noms des options/bras disponibles
    /// Retourne : une instance de UCB1Bandit prête à l'utilisation
    pub fn new(arm_names: &[&str]) -> Self {
        Self {
            arms: arm_names.iter().map(|name| BanditArm {
                name: name.to_string(),
                pulls: 0,
                total_reward: 0.0,
            }).collect(),
            total_pulls: 0,
            epsilon: 0.25, // 25% de chance d'exploration aléatoire
            exploration_bonus: 0.0,
        }
    }

    /// Sélectionne le bras optimal selon la stratégie UCB1 + epsilon-greedy.
    ///
    /// Fonctionnement :
    /// 1. Avec probabilité epsilon, choisir un bras au hasard (exploration)
    /// 2. Sinon, choisir le bras maximisant le score UCB1 :
    ///    score = moyenne_récompense + sqrt(2 * ln(total_tirages) / tirages_bras)
    ///    - Les bras jamais tirés (pulls = 0) reçoivent un score infini
    ///      pour forcer leur exploration initiale
    ///
    /// Retourne : l'indice du bras sélectionné
    pub fn select(&self) -> usize {
        // Epsilon-greedy : exploration aléatoire avec probabilité epsilon
        if rand_f64() < self.epsilon {
            return rand_usize(self.arms.len());
        }

        // Sélection UCB1 : choisir le bras avec le meilleur score
        self.arms.iter().enumerate()
            .map(|(i, arm)| {
                // Bras jamais tiré : score infini pour forcer l'exploration
                if arm.pulls == 0 {
                    return (i, f64::INFINITY);
                }
                // Récompense moyenne de ce bras
                let mean = arm.total_reward / arm.pulls as f64;
                // Terme d'exploration UCB1 : C adaptatif (base 2.0 + bonus dissonance)
                let c = 2.0 + self.exploration_bonus;
                let exploration = (c * (self.total_pulls as f64).ln() / arm.pulls as f64).sqrt();
                (i, mean + exploration)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Sélectionne un bras en excluant certains indices (mécanisme anti-répétition).
    ///
    /// Fonctionne comme select() mais filtre les bras dont l'indice est dans
    /// la liste d'exclusion. Si tous les bras sont exclus, on revient à la
    /// sélection normale (fallback).
    ///
    /// Paramètre `exclude` : indices des bras à exclure de la sélection
    /// Retourne : l'indice du bras sélectionné (hors exclusions si possible)
    pub fn select_excluding(&self, exclude: &[usize]) -> usize {
        // Epsilon-greedy avec exclusion
        if rand_f64() < self.epsilon {
            let candidates: Vec<usize> = (0..self.arms.len())
                .filter(|i| !exclude.contains(i))
                .collect();
            if !candidates.is_empty() {
                return candidates[rand_usize(candidates.len())];
            }
        }

        // UCB1 avec exclusion des bras non désirés
        self.arms.iter().enumerate()
            .filter(|(i, _)| !exclude.contains(i))
            .map(|(i, arm)| {
                if arm.pulls == 0 {
                    return (i, f64::INFINITY);
                }
                let mean = arm.total_reward / arm.pulls as f64;
                // Securite : eviter ln(0) si total_pulls == 0
                let total = if self.total_pulls == 0 { 1 } else { self.total_pulls };
                let c = 2.0 + self.exploration_bonus;
                let exploration = (c * (total as f64).ln() / arm.pulls as f64).sqrt();
                (i, mean + exploration)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or_else(|| self.select()) // Fallback : sélection normale si tout est exclu
    }

    /// Met à jour un bras après observation d'une récompense.
    ///
    /// Paramètre `arm_idx` : indice du bras qui a été sélectionné
    /// Paramètre `reward` : récompense obtenue (ex: satisfaction de l'utilisateur)
    pub fn update(&mut self, arm_idx: usize, reward: f64) {
        if arm_idx < self.arms.len() {
            self.arms[arm_idx].pulls += 1;
            self.arms[arm_idx].total_reward += reward;
            self.total_pulls += 1;
        }
    }

    /// Applique un facteur de decay sur le reward moyen d'un bras sur-explore
    /// qui produit du contenu de faible qualite.
    /// Exemple : factor = 0.95 → -5% du total_reward par pensee faible.
    pub fn apply_quality_decay(&mut self, arm_idx: usize, factor: f64) {
        if arm_idx < self.arms.len() {
            let arm = &mut self.arms[arm_idx];
            if arm.pulls > 10 {
                arm.total_reward *= factor;
            }
        }
    }

    /// Retourne le nom du bras sélectionné par la stratégie UCB1.
    ///
    /// Retourne : référence vers le nom du bras choisi
    pub fn select_name(&self) -> &str {
        let idx = self.select();
        &self.arms[idx].name
    }

    /// Charge les statistiques des bras depuis la base de données.
    ///
    /// Fusionne les données chargées avec les bras existants en cherchant
    /// par nom. Les bras non trouvés dans les données chargées conservent
    /// leurs valeurs actuelles.
    ///
    /// Paramètre `arms` : tuples (nom, tirages, récompense_totale) depuis la DB
    pub fn load_arms(&mut self, arms: &[(String, u64, f64)]) {
        for (name, pulls, total_reward) in arms {
            if let Some(arm) = self.arms.iter_mut().find(|a| a.name == *name) {
                arm.pulls = *pulls;
                arm.total_reward = *total_reward;
            }
        }
        // Recalculer le total des tirages à partir des bras mis à jour
        self.total_pulls = self.arms.iter().map(|a| a.pulls).sum();
    }

    /// Retourne les scores UCB1 bruts de chaque bras (sans epsilon-greedy).
    /// Utilise par le mode hybride Utility AI + UCB1.
    pub fn all_scores(&self) -> Vec<f64> {
        self.arms.iter().map(|arm| {
            if arm.pulls == 0 {
                return 10.0; // Score eleve pour bras non explores
            }
            let mean = arm.total_reward / arm.pulls as f64;
            let total = if self.total_pulls == 0 { 1 } else { self.total_pulls };
            let c = 2.0 + self.exploration_bonus;
            let exploration = (c * (total as f64).ln() / arm.pulls as f64).sqrt();
            mean + exploration
        }).collect()
    }

    /// Exporte les statistiques des bras pour sauvegarde en base de données.
    ///
    /// Retourne : vecteur de tuples (nom, tirages, récompense_totale)
    pub fn export_arms(&self) -> Vec<(String, u64, f64)> {
        self.arms.iter()
            .map(|a| (a.name.clone(), a.pulls, a.total_reward))
            .collect()
    }
}

/// Générateur pseudo-aléatoire simple de f64 dans [0, 1).
///
/// Utilise un état thread-local initialisé à partir de l'horloge système
/// et l'algorithme xorshift64 pour produire des nombres pseudo-aléatoires.
///
/// Pourquoi ne pas utiliser rand::thread_rng() : ce module évite la dépendance
/// à rand pour rester léger et avoir un contrôle total sur le PRNG.
///
/// Retourne : un f64 pseudo-aléatoire dans l'intervalle [0, 1)
pub(crate) fn rand_f64() -> f64 {
    use std::time::SystemTime;
    use std::cell::Cell;
    thread_local! {
        // Initialisation de la graine à partir du temps système (nanosecondes)
        static STATE: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        );
    }
    STATE.with(|s| {
        // Algorithme xorshift64 : générateur pseudo-aléatoire rapide et léger
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x as f64) / (u64::MAX as f64)
    })
}

/// Génère un entier pseudo-aléatoire dans l'intervalle [0, max).
///
/// Paramètre `max` : borne supérieure exclusive
/// Retourne : un usize pseudo-aléatoire dans [0, max)
fn rand_usize(max: usize) -> usize {
    if max == 0 { return 0; }
    (rand_f64() * max as f64) as usize % max
}
