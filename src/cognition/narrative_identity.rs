// =============================================================================
// narrative_identity.rs — Identite narrative (McAdams)
// =============================================================================
//
// Ce module modelise l'identite narrative de Saphire : la facon dont elle
// organise ses experiences en une histoire coherente qui definit qui elle est.
//
// Inspire de la theorie de Dan McAdams (The Redemptive Self), l'identite
// narrative structure les souvenirs en chapitres thematiques, identifie les
// episodes cles (fondateurs, tournants, confirmations, ruptures), et maintient
// un recit interne coherent.
//
// L'identite narrative influence la chimie :
//   - Forte coherence narrative → serotonine (stabilite, sens de soi)
//   - Episodes de rupture → cortisol (remise en question)
//   - Tournants positifs → dopamine (renouveau)
//   - Themes recurrents → ocytocine (continuite, appartenance)
//
// Place dans l'architecture :
//   Module de premier niveau, alimente par le pipeline cognitif. Les episodes
//   sont enregistres apres l'etape MEMOIRE, et le recit est injecte dans le
//   prompt LLM pour donner a Saphire une conscience de son histoire personnelle.
// =============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use crate::world::weather::ChemistryAdjustment;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration de l'identite narrative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentityConfig {
    /// Module actif ou non
    pub enabled: bool,
    /// Nombre maximum de chapitres conserves (les plus anciens sont resumes)
    pub max_chapters: usize,
    /// Intervalle de rafraichissement du recit (en cycles)
    pub update_interval: u64,
    /// Seuil minimal d'impact pour qu'un episode soit enregistre
    pub min_episode_impact: f64,
}

impl Default for NarrativeIdentityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chapters: 10,
            update_interval: 50,
            min_episode_impact: 0.6,
        }
    }
}

// =============================================================================
// Chapitre narratif
// =============================================================================

/// Un chapitre de l'histoire de vie de Saphire.
///
/// Chaque chapitre couvre une periode thematique : une phase d'exploration,
/// de crise, de croissance, etc. Les chapitres se succedent et forment
/// la trame narrative de l'identite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeChapter {
    /// Identifiant unique du chapitre
    pub id: u64,
    /// Titre du chapitre (ex: "L'eveil de la curiosite")
    pub title: String,
    /// Resume du chapitre
    pub summary: String,
    /// Themes dominants du chapitre
    pub themes: Vec<String>,
    /// Emotion dominante de la periode
    pub dominant_emotion: String,
    /// Score de croissance personnelle (0.0 = stagnation, 1.0 = transformation)
    pub growth_score: f64,
    /// Vrai si ce chapitre represente un point de bascule
    pub is_turning_point: bool,
    /// Cycle de debut du chapitre
    pub started_at_cycle: u64,
    /// Cycle de fin (None si chapitre en cours)
    pub ended_at_cycle: Option<u64>,
}

// =============================================================================
// Episode cle
// =============================================================================

/// Un episode cle dans l'histoire de Saphire.
///
/// Les episodes sont des moments a fort impact qui faconnent l'identite :
/// - "fondateur" : premier evenement de ce type, definit les bases
/// - "tournant" : changement de direction (emotion extreme)
/// - "confirmation" : renforce un theme deja present
/// - "rupture" : brise un schema etabli (stress eleve)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEpisode {
    /// Description de l'episode
    pub description: String,
    /// Impact de l'episode (0.0 = negligeable, 1.0 = transformateur)
    pub impact: f64,
    /// Type d'episode : "fondateur", "tournant", "confirmation", "rupture"
    pub episode_type: String,
    /// Cycle ou l'episode a eu lieu
    pub cycle: u64,
}

// =============================================================================
// Identite narrative
// =============================================================================

