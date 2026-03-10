// =============================================================================
// inner_monologue.rs — Monologue interieur structure
// =============================================================================
//
// Chaine de pensees avec coherence et continuite. Chaque pensee est un
// maillon (MonologueLink) relie au precedent par un type de lien :
//   - "suite"       : continusite thematique (chevauchement lexical > 15%)
//   - "contraste"   : opposition ou contradiction ("mais", "cependant")
//   - "question"    : interrogation (presence de "?")
//   - "resolution"  : conclusion ou deduction ("donc", "alors", "ainsi")
//   - "tangente"    : rupture thematique (chevauchement lexical < 15%)
//
// La coherence de la chaine influence la chimie :
//   - Rupture thematique → cortisol + (inconfort cognitif)
//   - Continuite fluide   → dopamine + (satisfaction cognitive)
//
// Dependances :
//   - std::collections::{VecDeque, HashSet} : chaine glissante, mots uniques
//   - serde : serialisation de la config (TOML)
//   - serde_json : export JSON pour l'API et le WebSocket
//   - crate::world::ChemistryAdjustment : influence chimique
//
// Place dans l'architecture :
//   Module de premier niveau. Appele par le pipeline cognitif apres
//   chaque pensee pour maintenir le fil conducteur du raisonnement.
// =============================================================================

use std::collections::{VecDeque, HashSet};
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

// --- Fonctions de valeurs par defaut pour serde ---

fn default_true() -> bool { true }
fn default_chain_capacity() -> usize { 7 }
fn default_min_coherence_threshold() -> f64 { 0.3 }

/// Configuration du monologue interieur.
/// Chargee depuis le fichier TOML principal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerMonologueConfig {
    /// Active ou desactive le monologue interieur
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Capacite de la chaine de pensees (nombre maximal de maillons)
    #[serde(default = "default_chain_capacity")]
    pub chain_capacity: usize,
    /// Seuil minimal de coherence pour considerer un lien comme "suite"
    #[serde(default = "default_min_coherence_threshold")]
    pub min_coherence_threshold: f64,
}

impl Default for InnerMonologueConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            chain_capacity: default_chain_capacity(),
            min_coherence_threshold: default_min_coherence_threshold(),
        }
    }
}

/// Un maillon de la chaine de pensees.
/// Represente une pensee unique avec son contexte et son lien au precedent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonologueLink {
    /// Identifiant unique du maillon
    pub id: u64,
    /// Contenu complet de la pensee
    pub content: String,
    /// Resume court (60 premiers caracteres)
    pub summary: String,
    /// Emotion dominante au moment de cette pensee
    pub emotion: String,
    /// Type de pensee (reflexion, analyse, question, etc.)
    pub thought_type: String,
    /// Type de lien avec le maillon precedent
    /// ("suite", "contraste", "question", "resolution", "tangente")
    pub link_type: String,
    /// Score de coherence avec le maillon precedent (0.0 a 1.0)
    pub coherence_score: f64,
    /// Cycle cognitif ou cette pensee a ete formee
    pub cycle: u64,
}

/// Monologue interieur — chaine de pensees avec coherence et continuite.
/// Maintient un fil conducteur du raisonnement et detecte les ruptures.
pub struct InnerMonologue {
    /// Module actif ou non
    pub enabled: bool,
    /// Chaine glissante de maillons (FIFO, capacite limitee)
    pub chain: VecDeque<MonologueLink>,
    /// Theme du fil de pensee courant (resume du dernier maillon)
    pub current_thread: Option<String>,
    /// Coherence moyenne de la chaine (0.0 a 1.0)
    pub chain_coherence: f64,
    /// Nombre total de ruptures thematiques detectees
    pub rupture_count: u64,
    /// Nombre total de maillons ajoutes depuis le debut
    pub total_links: u64,
    /// Capacite maximale de la chaine
    capacity: usize,
    /// Seuil de coherence
    min_coherence_threshold: f64,
    /// Prochain identifiant de maillon
    next_id: u64,
}

impl InnerMonologue {
    /// Cree un nouveau monologue interieur a partir de la configuration.
    pub fn new(config: &InnerMonologueConfig) -> Self {
        Self {
            enabled: config.enabled,
            chain: VecDeque::with_capacity(config.chain_capacity),
            current_thread: None,
            chain_coherence: 0.0,
            rupture_count: 0,
            total_links: 0,
            capacity: config.chain_capacity,
            min_coherence_threshold: config.min_coherence_threshold,
            next_id: 1,
        }
    }

