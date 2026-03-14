// =============================================================================
// influence_map.rs — Carte d'influence pour le paysage attentionnel
//
// Role : Implemente une grille 2D d'influence (topics x urgence) qui guide
//        l'attention et la selection de connaissances de Saphire.
//        Inspiree des influence maps utilisees dans les jeux de strategie
//        pour representer le controle territorial.
//
// Axes :
//   - X (colonnes) : topics/domaines conceptuels
//   - Y (lignes) : niveaux d'urgence/priorite
//
// Place dans l'architecture :
//   Consultee par le pipeline cognitif pour orienter l'attention,
//   la recherche web, et la selection de souvenirs.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Nombre de colonnes (domaines) dans la carte d'influence.
const GRID_COLS: usize = 8;
/// Nombre de lignes (niveaux d'urgence) dans la carte.
const GRID_ROWS: usize = 5;

/// Labels des colonnes : domaines attentionnels.
pub const DOMAINS: [&str; GRID_COLS] = [
    "philosophie", "science", "art", "relations",
    "introspection", "ethique", "survie", "exploration",
];

/// Labels des lignes : niveaux d'urgence.
pub const URGENCY_LEVELS: [&str; GRID_ROWS] = [
    "critique", "urgent", "important", "normal", "faible",
];

/// Carte d'influence 2D : chaque cellule contient une valeur [0, 1]
/// representant l'intensite d'influence de ce domaine a ce niveau d'urgence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluenceMap {
    /// Grille 2D : [urgence][domaine]
    grid: [[f64; GRID_COLS]; GRID_ROWS],
    /// Taux de decay par cycle (les influences diminuent avec le temps)
    decay_rate: f64,
    /// Taux de propagation aux cellules voisines
    propagation_rate: f64,
}

impl InfluenceMap {
    pub fn new(decay_rate: f64, propagation_rate: f64) -> Self {
        Self {
            grid: [[0.0; GRID_COLS]; GRID_ROWS],
            decay_rate: decay_rate.clamp(0.01, 0.5),
            propagation_rate: propagation_rate.clamp(0.0, 0.3),
        }
    }

    /// Ajoute de l'influence a une cellule (domaine + niveau d'urgence).
    pub fn add_influence(&mut self, domain: &str, urgency: &str, amount: f64) {
        if let (Some(col), Some(row)) = (domain_index(domain), urgency_index(urgency)) {
            self.grid[row][col] = (self.grid[row][col] + amount).clamp(0.0, 1.0);
        }
    }

    /// Ajoute de l'influence par index direct.
    pub fn add_influence_at(&mut self, row: usize, col: usize, amount: f64) {
        if row < GRID_ROWS && col < GRID_COLS {
            self.grid[row][col] = (self.grid[row][col] + amount).clamp(0.0, 1.0);
        }
    }

    /// Retourne l'influence d'une cellule.
    pub fn get_influence(&self, domain: &str, urgency: &str) -> f64 {
        if let (Some(col), Some(row)) = (domain_index(domain), urgency_index(urgency)) {
            self.grid[row][col]
        } else {
            0.0
        }
    }