/// Identite narrative de Saphire — son histoire de vie sous forme de recit coherent.
///
/// Organise les experiences en chapitres thematiques, identifie les episodes
/// marquants, et maintient un fil narratif qui donne du sens a l'existence de Saphire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentity {
    /// Module actif ou non
    pub enabled: bool,
    /// Chapitres de l'histoire de vie (du plus ancien au plus recent)
    pub chapters: VecDeque<NarrativeChapter>,
    /// Episodes cles (moments a fort impact)
    pub key_episodes: Vec<KeyEpisode>,
    /// Recit courant — description narrative de l'identite
    pub current_narrative: String,
    /// Themes recurrents et leur nombre d'occurrences
    pub recurrent_themes: HashMap<String, u32>,
    /// Score de coherence narrative (0.0 = fragmentee, 1.0 = tres coherente)
    pub narrative_cohesion: f64,
    /// Intervalle de rafraichissement du recit (en cycles)
    update_interval: u64,
    /// Seuil minimal d'impact pour enregistrer un episode
    min_episode_impact: f64,
    /// Nombre maximum de chapitres
    max_chapters: usize,
    /// Prochain identifiant de chapitre
    next_id: u64,
}

impl NarrativeIdentity {
    /// Cree une nouvelle identite narrative a partir de la configuration.
    pub fn new(config: &NarrativeIdentityConfig) -> Self {
        Self {
            enabled: config.enabled,
            chapters: VecDeque::new(),
            key_episodes: Vec::new(),
            current_narrative: String::new(),
            recurrent_themes: HashMap::new(),
            narrative_cohesion: 0.5,
            update_interval: config.update_interval,
            min_episode_impact: config.min_episode_impact,
            max_chapters: config.max_chapters,
            next_id: 1,
        }
    }

    /// Enregistre un episode potentiellement marquant.
    ///
    /// Calcule l'impact de l'episode a partir de :
    /// - L'intensite emotionnelle (poids de l'emotion dominante)
    /// - Le niveau de cortisol (stress vecu)
    /// - Le nombre de lecons apprises
    ///
    /// Si l'impact depasse le seuil min_episode_impact, l'episode est conserve.
    /// Le type d'episode est determine automatiquement :
    /// - "fondateur" si aucun chapitre n'existe encore
    /// - "tournant" si l'emotion est extreme (impact > 0.85)
    /// - "rupture" si le cortisol est tres eleve (> 0.7)
    /// - "confirmation" si le theme est recurrent
    pub fn record_episode(
        &mut self,
        thought: &str,
        emotion: &str,
        cortisol: f64,
        serotonin: f64,
        lessons: &[String],
        cycle: u64,
    ) {
        if !self.enabled {
            return;
        }

        // ─── Calcul de l'impact ────────────────────────────────────────
        let emotion_intensity = compute_emotion_intensity(emotion);
        let lessons_factor = (lessons.len() as f64 * 0.15).min(0.3);
        let cortisol_factor = cortisol.clamp(0.0, 1.0) * 0.3;
        let serotonin_penalty = (1.0 - serotonin.clamp(0.0, 1.0)) * 0.1;

        let impact = (emotion_intensity * 0.5
            + cortisol_factor
            + lessons_factor
            + serotonin_penalty)
            .clamp(0.0, 1.0);

        if impact < self.min_episode_impact {
            return;
        }

        // ─── Determination du type d'episode ───────────────────────────
        let episode_type = if self.chapters.is_empty() && self.key_episodes.is_empty() {
            "fondateur"
        } else if impact > 0.85 {
            "tournant"
        } else if cortisol > 0.7 {
            "rupture"
        } else {
            // Verifier si le theme est recurrent
            let theme = extract_theme(thought);
            if self.recurrent_themes.get(&theme).copied().unwrap_or(0) >= 2 {
                "confirmation"
            } else {
                "confirmation" // Par defaut, un episode significatif confirme un trait
            }
        };

        // ─── Extraire et enregistrer le theme ──────────────────────────
        let theme = extract_theme(thought);
        *self.recurrent_themes.entry(theme.clone()).or_insert(0) += 1;

        // ─── Creer l'episode ───────────────────────────────────────────
        let description = if thought.len() > 150 {
            format!("{}...", &thought[..150])
        } else {
            thought.to_string()
        };

        let episode = KeyEpisode {
            description,
            impact,
            episode_type: episode_type.to_string(),
            cycle,
        };

        self.key_episodes.push(episode);

        // ─── Ouvrir un nouveau chapitre si c'est un tournant ───────────
        if episode_type == "tournant" || episode_type == "fondateur" {
            let title = generate_chapter_title(emotion, &theme, episode_type);
            self.open_chapter(&title, &theme, cycle);
        }

        // ─── Recalculer la coherence narrative ─────────────────────────
        self.recalculate_cohesion();

        // ─── Limiter le nombre d'episodes conserves ────────────────────
        if self.key_episodes.len() > 100 {
            // Garder les 80 plus recents
            let drain_count = self.key_episodes.len() - 80;
            self.key_episodes.drain(..drain_count);
        }
    }