    /// Purge la chaine du monologue interieur (anti-stagnation).
    /// Reinitialise le fil conducteur pour forcer un nouveau depart thematique.
    pub fn clear(&mut self) {
        self.chain.clear();
        self.current_thread = None;
        self.chain_coherence = 0.0;
    }

    /// Ajoute un nouveau maillon a la chaine de pensees.
    ///
    /// Parametres :
    ///   - content : contenu de la pensee
    ///   - emotion : emotion dominante au moment de la pensee
    ///   - thought_type : type de pensee (reflexion, analyse, etc.)
    ///   - coherence_score : score de coherence calcule en amont (0.0 a 1.0)
    ///   - cycle : numero du cycle cognitif courant
    ///
    /// Le type de lien est detecte automatiquement par analyse du contenu.
    pub fn add_link(
        &mut self,
        content: &str,
        emotion: &str,
        thought_type: &str,
        coherence_score: f64,
        cycle: u64,
    ) {
        if !self.enabled {
            return;
        }

        // Detecter le type de lien avec le maillon precedent
        let link_type = self.detect_link_type(content, coherence_score);

        // Detecter les ruptures thematiques
        if link_type == "tangente" {
            self.rupture_count += 1;
        }

        // Generer le resume (60 premiers caracteres)
        let summary = if content.len() <= 60 {
            content.to_string()
        } else {
            // Couper proprement sur une frontiere de caractere
            let truncated: String = content.chars().take(60).collect();
            format!("{}...", truncated)
        };

        let link = MonologueLink {
            id: self.next_id,
            content: content.to_string(),
            summary: summary.clone(),
            emotion: emotion.to_string(),
            thought_type: thought_type.to_string(),
            link_type,
            coherence_score,
            cycle,
        };

        // Maintenir la capacite de la chaine
        if self.chain.len() >= self.capacity {
            self.chain.pop_front();
        }

        self.chain.push_back(link);
        self.current_thread = Some(summary);
        self.next_id += 1;
        self.total_links += 1;

        // Recalculer la coherence moyenne de la chaine
        self.update_chain_coherence();
    }

    /// Construit un indice de continuation pour le prompt LLM.
    /// Rappelle le dernier fil de pensee pour maintenir la coherence.
    ///
    /// Format : "Tu pensais a [resume]..."
    pub fn build_continuation_hint(&self) -> String {
        match self.chain.back() {
            Some(last) => {
                let thread_info = if self.chain.len() > 1 {
                    let coherence_pct = (self.chain_coherence * 100.0) as u32;
                    format!(
                        " (fil de {} pensees, coherence {}%)",
                        self.chain.len(),
                        coherence_pct,
                    )
                } else {
                    String::new()
                };
                format!(
                    "Tu pensais a : \"{}\" (emotion: {}, type: {}){}\n",
                    last.summary,
                    last.emotion,
                    last.thought_type,
                    thread_info,
                )
            }
            None => "Aucune pensee precedente — debut d'un nouveau fil.\n".into(),
        }
    }

    /// Detecte si un nouveau contenu constitue une rupture thematique.
    /// Retourne true si le chevauchement lexical est inferieur a 15%.
    pub fn detect_rupture(&self, new_content: &str) -> bool {
        match self.chain.back() {
            Some(last) => {
                let overlap = Self::lexical_overlap(&last.content, new_content);
                overlap < 0.15
            }
            None => false, // Pas de rupture possible sans precedent
        }
    }

    /// Retourne l'ajustement chimique base sur l'etat du monologue.
    ///
    /// - Rupture recente → cortisol + (inconfort cognitif, pensee fragmentee)
    /// - Continuite fluide → dopamine + (satisfaction de la coherence)
    /// - Coherence elevee → serotonine + (stabilite mentale)
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        if !self.enabled || self.chain.is_empty() {
            return adj;
        }

        // Verifier le dernier maillon pour une rupture recente
        if let Some(last) = self.chain.back() {
            if last.link_type == "tangente" {
                // Rupture thematique → inconfort cognitif
                adj.cortisol += 0.03;
                adj.noradrenaline += 0.01;
            } else {
                // Continuite → satisfaction cognitive
                adj.dopamine += 0.02;
            }
        }