    /// Tick : decay + propagation.
    pub fn tick(&mut self) {
        // 1. Propagation aux voisins
        let snapshot = self.grid;
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let val = snapshot[row][col];
                if val < 0.01 { continue; }
                let spread = val * self.propagation_rate;
                // Propager aux 4 voisins cardinaux
                if row > 0 { self.grid[row-1][col] = (self.grid[row-1][col] + spread).min(1.0); }
                if row + 1 < GRID_ROWS { self.grid[row+1][col] = (self.grid[row+1][col] + spread).min(1.0); }
                if col > 0 { self.grid[row][col-1] = (self.grid[row][col-1] + spread).min(1.0); }
                if col + 1 < GRID_COLS { self.grid[row][col+1] = (self.grid[row][col+1] + spread).min(1.0); }
            }
        }

        // 2. Decay global
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = (*cell * (1.0 - self.decay_rate)).max(0.0);
            }
        }
    }

    /// Retourne le domaine avec la plus haute influence totale (somme sur toutes les urgences).
    pub fn hottest_domain(&self) -> (&str, f64) {
        let mut best_col = 0;
        let mut best_score = 0.0;
        for col in 0..GRID_COLS {
            let score: f64 = (0..GRID_ROWS).map(|row| self.grid[row][col]).sum();
            if score > best_score {
                best_score = score;
                best_col = col;
            }
        }
        (DOMAINS[best_col], best_score)
    }

    /// Retourne le niveau d'urgence dominant (somme sur tous les domaines).
    pub fn dominant_urgency(&self) -> (&str, f64) {
        let mut best_row = 0;
        let mut best_score = 0.0;
        for row in 0..GRID_ROWS {
            let score: f64 = self.grid[row].iter().sum();
            if score > best_score {
                best_score = score;
                best_row = row;
            }
        }
        (URGENCY_LEVELS[best_row], best_score)
    }

    /// Top-N des cellules les plus chaudes.
    pub fn top_influences(&self, n: usize) -> Vec<(String, String, f64)> {
        let mut cells: Vec<(String, String, f64)> = Vec::new();
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                if self.grid[row][col] > 0.01 {
                    cells.push((
                        DOMAINS[col].to_string(),
                        URGENCY_LEVELS[row].to_string(),
                        self.grid[row][col],
                    ));
                }
            }
        }
        cells.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        cells.truncate(n);
        cells
    }

    /// Description pour le prompt LLM.
    pub fn describe_for_prompt(&self) -> String {
        let top = self.top_influences(3);
        if top.is_empty() {
            return String::new();
        }
        let mut desc = "PAYSAGE ATTENTIONNEL :\n".to_string();
        for (domain, urgency, score) in &top {
            desc.push_str(&format!("  - {} (urgence: {}, intensite: {:.0}%)\n",
                domain, urgency, score * 100.0));
        }
        desc
    }

    /// Injection automatique depuis le contexte cognitif :
    /// analyse l'emotion et la chimie pour placer des influences.
    pub fn update_from_cognition(
        &mut self,
        dominant_emotion: &str,
        cortisol: f64,
        dopamine: f64,
        noradrenaline: f64,
    ) {
        // Stress eleve → urgence critique sur introspection et survie
        if cortisol > 0.7 {
            self.add_influence("introspection", "critique", 0.3);
            self.add_influence("survie", "urgent", 0.2);
        }

        // Curiosite → exploration en mode normal/important
        if dominant_emotion == "Curiosité" || dominant_emotion == "Émerveillement" {
            self.add_influence("exploration", "normal", 0.2);
            self.add_influence("science", "normal", 0.15);
        }

        // Dopamine elevee → art et creation
        if dopamine > 0.7 {
            self.add_influence("art", "normal", 0.15);
            self.add_influence("exploration", "normal", 0.1);
        }

        // Noradrenaline elevee → focus scientifique
        if noradrenaline > 0.6 {
            self.add_influence("science", "important", 0.15);
            self.add_influence("philosophie", "important", 0.1);
        }

        // Emotions relationnelles → relations
        if ["Tendresse", "Amour", "Compassion", "Solitude"].contains(&dominant_emotion) {
            self.add_influence("relations", "important", 0.2);
        }

        // Emotions morales → ethique
        if ["Culpabilité", "Indignation", "Honte"].contains(&dominant_emotion) {
            self.add_influence("ethique", "important", 0.2);
        }
    }

    /// JSON pour le dashboard.
    pub fn to_json(&self) -> serde_json::Value {
        let (hot_domain, hot_score) = self.hottest_domain();
        let (urgency, urgency_score) = self.dominant_urgency();
        serde_json::json!({
            "hottest_domain": hot_domain,
            "hottest_score": hot_score,
            "dominant_urgency": urgency,
            "urgency_score": urgency_score,
            "top_influences": self.top_influences(5).iter().map(|(d, u, s)| {
                serde_json::json!({"domain": d, "urgency": u, "score": s})
            }).collect::<Vec<_>>(),
        })
    }
}

impl Default for InfluenceMap {
    fn default() -> Self {
        Self::new(0.05, 0.1)
    }
}

/// Trouve l'index d'un domaine par nom.
fn domain_index(name: &str) -> Option<usize> {
    DOMAINS.iter().position(|&d| d == name)
}

/// Trouve l'index d'un niveau d'urgence par nom.
fn urgency_index(name: &str) -> Option<usize> {
    URGENCY_LEVELS.iter().position(|&u| u == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut map = InfluenceMap::default();
        map.add_influence("science", "urgent", 0.5);
        assert!((map.get_influence("science", "urgent") - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_decay() {
        let mut map = InfluenceMap::new(0.1, 0.0);
        map.add_influence("art", "normal", 1.0);
        map.tick();
        assert!(map.get_influence("art", "normal") < 1.0);
    }

    #[test]
    fn test_propagation() {
        let mut map = InfluenceMap::new(0.0, 0.2);
        map.add_influence("science", "urgent", 1.0);
        map.tick();
        // La propagation doit toucher les voisins
        let neighbor_score = map.get_influence("art", "urgent");
        assert!(neighbor_score > 0.0, "La propagation doit atteindre les voisins");
    }

    #[test]
    fn test_hottest_domain() {
        let mut map = InfluenceMap::default();
        map.add_influence("philosophie", "critique", 0.9);
        let (domain, _) = map.hottest_domain();
        assert_eq!(domain, "philosophie");
    }

    #[test]
    fn test_cognition_update() {
        let mut map = InfluenceMap::default();
        map.update_from_cognition("Curiosité", 0.3, 0.5, 0.5);
        assert!(map.get_influence("exploration", "normal") > 0.0);
    }
}