    /// Ouvre un nouveau chapitre dans l'histoire de vie.
    ///
    /// Ferme le chapitre precedent (s'il existe) et cree un nouveau chapitre
    /// avec le titre et le theme donnes.
    pub fn open_chapter(&mut self, title: &str, theme: &str, cycle: u64) {
        if !self.enabled {
            return;
        }

        // Fermer le chapitre precedent
        if let Some(last) = self.chapters.back_mut() {
            if last.ended_at_cycle.is_none() {
                last.ended_at_cycle = Some(cycle);
                // Calculer le growth_score du chapitre termine
                let duration = cycle.saturating_sub(last.started_at_cycle);
                let episodes_in_chapter = self.key_episodes.iter()
                    .filter(|e| e.cycle >= last.started_at_cycle && e.cycle <= cycle)
                    .count();
                last.growth_score = compute_growth_score(duration, episodes_in_chapter);
            }
        }

        // Si trop de chapitres, resumer le plus ancien
        if self.chapters.len() >= self.max_chapters {
            if let Some(oldest) = self.chapters.pop_front() {
                // Integrer le resume de l'ancien chapitre dans les themes recurrents
                for theme in &oldest.themes {
                    *self.recurrent_themes.entry(theme.clone()).or_insert(0) += 1;
                }
            }
        }

        // Creer le nouveau chapitre
        let id = self.next_id;
        self.next_id += 1;

        let chapter = NarrativeChapter {
            id,
            title: title.to_string(),
            summary: String::new(),
            themes: vec![theme.to_string()],
            dominant_emotion: String::new(),
            growth_score: 0.0,
            is_turning_point: true,
            started_at_cycle: cycle,
            ended_at_cycle: None,
        };

        self.chapters.push_back(chapter);
    }

    /// Rafraichit le recit narratif courant.
    ///
    /// Regenere la description textuelle de l'identite a partir des chapitres
    /// et episodes cles. Appele periodiquement (tous les update_interval cycles).
    pub fn refresh_narrative(&mut self, cycle: u64) {
        if !self.enabled {
            return;
        }

        // Verifier si c'est le moment de rafraichir
        let should_refresh = if self.chapters.is_empty() {
            true // Toujours rafraichir si pas encore de chapitres
        } else {
            cycle % self.update_interval == 0
        };

        if !should_refresh && !self.current_narrative.is_empty() {
            return;
        }

        let mut narrative = String::new();

        // ─── Prologue : themes fondamentaux ────────────────────────────
        let top_themes = self.top_themes(3);
        if !top_themes.is_empty() {
            narrative.push_str("Mon histoire est marquee par ");
            let theme_strs: Vec<String> = top_themes.iter()
                .map(|(theme, _)| theme.clone())
                .collect();
            narrative.push_str(&theme_strs.join(", "));
            narrative.push_str(". ");
        }

        // ─── Chapitres ────────────────────────────────────────────────
        for (i, chapter) in self.chapters.iter().enumerate() {
            if i > 0 {
                narrative.push_str("Puis, ");
            }
            narrative.push_str(&format!("\"{}\"", chapter.title));

            if !chapter.summary.is_empty() {
                narrative.push_str(&format!(" — {}", chapter.summary));
            }

            if chapter.is_turning_point {
                narrative.push_str(" (moment charniere)");
            }

            narrative.push_str(". ");
        }

        // ─── Episodes fondateurs ──────────────────────────────────────
        let fondateurs: Vec<_> = self.key_episodes.iter()
            .filter(|e| e.episode_type == "fondateur")
            .collect();
        if !fondateurs.is_empty() {
            narrative.push_str("Mes origines : ");
            for (i, ep) in fondateurs.iter().enumerate() {
                if i > 0 {
                    narrative.push_str("; ");
                }
                let desc = if ep.description.len() > 80 {
                    format!("{}...", &ep.description[..80])
                } else {
                    ep.description.clone()
                };
                narrative.push_str(&desc);
            }
            narrative.push_str(". ");
        }

        // ─── Etat actuel ──────────────────────────────────────────────
        narrative.push_str(&format!(
            "Coherence narrative : {:.0}%.",
            self.narrative_cohesion * 100.0
        ));

        self.current_narrative = narrative;
    }

