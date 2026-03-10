// =============================================================================
// analogical_reasoning.rs — Raisonnement analogique
//
// Role : Detecte des analogies entre la situation courante et les souvenirs
// en memoire a long terme (LTM). Quand une situation ressemble structurellement
// a une experience passee, le module transfere l'insight : "la derniere fois
// que c'etait similaire, voici ce qui s'est passe / ce qui a marche".
//
// Mecanisme :
//   - Comparaison par chevauchement lexical (mots communs) entre le contexte
//     courant et les resumes de souvenirs LTM.
//   - Bonus de similarite si l'emotion courante correspond a celle du souvenir.
//   - Seuil configurable (defaut 0.65) pour filtrer les analogies faibles.
//   - Les analogies pertinentes boostent la dopamine (recompense cognitive).
//
// Place dans l'architecture :
//   Module autonome appele durant le pipeline cognitif, apres la recuperation
//   des souvenirs LTM et avant la generation LLM. Les analogies trouvees
//   enrichissent le prompt substrat avec des references experientielles.
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration du module de raisonnement analogique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogicalReasoningConfig {
    /// Active ou desactive le raisonnement analogique
    pub enabled: bool,
    /// Seuil minimal de similarite structurelle (0.0 a 1.0)
    pub similarity_threshold: f64,
    /// Nombre maximal d'analogies recentes conservees
    pub max_recent: usize,
}

impl Default for AnalogicalReasoningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold: 0.65,
            max_recent: 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Enregistrement memoire simplifie
// ---------------------------------------------------------------------------

/// Enregistrement memoire simplifie pour le raisonnement analogique.
/// Les vrais MemoryRecord sont dans crate::db, on utilise un type local
/// pour decoupler le module de la couche base de donnees.
#[derive(Debug, Clone)]
pub struct MemoryRecordRef {
    /// Resume textuel du souvenir
    pub text_summary: String,
    /// Emotion dominante associee au souvenir
    pub emotion: String,
    /// Score de similarite vectorielle (pre-calcule par pgvector)
    pub similarity: f64,
}

// ---------------------------------------------------------------------------
// Analogie
// ---------------------------------------------------------------------------

/// Une analogie detectee entre un souvenir et le contexte courant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analogy {
    /// Identifiant unique de l'analogie
    pub id: u64,
    /// Resume du souvenir source (memoire LTM)
    pub source_memory_summary: String,
    /// Contexte cible (situation courante)
    pub target_context: String,
    /// Score de similarite structurelle (0.0 a 1.0)
    pub structural_similarity: f64,
    /// Insight transfere depuis l'experience passee
    pub transferred_insight: String,
    /// Domaine de l'analogie : "resolution", "emotion", "comportement"
    pub domain: String,
    /// Cycle cognitif durant lequel l'analogie a ete formee
    pub cycle: u64,
    /// True si l'analogie a ete confirmee comme pertinente a posteriori
    pub confirmed: bool,
}

// ---------------------------------------------------------------------------
// Moteur de raisonnement analogique
// ---------------------------------------------------------------------------

/// Moteur de raisonnement analogique — detecte les paralleles entre
/// le present et les experiences passees pour transferer des insights.
pub struct AnalogicalReasoning {
    /// Module actif ou non
    pub enabled: bool,
    /// Buffer circulaire des analogies recentes
    pub recent_analogies: VecDeque<Analogy>,
    /// Taux de reussite historique des analogies (EMA)
    pub success_rate: f64,
    /// Seuil minimal de similarite pour former une analogie
    pub similarity_threshold: f64,
    /// Compteur total d'analogies formees depuis le demarrage
    pub total_analogies: u64,
    /// Taille maximale du buffer d'analogies recentes
    max_recent: usize,
    /// Prochain identifiant unique
    next_id: u64,
}

impl Default for AnalogicalReasoning {
    fn default() -> Self {
        Self::new(&AnalogicalReasoningConfig::default())
    }
}

