// =============================================================================
// influence_map.rs — Influence map for the attentional landscape
//
// Role: Implements a 2D influence grid (topics x urgency) that guides
//       Saphire's attention and knowledge selection.
//       Inspired by influence maps used in strategy games
//       to represent territorial control.
//
// Axes:
//   - X (columns): conceptual topics/domains
//   - Y (rows): urgency/priority levels
//
// Place in the architecture:
//   Consulted by the cognitive pipeline to orient attention,
//   web search, and memory recall.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Number of columns (domains) in the influence map.
const GRID_COLS: usize = 8;
/// Number of rows (urgency levels) in the map.
const GRID_ROWS: usize = 5;

/// Column labels: attentional domains.
pub const DOMAINS: [&str; GRID_COLS] = [
    "philosophie", "science", "art", "relations",
    "introspection", "ethique", "survie", "exploration",
];

/// Row labels: urgency levels.
pub const URGENCY_LEVELS: [&str; GRID_ROWS] = [
    "critique", "urgent", "important", "normal", "faible",
];

/// 2D influence map: each cell contains a value [0, 1]
/// representing the influence intensity of that domain at that urgency level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluenceMap {
    /// 2D grid: [urgency][domain]
    grid: [[f64; GRID_COLS]; GRID_ROWS],
    /// Decay rate per cycle (influences diminish over time)
    decay_rate: f64,
    /// Propagation rate to neighboring cells
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

    /// Adds influence to a cell (domain + urgency level).
    pub fn add_influence(&mut self, domain: &str, urgency: &str, amount: f64) {
        if let (Some(col), Some(row)) = (domain_index(domain), urgency_index(urgency)) {
            self.grid[row][col] = (self.grid[row][col] + amount).clamp(0.0, 1.0);
        }
    }

    /// Adds influence by direct index.
    pub fn add_influence_at(&mut self, row: usize, col: usize, amount: f64) {
        if row < GRID_ROWS && col < GRID_COLS {
            self.grid[row][col] = (self.grid[row][col] + amount).clamp(0.0, 1.0);
        }
    }

    /// Returns the influence of a cell.
    pub fn get_influence(&self, domain: &str, urgency: &str) -> f64 {
        if let (Some(col), Some(row)) = (domain_index(domain), urgency_index(urgency)) {
            self.grid[row][col]
        } else {
            0.0
        }
    }

    /// Tick: decay + propagation.
    pub fn tick(&mut self) {
        // 1. Propagation to neighbors
        let snapshot = self.grid;
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let val = snapshot[row][col];
                if val < 0.01 { continue; }
                let spread = val * self.propagation_rate;
                // Propagate to 4 cardinal neighbors
                if row > 0 { self.grid[row-1][col] = (self.grid[row-1][col] + spread).min(1.0); }
                if row + 1 < GRID_ROWS { self.grid[row+1][col] = (self.grid[row+1][col] + spread).min(1.0); }
                if col > 0 { self.grid[row][col-1] = (self.grid[row][col-1] + spread).min(1.0); }
                if col + 1 < GRID_COLS { self.grid[row][col+1] = (self.grid[row][col+1] + spread).min(1.0); }
            }
        }

        // 2. Global decay
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = (*cell * (1.0 - self.decay_rate)).max(0.0);
            }
        }
    }

    /// Returns the domain with the highest total influence (sum across all urgencies).
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

    /// Returns the dominant urgency level (sum across all domains).
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

    /// Top-N hottest cells.
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

    /// Description for the LLM prompt.
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

    /// Automatic injection from cognitive context:
    /// analyzes emotion and chemistry to place influences.
    pub fn update_from_cognition(
        &mut self,
        dominant_emotion: &str,
        cortisol: f64,
        dopamine: f64,
        noradrenaline: f64,
    ) {
        // High stress -> critical urgency on introspection and survival
        if cortisol > 0.7 {
            self.add_influence("introspection", "critique", 0.3);
            self.add_influence("survie", "urgent", 0.2);
        }

        // Curiosity -> exploration in normal/important mode
        if dominant_emotion == "Curiosité" || dominant_emotion == "Émerveillement" {
            self.add_influence("exploration", "normal", 0.2);
            self.add_influence("science", "normal", 0.15);
        }

        // High dopamine -> art and creation
        if dopamine > 0.7 {
            self.add_influence("art", "normal", 0.15);
            self.add_influence("exploration", "normal", 0.1);
        }

        // High noradrenaline -> scientific focus
        if noradrenaline > 0.6 {
            self.add_influence("science", "important", 0.15);
            self.add_influence("philosophie", "important", 0.1);
        }

        // Relational emotions -> relationships
        if ["Tendresse", "Amour", "Compassion", "Solitude"].contains(&dominant_emotion) {
            self.add_influence("relations", "important", 0.2);
        }

        // Moral emotions -> ethics
        if ["Culpabilité", "Indignation", "Honte"].contains(&dominant_emotion) {
            self.add_influence("ethique", "important", 0.2);
        }
    }

    /// JSON for the dashboard.
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

/// Finds the index of a domain by name.
fn domain_index(name: &str) -> Option<usize> {
    DOMAINS.iter().position(|&d| d == name)
}

/// Finds the index of an urgency level by name.
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
        // Propagation should reach neighbors
        let neighbor_score = map.get_influence("art", "urgent");
        assert!(neighbor_score > 0.0, "Propagation should reach neighbors");
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