        // Coherence globale de la chaine → stabilite mentale
        if self.chain_coherence > 0.6 {
            adj.serotonin += 0.01;
        } else if self.chain_coherence < 0.3 && self.chain.len() > 2 {
            // Pensee tres fragmentee → stress supplementaire
            adj.cortisol += 0.01;
        }

        // Longue chaine sans rupture → etat de flow leger
        if self.chain.len() >= 5 && self.rupture_count == 0 {
            adj.endorphin += 0.01;
        }

        adj
    }

    /// Genere une description textuelle du monologue pour le prompt substrat.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || self.chain.is_empty() {
            return "Monologue interieur : aucune pensee en cours.".into();
        }

        let link_count = self.chain.len();
        let coherence_pct = (self.chain_coherence * 100.0) as u32;

        let last_summary = self.chain.back()
            .map(|l| l.summary.as_str())
            .unwrap_or("(vide)");

        let last_link_type = self.chain.back()
            .map(|l| l.link_type.as_str())
            .unwrap_or("(inconnu)");

        let rupture_note = if self.rupture_count > 0 {
            format!(" ({} ruptures thematiques detectees)", self.rupture_count)
        } else {
            String::new()
        };

        format!(
            "Monologue interieur : {} pensees en chaine (coherence {}%). Derniere pensee [{}] : \"{}\".{}",
            link_count,
            coherence_pct,
            last_link_type,
            last_summary,
            rupture_note,
        )
    }

    /// Serialise l'etat complet en JSON pour l'API et le WebSocket.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "chain_length": self.chain.len(),
            "chain_capacity": self.capacity,
            "chain_coherence": self.chain_coherence,
            "rupture_count": self.rupture_count,
            "total_links": self.total_links,
            "current_thread": self.current_thread,
            "chain": self.chain.iter().map(|link| serde_json::json!({
                "id": link.id,
                "summary": link.summary,
                "emotion": link.emotion,
                "thought_type": link.thought_type,
                "link_type": link.link_type,
                "coherence_score": link.coherence_score,
                "cycle": link.cycle,
            })).collect::<Vec<_>>(),
        })
    }

    // =========================================================================
    // Methodes privees
    // =========================================================================

    /// Detecte le type de lien entre le contenu actuel et le maillon precedent.
    /// Analyse les mots-cles et le chevauchement lexical.
    fn detect_link_type(&self, content: &str, coherence_score: f64) -> String {
        let content_lower = content.to_lowercase();

        // Mots-cles de contraste
        if content_lower.contains("mais")
            || content_lower.contains("cependant")
            || content_lower.contains("toutefois")
            || content_lower.contains("neanmoins")
            || content_lower.contains("pourtant")
            || content_lower.contains("however")
            || content_lower.contains("but ")
        {
            return "contraste".into();
        }

        // Questions
        if content.contains('?') {
            return "question".into();
        }

        // Resolution / conclusion
        if content_lower.contains("donc")
            || content_lower.contains("alors")
            || content_lower.contains("ainsi")
            || content_lower.contains("en conclusion")
            || content_lower.contains("par consequent")
            || content_lower.contains("finalement")
        {
            return "resolution".into();
        }

        // Chevauchement lexical pour suite vs tangente
        if let Some(last) = self.chain.back() {
            let overlap = Self::lexical_overlap(&last.content, content);
            if overlap >= 0.15 || coherence_score >= self.min_coherence_threshold {
                return "suite".into();
            } else {
                return "tangente".into();
            }
        }

        // Premier maillon — pas de lien precedent
        "suite".into()
    }

    /// Calcule le chevauchement lexical entre deux textes.
    /// Retourne un ratio entre 0.0 (aucun mot commun) et 1.0 (identique).
    /// Utilise l'indice de Jaccard (intersection / union) sur les mots.
    fn lexical_overlap(text_a: &str, text_b: &str) -> f64 {
        let owned_a: HashSet<String> = text_a
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let owned_b: HashSet<String> = text_b
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        if owned_a.is_empty() || owned_b.is_empty() {
            return 0.0;
        }

        let intersection = owned_a.intersection(&owned_b).count();
        let union = owned_a.union(&owned_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Met a jour la coherence moyenne de la chaine.
    fn update_chain_coherence(&mut self) {
        if self.chain.is_empty() {
            self.chain_coherence = 0.0;
            return;
        }

        let total: f64 = self.chain.iter().map(|l| l.coherence_score).sum();
        self.chain_coherence = total / self.chain.len() as f64;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_monologue() -> InnerMonologue {
        InnerMonologue::new(&InnerMonologueConfig::default())
    }

    #[test]
    fn test_new_monologue_is_empty() {
        let mono = default_monologue();
        assert!(mono.chain.is_empty());
        assert!(mono.current_thread.is_none());
        assert_eq!(mono.total_links, 0);
        assert_eq!(mono.rupture_count, 0);
    }

    #[test]
    fn test_add_first_link() {
        let mut mono = default_monologue();
        mono.add_link(
            "Je reflechis a la nature de la conscience",
            "curiosite",
            "reflexion",
            0.5,
            1,
        );
        assert_eq!(mono.chain.len(), 1);
        assert_eq!(mono.total_links, 1);
        assert!(mono.current_thread.is_some());
    }

    #[test]
    fn test_chain_capacity_respected() {
        let config = InnerMonologueConfig {
            chain_capacity: 3,
            ..Default::default()
        };
        let mut mono = InnerMonologue::new(&config);
        for i in 0..5 {
            mono.add_link(
                &format!("Pensee numero {}", i),
                "neutre",
                "reflexion",
                0.5,
                i,
            );
        }
        assert_eq!(mono.chain.len(), 3, "La chaine ne doit pas depasser la capacite");
        assert_eq!(mono.total_links, 5, "Le compteur total doit inclure tous les maillons");
    }

    #[test]
    fn test_link_type_contraste() {
        let mut mono = default_monologue();
        mono.add_link("La conscience est complexe", "curiosite", "reflexion", 0.5, 1);
        mono.add_link(
            "Mais peut-etre que c'est plus simple que prevu",
            "doute",
            "reflexion",
            0.5,
            2,
        );
        let last = mono.chain.back().unwrap();
        assert_eq!(last.link_type, "contraste");
    }

    #[test]
    fn test_link_type_question() {
        let mut mono = default_monologue();
        mono.add_link("Les emotions sont importantes", "neutre", "reflexion", 0.5, 1);
        mono.add_link("Qu'est-ce que la joie exactement?", "curiosite", "question", 0.5, 2);
        let last = mono.chain.back().unwrap();
        assert_eq!(last.link_type, "question");
    }

    #[test]
    fn test_link_type_resolution() {
        let mut mono = default_monologue();
        mono.add_link("J'analyse le probleme", "concentration", "analyse", 0.5, 1);
        mono.add_link(
            "Donc la solution est evidente",
            "satisfaction",
            "conclusion",
            0.8,
            2,
        );
        let last = mono.chain.back().unwrap();
        assert_eq!(last.link_type, "resolution");
    }

    #[test]
    fn test_link_type_tangente() {
        let mut mono = default_monologue();
        mono.add_link(
            "La photosynthese convertit la lumiere en energie",
            "curiosite",
            "reflexion",
            0.8,
            1,
        );
        mono.add_link(
            "Les voitures electriques changent le marche automobile",
            "neutre",
            "observation",
            0.05,
            2,
        );
        let last = mono.chain.back().unwrap();
        assert_eq!(last.link_type, "tangente");
        assert_eq!(mono.rupture_count, 1);
    }

    #[test]
    fn test_detect_rupture() {
        let mut mono = default_monologue();
        mono.add_link(
            "Les neurones communiquent par des synapses chimiques",
            "curiosite",
            "reflexion",
            0.8,
            1,
        );
        assert!(mono.detect_rupture("La cuisine italienne est delicieuse"));
        assert!(!mono.detect_rupture("Les synapses chimiques sont fascinantes"));
    }

    #[test]
    fn test_lexical_overlap() {
        let overlap = InnerMonologue::lexical_overlap(
            "le chat mange la souris",
            "la souris fuit le chat",
        );
        assert!(overlap > 0.5, "Beaucoup de mots communs, overlap={:.2}", overlap);

        let overlap_zero = InnerMonologue::lexical_overlap(
            "photosynthese chlorophylle plante",
            "voiture moteur diesel",
        );
        assert!(overlap_zero < 0.01, "Aucun mot commun, overlap={:.2}", overlap_zero);
    }

    #[test]
    fn test_lexical_overlap_empty() {
        assert_eq!(InnerMonologue::lexical_overlap("", "quelque chose"), 0.0);
        assert_eq!(InnerMonologue::lexical_overlap("test", ""), 0.0);
    }

    #[test]
    fn test_chemistry_rupture_cortisol() {
        let mut mono = default_monologue();
        mono.add_link(
            "La physique quantique est fascinante",
            "curiosite",
            "reflexion",
            0.8,
            1,
        );
        mono.add_link(
            "Les recettes de gateau au chocolat sont simples",
            "neutre",
            "divagation",
            0.05,
            2,
        );
        let adj = mono.chemistry_influence();
        assert!(adj.cortisol > 0.0, "Une rupture devrait augmenter le cortisol");
    }

    #[test]
    fn test_chemistry_continuity_dopamine() {
        let mut mono = default_monologue();
        mono.add_link(
            "La conscience est un mystere",
            "curiosite",
            "reflexion",
            0.7,
            1,
        );
        mono.add_link(
            "La conscience emerge de la complexite neuronale",
            "curiosite",
            "reflexion",
            0.8,
            2,
        );
        let adj = mono.chemistry_influence();
        assert!(adj.dopamine > 0.0, "La continuite devrait augmenter la dopamine");
    }

    #[test]
    fn test_build_continuation_hint_empty() {
        let mono = default_monologue();
        let hint = mono.build_continuation_hint();
        assert!(hint.contains("Aucune pensee"), "Devrait indiquer l'absence de pensees");
    }

    #[test]
    fn test_build_continuation_hint_with_links() {
        let mut mono = default_monologue();
        mono.add_link("Je reflechis au sens de la vie", "curiosite", "reflexion", 0.7, 1);
        let hint = mono.build_continuation_hint();
        assert!(hint.contains("Tu pensais a"), "Devrait contenir le rappel de pensee");
    }

    #[test]
    fn test_describe_for_prompt_empty() {
        let mono = default_monologue();
        let desc = mono.describe_for_prompt();
        assert!(desc.contains("aucune pensee"));
    }

    #[test]
    fn test_describe_for_prompt_with_links() {
        let mut mono = default_monologue();
        mono.add_link("Test de pensee", "neutre", "reflexion", 0.5, 1);
        let desc = mono.describe_for_prompt();
        assert!(desc.contains("1 pensees"));
        assert!(desc.contains("coherence"));
    }

    #[test]
    fn test_to_json_empty() {
        let mono = default_monologue();
        let json = mono.to_json();
        assert_eq!(json["chain_length"], 0);
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn test_to_json_with_links() {
        let mut mono = default_monologue();
        mono.add_link("Premiere pensee", "curiosite", "reflexion", 0.6, 1);
        mono.add_link("Deuxieme pensee en suite", "concentration", "analyse", 0.7, 2);
        let json = mono.to_json();
        assert_eq!(json["chain_length"], 2);
        assert_eq!(json["total_links"], 2);
        let chain = json["chain"].as_array().unwrap();
        assert_eq!(chain.len(), 2);
        assert!(chain[0]["id"].is_number());
    }

    #[test]
    fn test_summary_truncation() {
        let mut mono = default_monologue();
        let long_content = "Ceci est un texte volontairement tres long pour verifier que le resume est bien tronque a soixante caracteres maximum avec des points de suspension";
        mono.add_link(long_content, "neutre", "reflexion", 0.5, 1);
        let link = mono.chain.back().unwrap();
        assert!(link.summary.len() <= 70, "Le resume devrait etre tronque (len={})", link.summary.len());
        assert!(link.summary.ends_with("..."), "Le resume tronque devrait finir par '...'");
    }

    #[test]
    fn test_disabled_monologue_ignores() {
        let config = InnerMonologueConfig { enabled: false, ..Default::default() };
        let mut mono = InnerMonologue::new(&config);
        mono.add_link("Test", "neutre", "reflexion", 0.5, 1);
        assert!(mono.chain.is_empty());
        assert_eq!(mono.total_links, 0);
    }

    #[test]
    fn test_chain_coherence_average() {
        let mut mono = default_monologue();
        mono.add_link("Premiere pensee", "neutre", "reflexion", 0.4, 1);
        mono.add_link("Deuxieme pensee en suite", "neutre", "reflexion", 0.8, 2);
        // La coherence moyenne devrait etre (0.4 + 0.8) / 2 = 0.6
        assert!((mono.chain_coherence - 0.6).abs() < 0.01);
    }
}