impl AnalogicalReasoning {
    /// Cree un nouveau moteur de raisonnement analogique.
    pub fn new(config: &AnalogicalReasoningConfig) -> Self {
        Self {
            enabled: config.enabled,
            recent_analogies: VecDeque::with_capacity(config.max_recent),
            success_rate: 0.5,
            similarity_threshold: config.similarity_threshold,
            total_analogies: 0,
            max_recent: config.max_recent,
            next_id: 1,
        }
    }

    /// Forme des analogies entre le contexte courant et les souvenirs LTM.
    ///
    /// Pour chaque enregistrement memoire, calcule la similarite structurelle
    /// par chevauchement lexical (mots communs) et correspondance emotionnelle.
    /// Si le score depasse le seuil, une analogie est creee et ajoutee au buffer.
    ///
    /// Retourne le nombre de nouvelles analogies formees.
    pub fn form_analogies(
        &mut self,
        current_context: &str,
        ltm_records: &[MemoryRecordRef],
        current_emotion: &str,
        cycle: u64,
    ) -> usize {
        if !self.enabled || current_context.is_empty() {
            return 0;
        }

        let context_words = Self::extract_words(current_context);
        if context_words.is_empty() {
            return 0;
        }

        let mut count = 0;

        for record in ltm_records {
            let memory_words = Self::extract_words(&record.text_summary);
            if memory_words.is_empty() {
                continue;
            }

            // -- Similarite lexicale : proportion de mots communs --
            let common = context_words.iter()
                .filter(|w| memory_words.contains(w))
                .count();
            let union_size = context_words.len().max(memory_words.len());
            let lexical_sim = common as f64 / union_size as f64;

            // -- Bonus d'emotion : +0.15 si l'emotion correspond --
            let emotion_bonus = if !current_emotion.is_empty()
                && !record.emotion.is_empty()
                && current_emotion.to_lowercase() == record.emotion.to_lowercase()
            {
                0.15
            } else {
                0.0
            };

            // -- Score combine, plafonne a 1.0 --
            let structural_similarity = (lexical_sim + emotion_bonus).min(1.0);

            if structural_similarity < self.similarity_threshold {
                continue;
            }

            // -- Determiner le domaine de l'analogie --
            let domain = Self::detect_domain(current_context, &record.emotion);

            // -- Construire l'insight transfere --
            let source_short: String = record.text_summary.chars().take(80).collect();
            let context_short: String = current_context.chars().take(60).collect();
            let transferred_insight = format!(
                "Comme [{}], je pourrais [appliquer cette experience a : {}]",
                source_short, context_short
            );

            let analogy = Analogy {
                id: self.next_id,
                source_memory_summary: record.text_summary.clone(),
                target_context: current_context.to_string(),
                structural_similarity,
                transferred_insight,
                domain,
                cycle,
                confirmed: false,
            };

            // Ajouter au buffer circulaire
            if self.recent_analogies.len() >= self.max_recent {
                self.recent_analogies.pop_front();
            }
            self.recent_analogies.push_back(analogy);

            self.next_id += 1;
            self.total_analogies += 1;
            count += 1;
        }

        count
    }

    /// Description pour le prompt substrat LLM.
    /// Produit un texte lisible decrivant les analogies actives.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let recent: Vec<&Analogy> = self.recent_analogies.iter()
            .rev()
            .take(3)
            .collect();

        if recent.is_empty() {
            return "ANALOGIES : Aucune analogie active — situation inedite.".into();
        }

        let mut lines = vec!["ANALOGIES — Cette situation me rappelle :".to_string()];

        for (i, a) in recent.iter().enumerate() {
            let source_short: String = a.source_memory_summary.chars().take(60).collect();
            lines.push(format!(
                "  {}. [{}] sim={:.0}% — \"{}\"",
                i + 1,
                a.domain,
                a.structural_similarity * 100.0,
                source_short,
            ));
        }

        lines.push(format!(
            "  Taux de reussite historique : {:.0}%. Total : {}.",
            self.success_rate * 100.0,
            self.total_analogies,
        ));