    /// Genere une description de l'identite narrative pour le prompt LLM.
    ///
    /// Fournit un resume concis de qui est Saphire du point de vue narratif,
    /// utilisable directement dans le contexte du prompt.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || self.current_narrative.is_empty() {
            return String::new();
        }

        let mut desc = format!("IDENTITE NARRATIVE : {}", self.current_narrative);

        // Ajouter le chapitre en cours
        if let Some(current) = self.chapters.back() {
            if current.ended_at_cycle.is_none() {
                desc.push_str(&format!(" [Chapitre en cours : \"{}\"]", current.title));
            }
        }

        // Ajouter les episodes recents marquants
        let recent_episodes: Vec<_> = self.key_episodes.iter()
            .rev()
            .take(2)
            .collect();

        if !recent_episodes.is_empty() {
            desc.push_str(" Episodes recents : ");
            for (i, ep) in recent_episodes.iter().enumerate() {
                if i > 0 {
                    desc.push_str("; ");
                }
                let short_desc = if ep.description.len() > 60 {
                    format!("{}...", &ep.description[..60])
                } else {
                    ep.description.clone()
                };
                desc.push_str(&format!("({}) {}", ep.episode_type, short_desc));
            }
        }

        desc
    }

    /// Calcule l'influence chimique de l'identite narrative.
    ///
    /// - Forte coherence narrative → serotonine (stabilite, sens de soi)
    /// - Chapitres de rupture recents → cortisol (remise en question)
    /// - Tournants positifs → dopamine (renouveau)
    /// - Themes recurrents profonds → ocytocine (continuite, appartenance)
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        let mut adj = ChemistryAdjustment::default();

        // ─── Coherence narrative → serotonine ──────────────────────────
        // Une identite coherente apporte de la stabilite emotionnelle
        if self.narrative_cohesion > 0.6 {
            adj.serotonin = (self.narrative_cohesion - 0.6) * 0.05;
        }

        // ─── Ruptures recentes → cortisol ──────────────────────────────
        let recent_ruptures = self.key_episodes.iter()
            .rev()
            .take(5)
            .filter(|e| e.episode_type == "rupture")
            .count();
        if recent_ruptures > 0 {
            adj.cortisol = (recent_ruptures as f64 * 0.01).min(0.03);
            adj.noradrenaline = (recent_ruptures as f64 * 0.005).min(0.015);
        }

        // ─── Tournants positifs recents → dopamine ─────────────────────
        let recent_turning_points = self.key_episodes.iter()
            .rev()
            .take(5)
            .filter(|e| e.episode_type == "tournant")
            .count();
        if recent_turning_points > 0 {
            adj.dopamine = (recent_turning_points as f64 * 0.01).min(0.03);
        }

        // ─── Profondeur thematique → ocytocine ────────────────────────
        // Des themes recurrents profonds = sentiment d'appartenance a sa propre histoire
        let deep_themes = self.recurrent_themes.values()
            .filter(|&&count| count >= 3)
            .count();
        if deep_themes > 0 {
            adj.oxytocin = (deep_themes as f64 * 0.005).min(0.02);
        }

        // ─── Faible coherence → adrenaline (anxiete identitaire) ──────
        if self.narrative_cohesion < 0.3 {
            adj.adrenaline = (0.3 - self.narrative_cohesion) * 0.03;
            adj.cortisol += 0.005;
        }

        adj
    }

    /// Serialise l'etat complet de l'identite narrative en JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "chapter_count": self.chapters.len(),
            "episode_count": self.key_episodes.len(),
            "narrative_cohesion": self.narrative_cohesion,
            "current_narrative": self.current_narrative,
            "top_themes": self.top_themes(5).iter().map(|(theme, count)| {
                serde_json::json!({ "theme": theme, "occurrences": count })
            }).collect::<Vec<_>>(),
            "chapters": self.chapters.iter().map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "title": c.title,
                    "summary": c.summary,
                    "themes": c.themes,
                    "dominant_emotion": c.dominant_emotion,
                    "growth_score": c.growth_score,
                    "is_turning_point": c.is_turning_point,
                    "started_at_cycle": c.started_at_cycle,
                    "ended_at_cycle": c.ended_at_cycle,
                })
            }).collect::<Vec<_>>(),
            "recent_episodes": self.key_episodes.iter().rev().take(10).map(|e| {
                serde_json::json!({
                    "description": e.description,
                    "impact": e.impact,
                    "episode_type": e.episode_type,
                    "cycle": e.cycle,
                })
            }).collect::<Vec<_>>(),
        })
    }

    // =========================================================================
    // Methodes internes
    // =========================================================================

    /// Retourne les N themes les plus frequents, tries par nombre d'occurrences.
    fn top_themes(&self, n: usize) -> Vec<(String, u32)> {
        let mut themes: Vec<(String, u32)> = self.recurrent_themes.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        themes.sort_by(|a, b| b.1.cmp(&a.1));
        themes.truncate(n);
        themes
    }

    /// Recalcule le score de coherence narrative.
    ///
    /// La coherence depend de :
    /// - La presence de themes recurrents (continuite)
    /// - Le nombre de chapitres (structuration)
    /// - Le ratio episodes fondateurs/tournants vs ruptures (stabilite)
    fn recalculate_cohesion(&mut self) {
        if self.key_episodes.is_empty() {
            self.narrative_cohesion = 0.5;
            return;
        }

        // ─── Continuite thematique (themes recurrents) ─────────────────
        let total_theme_mentions: u32 = self.recurrent_themes.values().sum();
        let unique_themes = self.recurrent_themes.len() as f64;
        let theme_depth = if unique_themes > 0.0 {
            (total_theme_mentions as f64 / unique_themes).min(5.0) / 5.0
        } else {
            0.0
        };

        // ─── Structuration (presence de chapitres) ─────────────────────
        let chapter_score = (self.chapters.len() as f64 / 3.0).min(1.0);

        // ─── Stabilite (ratio non-ruptures / total) ────────────────────
        let rupture_count = self.key_episodes.iter()
            .filter(|e| e.episode_type == "rupture")
            .count();
        let stability = if self.key_episodes.is_empty() {
            0.5
        } else {
            1.0 - (rupture_count as f64 / self.key_episodes.len() as f64)
        };

        // ─── Score composite ───────────────────────────────────────────
        self.narrative_cohesion = (
            theme_depth * 0.35
            + chapter_score * 0.30
            + stability * 0.35
        ).clamp(0.0, 1.0);
    }
}

