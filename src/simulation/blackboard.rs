// =============================================================================
// blackboard.rs — Architecture Blackboard pour la coordination des algorithmes
//
// Role : Tableau central ou chaque algorithme (BT, FSM, InfluenceMap, Steering,
//        Utility AI) ecrit ses recommandations. Le Blackboard resout les conflits
//        par priorite et genere un resume compact pour le prompt LLM.
//
// Place dans l'architecture :
//   Remplace les injections separees de chaque algorithme par un point unique
//   de coordination. Chaque algo ecrit dans le blackboard, et le prompt lit
//   une synthese coherente.
// =============================================================================

use std::collections::HashMap;

/// Entree dans le blackboard.
#[derive(Debug, Clone)]
pub struct BlackboardEntry {
    /// Valeur textuelle de la recommandation
    pub value: String,
    /// Source de la recommandation (nom de l'algorithme)
    pub source: &'static str,
    /// Priorite (plus haut = prioritaire)
    pub priority: u8,
    /// Cycle ou cette entree a ete ecrite
    pub cycle: u64,
}

/// Tableau central de coordination inter-algorithmes.
#[derive(Debug, Clone)]
pub struct Blackboard {
    /// Slots : cle → liste d'entrees (ordonnees par priorite)
    entries: HashMap<String, Vec<BlackboardEntry>>,
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Blackboard {
    /// Cree un blackboard vide.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Ecrit une entree dans un slot du blackboard.
    /// Si le slot existe deja, ajoute l'entree a la liste.
    pub fn write(&mut self, slot: &str, value: String, source: &'static str, priority: u8, cycle: u64) {
        let entry = BlackboardEntry { value, source, priority, cycle };
        self.entries.entry(slot.to_string()).or_default().push(entry);
    }

    /// Lit la meilleure entree d'un slot (plus haute priorite).
    pub fn read_best(&self, slot: &str) -> Option<&BlackboardEntry> {
        self.entries.get(slot)
            .and_then(|entries| entries.iter().max_by_key(|e| e.priority))
    }

    /// Lit toutes les entrees d'un slot.
    pub fn read_all(&self, slot: &str) -> Vec<&BlackboardEntry> {
        self.entries.get(slot)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }

    /// Supprime les entrees obsoletes (plus vieilles que `max_age` cycles).
    pub fn clear_stale(&mut self, current_cycle: u64, max_age: u64) {
        for entries in self.entries.values_mut() {
            entries.retain(|e| current_cycle - e.cycle <= max_age);
        }
        // Supprimer les slots vides
        self.entries.retain(|_, v| !v.is_empty());
    }

    /// Genere un resume compact pour le prompt LLM.
    /// Prend la meilleure entree par slot, max 3 lignes.
    pub fn describe_for_prompt(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (slot, entries) in &self.entries {
            if let Some(best) = entries.iter().max_by_key(|e| e.priority) {
                lines.push(format!("{}: {} ({})", slot, best.value, best.source));
            }
        }
        lines.sort();
        lines.truncate(3);
        if lines.is_empty() {
            String::new()
        } else {
            lines.join(" | ")
        }
    }

    /// Reinitialise le blackboard.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Nombre de slots actifs.
    pub fn slot_count(&self) -> usize {
        self.entries.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let mut bb = Blackboard::new();
        bb.write("mode", "explore".into(), "BT", 150, 10);
        bb.write("mode", "focus".into(), "FSM", 120, 10);
        let best = bb.read_best("mode").unwrap();
        assert_eq!(best.value, "explore");
        assert_eq!(best.source, "BT");
        assert_eq!(best.priority, 150);
    }

    #[test]
    fn test_clear_stale() {
        let mut bb = Blackboard::new();
        bb.write("old", "data".into(), "test", 100, 5);
        bb.write("new", "data".into(), "test", 100, 15);
        bb.clear_stale(20, 10);
        assert!(bb.read_best("old").is_none());
        assert!(bb.read_best("new").is_some());
    }

    #[test]
    fn test_describe_for_prompt() {
        let mut bb = Blackboard::new();
        bb.write("mode", "explore".into(), "BT", 150, 10);
        bb.write("focus", "philosophie".into(), "IM", 100, 10);
        let desc = bb.describe_for_prompt();
        assert!(!desc.is_empty());
        assert!(desc.contains("explore"));
    }
}