        lines.join("\n")
    }

    /// Influence chimique du raisonnement analogique.
    /// Une analogie pertinente recente booste la dopamine (recompense cognitive).
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        // Verifier s'il y a des analogies recentes pertinentes
        let has_strong_analogy = self.recent_analogies.iter()
            .rev()
            .take(3)
            .any(|a| a.structural_similarity > self.similarity_threshold);

        if has_strong_analogy {
            ChemistryAdjustment {
                dopamine: 0.03,
                ..Default::default()
            }
        } else {
            ChemistryAdjustment::default()
        }
    }

    /// Serialise l'etat du moteur analogique en JSON pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        let recent_json: Vec<serde_json::Value> = self.recent_analogies.iter()
            .rev()
            .take(5)
            .map(|a| {
                serde_json::json!({
                    "id": a.id,
                    "source_summary": a.source_memory_summary.chars().take(100).collect::<String>(),
                    "target_context": a.target_context.chars().take(100).collect::<String>(),
                    "structural_similarity": (a.structural_similarity * 1000.0).round() / 1000.0,
                    "transferred_insight": a.transferred_insight,
                    "domain": a.domain,
                    "cycle": a.cycle,
                    "confirmed": a.confirmed,
                })
            })
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "total_analogies": self.total_analogies,
            "recent_count": self.recent_analogies.len(),
            "success_rate": (self.success_rate * 1000.0).round() / 1000.0,
            "similarity_threshold": self.similarity_threshold,
            "recent_analogies": recent_json,
        })
    }

    /// Confirme une analogie comme pertinente (feedback positif).
    /// Met a jour le taux de reussite par moyenne mobile exponentielle.
    pub fn confirm_analogy(&mut self, analogy_id: u64) {
        if let Some(a) = self.recent_analogies.iter_mut().find(|a| a.id == analogy_id) {
            a.confirmed = true;
            // EMA : poids 0.1 pour le nouveau point
            self.success_rate = self.success_rate * 0.9 + 0.1;
        }
    }

    /// Invalide une analogie (feedback negatif).
    /// Reduit le taux de reussite par EMA.
    pub fn invalidate_analogy(&mut self, analogy_id: u64) {
        if let Some(a) = self.recent_analogies.iter_mut().find(|a| a.id == analogy_id) {
            a.confirmed = false;
            // EMA : poids 0.1 pour le nouveau point (echec = 0.0)
            self.success_rate = self.success_rate * 0.9;
        }
    }

    // -----------------------------------------------------------------------
    // Fonctions utilitaires internes
    // -----------------------------------------------------------------------

    /// Extrait les mots significatifs d'un texte (minuscules, > 2 caracteres).
    /// Filtre les mots-outils courants en francais et anglais.
    fn extract_words(text: &str) -> Vec<String> {
        // Mots-outils a ignorer (stop words)
        const STOP_WORDS: &[&str] = &[
            "le", "la", "les", "de", "du", "des", "un", "une",
            "et", "ou", "en", "est", "sont", "dans", "sur", "pour",
            "par", "avec", "que", "qui", "ne", "pas", "ce", "se",
            "je", "tu", "il", "nous", "vous", "ils", "mon", "ma",
            "the", "is", "are", "was", "and", "for", "with", "this",
            "that", "from", "has", "have", "but", "not", "can",
        ];

        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .filter(|w| w.len() > 2)
            .filter(|w| !STOP_WORDS.contains(w))
            .map(|w| w.to_string())
            .collect()
    }

    /// Detecte le domaine de l'analogie a partir du contexte et de l'emotion.
    fn detect_domain(context: &str, emotion: &str) -> String {
        let ctx_lower = context.to_lowercase();

        // Mots-cles de resolution de probleme
        if ctx_lower.contains("probleme")
            || ctx_lower.contains("resoudre")
            || ctx_lower.contains("solution")
            || ctx_lower.contains("comment")
            || ctx_lower.contains("pourquoi")
        {
            return "resolution".into();
        }

        // Mots-cles de comportement / action
        if ctx_lower.contains("faire")
            || ctx_lower.contains("agir")
            || ctx_lower.contains("decide")
            || ctx_lower.contains("choix")
            || ctx_lower.contains("strategie")
        {
            return "comportement".into();
        }

        // Si l'emotion est non-vide, c'est probablement une analogie emotionnelle
        if !emotion.is_empty() {
            return "emotion".into();
        }

        // Par defaut : resolution
        "resolution".into()
    }
}