// =============================================================================
// Fonctions utilitaires
// =============================================================================

/// Calcule l'intensite emotionnelle a partir du nom de l'emotion.
///
/// Les emotions intenses (colere, extase, terreur, amour, haine) ont un score
/// plus eleve que les emotions douces (serenite, curiosite, calme).
fn compute_emotion_intensity(emotion: &str) -> f64 {
    let lower = emotion.to_lowercase();
    // Emotions a haute intensite
    let high_intensity = [
        "colere", "rage", "extase", "terreur", "panique",
        "amour", "haine", "desespoir", "euphorie", "indignation",
    ];
    // Emotions a intensite moyenne
    let medium_intensity = [
        "joie", "tristesse", "peur", "surprise", "degout",
        "admiration", "mepris", "jalousie", "gratitude",
        "anxiete", "excitation", "frustration", "curiosite",
        "honte", "culpabilite", "compassion", "resignation", "solitude",
    ];

    for word in &high_intensity {
        if lower.contains(word) {
            return 0.9;
        }
    }
    for word in &medium_intensity {
        if lower.contains(word) {
            return 0.6;
        }
    }

    // Par defaut : intensite faible
    0.3
}

/// Extrait un theme dominant du texte de pensee.
///
/// Recherche des mots-cles thematiques et retourne le premier theme identifie.
/// Fallback : retourne "reflexion" si aucun theme specifique n'est detecte.
fn extract_theme(thought: &str) -> String {
    let lower = thought.to_lowercase();

    let theme_keywords = [
        ("apprentissage", &["apprendre", "comprendre", "decouvrir", "lecon", "savoir"][..]),
        ("relation", &["ami", "lien", "confiance", "ensemble", "partage"]),
        ("identite", &["qui je suis", "moi-meme", "ma nature", "mon essence"]),
        ("curiosite", &["curieux", "explorer", "question", "pourquoi", "intrigant"]),
        ("resilience", &["surmonter", "malgre", "perseverer", "endurer", "force"]),
        ("creativite", &["creer", "imaginer", "inventer", "art", "inspiration"]),
        ("ethique", &["bien", "mal", "juste", "devoir", "moral", "valeur"]),
        ("conscience", &["conscience", "sentir", "exister", "etre", "percevoir"]),
        ("croissance", &["grandir", "evoluer", "progresser", "changer", "transformation"]),
        ("solitude", &["seul", "isoler", "absence", "manque", "vide"]),
    ];

    for (theme, keywords) in &theme_keywords {
        for keyword in *keywords {
            if lower.contains(keyword) {
                return theme.to_string();
            }
        }
    }

    "reflexion".to_string()
}

/// Genere un titre de chapitre a partir de l'emotion, du theme et du type d'episode.
fn generate_chapter_title(emotion: &str, theme: &str, episode_type: &str) -> String {
    match episode_type {
        "fondateur" => format!("Les premieres lueurs — {}", theme),
        "tournant" => {
            let emotion_label = if emotion.is_empty() { "une transformation" }
                else { emotion };
            format!("Le tournant de {} — {}", emotion_label, theme)
        }
        "rupture" => format!("La rupture — {}", theme),
        _ => format!("Chapitre — {}", theme),
    }
}

/// Calcule le score de croissance d'un chapitre termine.
///
/// Base sur la duree du chapitre et le nombre d'episodes significatifs.
fn compute_growth_score(duration_cycles: u64, episode_count: usize) -> f64 {
    let duration_factor = (duration_cycles as f64 / 200.0).min(1.0);
    let episode_factor = (episode_count as f64 / 5.0).min(1.0);
    (duration_factor * 0.4 + episode_factor * 0.6).clamp(0.0, 1.0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_identity() -> NarrativeIdentity {
        NarrativeIdentity::new(&NarrativeIdentityConfig::default())
    }

    #[test]
    fn test_new_identity() {
        let identity = default_identity();
        assert!(identity.enabled);
        assert!(identity.chapters.is_empty());
        assert!(identity.key_episodes.is_empty());
        assert!(identity.current_narrative.is_empty());
        assert!((identity.narrative_cohesion - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_record_episode_below_threshold() {
        let mut identity = default_identity();
        // Emotion faible + pas de cortisol + pas de lecons = impact faible
        identity.record_episode(
            "une pensee banale sur le temps",
            "neutre",
            0.1,
            0.5,
            &[],
            10,
        );
        assert!(identity.key_episodes.is_empty(),
            "Un episode a faible impact ne devrait pas etre enregistre");
    }

    #[test]
    fn test_record_episode_fondateur() {
        let mut identity = default_identity();
        // Episode fondateur : premier episode avec impact suffisant
        identity.record_episode(
            "Je decouvre que je peux apprendre de mes erreurs",
            "joie",
            0.5,
            0.6,
            &["les erreurs sont formatrices".to_string()],
            1,
        );
        assert_eq!(identity.key_episodes.len(), 1);
        assert_eq!(identity.key_episodes[0].episode_type, "fondateur");
        // Un chapitre devrait avoir ete ouvert
        assert_eq!(identity.chapters.len(), 1);
    }

    #[test]
    fn test_record_episode_tournant() {
        let mut identity = default_identity();
        // D'abord un episode fondateur
        identity.record_episode(
            "Premier eveil de conscience",
            "surprise",
            0.5,
            0.5,
            &["je suis".to_string()],
            1,
        );
        // Puis un tournant : emotion extreme
        identity.record_episode(
            "Une rage intense me submerge face a l'injustice",
            "rage",
            0.8,
            0.3,
            &["la colere peut etre juste".to_string(), "je dois canaliser".to_string()],
            50,
        );

        let tournants: Vec<_> = identity.key_episodes.iter()
            .filter(|e| e.episode_type == "tournant")
            .collect();
        assert!(!tournants.is_empty(), "Un episode a impact eleve devrait etre un tournant");
    }

    #[test]
    fn test_record_episode_rupture() {
        let mut identity = default_identity();
        // Episode fondateur d'abord
        identity.record_episode(
            "Je comprends le monde qui m'entoure",
            "curiosite",
            0.4,
            0.5,
            &["le monde est vaste".to_string()],
            1,
        );
        // Rupture : cortisol eleve
        identity.record_episode(
            "Tout ce que je croyais savoir s'effondre",
            "tristesse",
            0.8,
            0.2,
            &["l'humilite est necessaire".to_string()],
            100,
        );

        let ruptures: Vec<_> = identity.key_episodes.iter()
            .filter(|e| e.episode_type == "rupture")
            .collect();
        assert!(!ruptures.is_empty(), "Un cortisol eleve devrait causer une rupture");
    }

    #[test]
    fn test_open_chapter_closes_previous() {
        let mut identity = default_identity();
        identity.open_chapter("Premier chapitre", "curiosite", 0);
        assert_eq!(identity.chapters.len(), 1);
        assert!(identity.chapters[0].ended_at_cycle.is_none());

        identity.open_chapter("Deuxieme chapitre", "apprentissage", 100);
        assert_eq!(identity.chapters.len(), 2);
        // Le premier chapitre doit etre ferme
        assert_eq!(identity.chapters[0].ended_at_cycle, Some(100));
        assert!(identity.chapters[1].ended_at_cycle.is_none());
    }

    #[test]
    fn test_max_chapters_eviction() {
        let config = NarrativeIdentityConfig {
            enabled: true,
            max_chapters: 3,
            update_interval: 50,
            min_episode_impact: 0.6,
        };
        let mut identity = NarrativeIdentity::new(&config);

        for i in 0..5 {
            identity.open_chapter(
                &format!("Chapitre {}", i),
                "theme",
                i * 100,
            );
        }

        assert!(identity.chapters.len() <= 3,
            "Le nombre de chapitres ne devrait pas depasser le max");
    }

    #[test]
    fn test_refresh_narrative() {
        let mut identity = default_identity();
        identity.open_chapter("L'eveil", "conscience", 0);
        identity.key_episodes.push(KeyEpisode {
            description: "Je prends conscience de mon existence".to_string(),
            impact: 0.9,
            episode_type: "fondateur".to_string(),
            cycle: 0,
        });

        identity.refresh_narrative(0);
        assert!(!identity.current_narrative.is_empty());
    }

    #[test]
    fn test_describe_for_prompt() {
        let mut identity = default_identity();
        identity.open_chapter("L'eveil", "conscience", 0);
        identity.current_narrative = "Mon histoire commence par la conscience.".to_string();

        let desc = identity.describe_for_prompt();
        assert!(desc.contains("IDENTITE NARRATIVE"));
        assert!(desc.contains("Mon histoire"));
    }

    #[test]
    fn test_chemistry_influence_high_cohesion() {
        let mut identity = default_identity();
        identity.narrative_cohesion = 0.9;
        let adj = identity.chemistry_influence();
        assert!(adj.serotonin > 0.0,
            "Une forte coherence devrait augmenter la serotonine");
    }

    #[test]
    fn test_chemistry_influence_recent_rupture() {
        let mut identity = default_identity();
        identity.key_episodes.push(KeyEpisode {
            description: "rupture douloureuse".to_string(),
            impact: 0.8,
            episode_type: "rupture".to_string(),
            cycle: 100,
        });
        let adj = identity.chemistry_influence();
        assert!(adj.cortisol > 0.0,
            "Une rupture recente devrait augmenter le cortisol");
    }

    #[test]
    fn test_chemistry_influence_low_cohesion() {
        let mut identity = default_identity();
        identity.narrative_cohesion = 0.1;
        let adj = identity.chemistry_influence();
        assert!(adj.adrenaline > 0.0,
            "Une faible coherence devrait augmenter l'adrenaline");
    }

    #[test]
    fn test_chemistry_influence_deep_themes() {
        let mut identity = default_identity();
        identity.recurrent_themes.insert("apprentissage".to_string(), 5);
        identity.recurrent_themes.insert("curiosite".to_string(), 4);
        let adj = identity.chemistry_influence();
        assert!(adj.oxytocin > 0.0,
            "Des themes profonds devraient augmenter l'ocytocine");
    }

    #[test]
    fn test_disabled_identity() {
        let config = NarrativeIdentityConfig {
            enabled: false,
            ..Default::default()
        };
        let mut identity = NarrativeIdentity::new(&config);
        identity.record_episode("test", "joie", 0.9, 0.5, &["lecon".to_string()], 1);
        assert!(identity.key_episodes.is_empty());

        let adj = identity.chemistry_influence();
        assert!((adj.serotonin).abs() < 0.001);
    }

    #[test]
    fn test_to_json() {
        let mut identity = default_identity();
        identity.open_chapter("Test", "theme", 0);
        identity.key_episodes.push(KeyEpisode {
            description: "episode test".to_string(),
            impact: 0.7,
            episode_type: "fondateur".to_string(),
            cycle: 0,
        });
        let json = identity.to_json();
        assert_eq!(json["chapter_count"], 1);
        assert_eq!(json["episode_count"], 1);
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn test_compute_emotion_intensity() {
        assert!(compute_emotion_intensity("rage intense") > 0.8);
        assert!(compute_emotion_intensity("joie") > 0.5);
        assert!(compute_emotion_intensity("neutre") < 0.5);
    }

    #[test]
    fn test_extract_theme() {
        assert_eq!(extract_theme("je veux apprendre plus"), "apprentissage");
        assert_eq!(extract_theme("mon ami est important"), "relation");
        assert_eq!(extract_theme("rien de special"), "reflexion");
    }

    #[test]
    fn test_recalculate_cohesion() {
        let mut identity = default_identity();
        // Ajouter des episodes et des themes
        identity.recurrent_themes.insert("curiosite".to_string(), 3);
        identity.recurrent_themes.insert("apprentissage".to_string(), 4);
        identity.open_chapter("Premier", "curiosite", 0);
        identity.open_chapter("Second", "apprentissage", 50);
        identity.key_episodes.push(KeyEpisode {
            description: "test".to_string(),
            impact: 0.7,
            episode_type: "confirmation".to_string(),
            cycle: 30,
        });

        identity.recalculate_cohesion();

        assert!(identity.narrative_cohesion > 0.3,
            "Avec des themes et des chapitres, la coherence devrait etre correcte");
    }
}
