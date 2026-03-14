// =============================================================================
// lifecycle/thinking.rs — Orchestrateur de la pensee autonome
// =============================================================================
//
// Ce fichier contient l'orchestrateur principal (autonomous_think) qui appelle
// les phases reparties dans les sous-fichiers :
//   - thinking_perception.rs   — Phases pre-LLM (perception du monde)
//   - thinking_preparation.rs  — Phases de selection et preparation du prompt
//   - thinking_processing.rs   — Phases LLM et post-traitement immediat
//   - thinking_reflection.rs   — Phases de reflexion, apprentissage, psychologie
//
// Ce fichier contient aussi les structures partagees :
//   - ThinkingContext : contexte mutable partage entre toutes les phases
//   - FeedbackRequest : demande de feedback humain en attente
//   - strip_chemical_trace() : nettoyage de la trace chimique LLM
//   - is_positive_feedback() : analyse simple du feedback humain
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;

use crate::emotions::EmotionalState;
use super::SaphireAgent;
use super::ProcessResult;

// =============================================================================
// FeedbackRequest — Demande de feedback humain en attente
// =============================================================================

/// Demande de feedback humain en attente de reponse.
/// Stockee dans `SaphireAgent.feedback_pending` quand Saphire pose une question.
#[allow(dead_code)]
pub(super) struct FeedbackRequest {
    pub thought_text: String,
    pub thought_type: crate::agent::thought_engine::ThoughtType,
    pub auto_reward: f64,
    pub asked_at_cycle: u64,
}

/// Analyse simple du feedback humain sans appel LLM supplementaire.
/// Retourne true si le feedback est globalement positif (approbation claire).
/// Les messages correctifs ("oui mais...", "plus simplement...") sont negatifs.
/// Analyse le sentiment d'un feedback humain via le LLM.
/// Fallback sur heuristique simple si le LLM echoue.
pub(super) async fn is_positive_feedback_llm(response: &str, llm_config: &crate::llm::LlmConfig) -> bool {
    let backend = crate::llm::create_backend(llm_config);
    let system = "Tu es un analyseur de sentiment. On te donne un message humain envoye en reponse \
                  a une question posee par une IA. Determine si le sentiment global du message est \
                  positif (encouragement, accord, interet, compliment, soutien) ou negatif \
                  (desaccord, critique, correction, rejet). \
                  Reponds UNIQUEMENT par le mot \"positif\" ou \"negatif\".".to_string();
    let user = response.to_string();

    let result = tokio::task::spawn_blocking(move || {
        backend.chat(&system, &user, 0.1, 5)
    }).await;

    match result {
        Ok(Ok(answer)) => {
            let lower = answer.trim().to_lowercase();
            if lower.contains("positif") {
                true
            } else if lower.contains("negatif") || lower.contains("négatif") {
                false
            } else {
                tracing::warn!("Feedback LLM: reponse inattendue '{}', fallback heuristique", answer.trim());
                is_positive_feedback_heuristic(response)
            }
        }
        _ => {
            tracing::warn!("Feedback LLM: echec appel, fallback heuristique");
            is_positive_feedback_heuristic(response)
        }
    }
}

/// Heuristique simple de fallback (ancienne methode).
fn is_positive_feedback_heuristic(response: &str) -> bool {
    let lower = response.to_lowercase();
    let positive = [
        "oui", "bien", "exact", "d'accord", "interessant", "bravo",
        "continue", "j'aime", "genial", "super", "bon", "vrai",
        "absolument", "tout a fait", "en effet", "bonne", "yes",
        "great", "good", "nice", "right", "agree", "cool",
    ];
    let negative = [
        "non", "pas d'accord", "faux", "incorrect", "mauvais",
        "arrete", "stop", "ridicule", "n'importe quoi", "absurde",
        "no", "wrong", "bad", "disagree",
    ];
    let pos_score: usize = positive.iter().filter(|w| lower.contains(*w)).count();
    let neg_score: usize = negative.iter().filter(|w| lower.contains(*w)).count();
    pos_score > neg_score
}

/// Retire la trace chimique du debut d'une pensee LLM.
/// Format typique : "C[sero,0.75,ocyt,0.55] E:Espoir+Curiosite V+0.90 A0.55 Texte reel..."
/// Retourne le texte nettoye, pret a etre affiche a l'humain.
pub(super) fn strip_chemical_trace(text: &str) -> String {
    let t = text.trim();
    // Chercher la fin du header chimique : apres le dernier champ numerique (V+x.xx ou Ax.xx)
    // Le pattern est : C[...] E:... V+x.xx Ax.xx <texte>
    // Strategie : trouver le premier caractere alphabetique apres un pattern Ax.xx ou V+x.xx
    if let Some(c_start) = t.find("C[") {
        // Chercher la fin du bloc : on saute les tokens chimiques
        let after_c = &t[c_start..];
        // Chercher un pattern " A" suivi d'un chiffre puis trouver le texte apres
        if let Some(a_pos) = after_c.rfind(" A") {
            let rest = &after_c[a_pos + 2..];
            // Sauter le nombre (ex: "0.55")
            let skip = rest.find(|c: char| c == ' ').unwrap_or(rest.len());
            if skip < rest.len() {
                return rest[skip..].trim().to_string();
            }
        }
        // Fallback : chercher apres le "]" le vrai texte (sauter E:, V+, A tokens)
        if let Some(bracket_end) = after_c.find(']') {
            let rest = &after_c[bracket_end + 1..].trim();
            // Sauter les tokens E:, V+, A qui sont courts
            let chars = rest.chars().peekable();
            let mut pos = 0;
            let bytes = rest.as_bytes();
            while pos < rest.len() {
                // Sauter les espaces
                while pos < rest.len() && bytes[pos] == b' ' { pos += 1; }
                if pos >= rest.len() { break; }
                // Token E:...
                if rest[pos..].starts_with("E:") {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                // Token V+ ou V-
                if rest[pos..].starts_with("V+") || rest[pos..].starts_with("V-") {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                // Token A suivi d'un chiffre
                if bytes[pos] == b'A' && pos + 1 < rest.len() && bytes[pos+1].is_ascii_digit() {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                break;
            }
            let _ = chars;
            return rest[pos..].trim().to_string();
        }
    }
    t.to_string()
}

// =============================================================================
// ThinkingContext — Contexte mutable partage entre toutes les phases
// =============================================================================

/// Contexte mutable partage entre toutes les phases de la pensee autonome.
///
/// Ce struct regroupe toutes les variables intermediaires qui etaient
/// auparavant des variables locales dans autonomous_think(). Chaque phase
/// lit et/ou ecrit dans ce contexte, eliminant le besoin de passer
/// 15+ parametres entre les fonctions.
pub(super) struct ThinkingContext {
    /// Instant de debut du cycle pour mesurer la duree totale
    pub cycle_start: Instant,

    /// Type de pensee selectionne par le bandit UCB1
    pub thought_type: crate::agent::thought_engine::ThoughtType,

    /// Indice de variante pour alterner les prompts
    pub variant: usize,

    /// Etat emotionnel calcule a partir de la chimie courante
    pub emotion: EmotionalState,

    /// Contexte de connaissance web (texte + KnowledgeResult)
    pub knowledge_context: Option<(String, crate::knowledge::KnowledgeResult)>,

    /// Flag indiquant si une recherche web a eu lieu
    pub was_web_search: bool,

    /// Hint textuel pour le prompt LLM
    pub hint: String,

    /// Resume du monde (meteo, heure, etc.)
    pub world_summary: String,

    /// Contexte memoire construit pour le prompt
    pub memory_context: String,

    /// Patterns intuitifs detectes avant le LLM
    pub intuition_patterns: Vec<crate::vital::intuition::IntuitionPattern>,

    /// Nouvelles premonitions generees
    pub new_premonitions: Vec<crate::vital::premonition::Premonition>,

    /// Prompt systeme (statique, cacheable KV-cache)
    pub system_prompt: String,

    /// Prompt dynamique (message utilisateur)
    pub prompt: String,

    /// Texte de la pensee generee par le LLM
    pub thought_text: String,

    /// Temps de reponse du LLM en secondes
    pub llm_elapsed: f64,

    /// Resultat du pipeline cerebral (consensus, emotion, conscience)
    pub process_result: Option<ProcessResult>,

    /// Recompense UCB1 calculee pour ce cycle
    pub reward: f64,

    /// Flag indiquant si un element a ete ejecte de la memoire de travail
    pub had_wm_ejection: bool,

    /// Flag indiquant qu'il faut abandonner le cycle (erreur LLM)
    pub should_abort: bool,

    /// Deliberation volontaire eventuelle de ce cycle
    pub deliberation: Option<crate::psychology::will::Deliberation>,

    /// Nombre total d'apprentissages vectoriels (pour metrics)
    pub nn_learnings_count: i32,

    /// Qualite de pensee evaluee par metacognition (0.0-1.0)
    pub quality: f64,

    /// Donnees de rappel memoire pour la trace cognitive
    pub memory_trace_data: serde_json::Value,

    /// Hint d'analogie pour le prompt (raisonnement analogique M6)
    pub analogy_hint: String,

    /// Associations trouvees par le connectome (A* pathfinding)
    pub connectome_associations: String,

    /// Ancrage experiential : contexte concret pour enrichir la pensee
    pub anchor: Option<String>,

    /// Tension du reseau MAP (ecart perception/reaction cerebrale)
    pub network_tension: f64,

    /// Cadre auto-formule par Saphire (self-framing) : metriques, angle, profondeur
    pub self_framing: Option<String>,
}

impl ThinkingContext {
    /// Cree un nouveau contexte avec des valeurs par defaut.
    /// Les vrais contenus sont remplis par chaque phase.
    pub(super) fn new() -> Self {
        Self {
            cycle_start: Instant::now(),
            thought_type: crate::agent::thought_engine::ThoughtType::Introspection,
            variant: 0,
            emotion: EmotionalState {
                dominant: String::new(),
                dominant_similarity: 0.0,
                secondary: None,
                valence: 0.0,
                arousal: 0.0,
                spectrum: Vec::new(),
                core_valence: 0.0,
                core_arousal: 0.0,
                context_influence: String::new(),
            },
            knowledge_context: None,
            was_web_search: false,
            hint: String::new(),
            world_summary: String::new(),
            memory_context: String::new(),
            intuition_patterns: Vec::new(),
            new_premonitions: Vec::new(),
            system_prompt: String::new(),
            prompt: String::new(),
            thought_text: String::new(),
            llm_elapsed: 0.0,
            process_result: None,
            reward: 0.0,
            had_wm_ejection: false,
            should_abort: false,
            deliberation: None,
            nn_learnings_count: 0,
            quality: 0.0,
            memory_trace_data: serde_json::json!({}),
            analogy_hint: String::new(),
            connectome_associations: String::new(),
            anchor: None,
            network_tension: 0.0,
            self_framing: None,
        }
    }
}

// =============================================================================
// Orchestrateur principal — autonomous_think()
// =============================================================================

impl SaphireAgent {
    /// Pensee autonome : generee quand aucun humain n'interagit avec Saphire.
    ///
    /// Cette methode est appelee periodiquement par la boucle de vie (toutes
    /// les `thought_interval` secondes). Elle orchestre ~55 phases via un
    /// ThinkingContext partage. Chaque phase est une methode nommee definie
    /// dans les sous-fichiers thinking_*.rs.
    ///
    /// Retourne : `Some(texte)` si une pensee a ete generee, `None` si le LLM etait occupe.
    pub async fn autonomous_think(&mut self) -> Option<String> {
        if self.llm_busy.load(Ordering::Relaxed) {
            return None;
        }
        let mut ctx = ThinkingContext::new();

        // Phases pre-LLM : mise a jour du monde et de l'etat interne
        self.phase_init(&mut ctx);
        self.phase_weather_and_body(&mut ctx);
        self.phase_needs(&mut ctx);
        self.phase_vital_spark(&mut ctx).await;
        self.phase_senses(&mut ctx);
        self.phase_map_sync(&mut ctx);             // MAP : synchronise BrainNetwork + Connectome
        self.phase_chemistry_history(&mut ctx);
        self.phase_birthday(&mut ctx).await;
        self.phase_world_broadcast(&mut ctx);
        self.phase_memory_decay(&mut ctx).await;
        self.phase_conversation_timeout(&mut ctx).await;
        self.phase_episodic_decay(&mut ctx).await;
        self.phase_consolidation(&mut ctx).await;
        self.phase_auto_algorithms(&mut ctx).await;

        // Phases de selection et preparation du prompt
        self.phase_select_thought(&mut ctx);
        self.phase_generate_dynamic_prompt(&mut ctx).await;
        self.phase_connectome_associations(&mut ctx); // GA1 : A* pathfinding connectome
        self.phase_prospective(&mut ctx);           // M4 : memoire prospective
        self.phase_web_search(&mut ctx).await;
        self.phase_build_context(&mut ctx).await;
        self.phase_analogies(&mut ctx);             // M6 : raisonnement analogique
        self.phase_intuition_premonition(&mut ctx);
        self.phase_orchestrators(&mut ctx).await;
        self.phase_cognitive_load(&mut ctx);        // M7 : charge cognitive
        self.phase_build_prompt(&mut ctx);          // inclut continuation monologue M2
        self.phase_deliberation(&mut ctx);

        // Phase LLM
        self.phase_call_llm(&mut ctx).await;
        if ctx.should_abort {
            return None;
        }

        // Phases post-LLM : traitement de la reponse
        self.phase_llm_history(&mut ctx);
        self.phase_vectorial_filter(&mut ctx);         // P2 : filtrage vectoriel anti-repetition
        self.phase_drift_check(&mut ctx);              // P0 : moniteur de derive de persona
        if ctx.should_abort {
            return None;
        }
        self.phase_algorithm_request(&mut ctx).await;
        self.phase_pipeline(&mut ctx);
        self.phase_monologue(&mut ctx);             // M2 : monologue interieur
        self.phase_dissonance(&mut ctx);            // M3 : dissonance cognitive
        self.phase_imagery(&mut ctx).await;           // M9 : imagerie mentale
        self.phase_sentiments(&mut ctx);              // Sentiments (etats affectifs durables)
        self.phase_state_clustering(&mut ctx);         // PCA + K-Means etat cognitif
        self.phase_working_memory(&mut ctx).await;
        self.phase_memory_echo(&mut ctx).await;
        self.phase_reward_and_ethics(&mut ctx).await;
        self.phase_verify_predictions(&mut ctx);
        self.phase_maybe_ask_feedback(&mut ctx);
        self.phase_lora_collect(&mut ctx).await;
        self.phase_knowledge_bonus(&mut ctx).await;
        self.phase_thought_log(&mut ctx).await;
        self.phase_profiling(&mut ctx).await;
        self.phase_cognitive_trace(&mut ctx);
        self.phase_broadcast(&mut ctx).await;
        self.phase_metrics(&mut ctx);
        self.phase_learning(&mut ctx).await;
        self.phase_nn_learning(&mut ctx).await;
        self.phase_metacognition(&mut ctx).await;   // inclut M8+M10
        self.phase_self_critique(&mut ctx).await;  // Auto-critique reflexive (periodique)
        self.phase_personality_snapshot(&mut ctx).await;  // Portrait temporel (50 cycles)
        self.phase_introspection_journal(&mut ctx).await; // Journal introspectif (200 cycles)
        self.phase_desire_birth(&mut ctx).await;
        self.phase_self_modification(&mut ctx).await;  // Auto-modification niveaux 1+2
        self.phase_psychology(&mut ctx);
        self.phase_values(&mut ctx);                // Valeurs de caractere (vertus)
        self.phase_narrative(&mut ctx);             // M5 : identite narrative
        self.phase_behavior_tree(&mut ctx);          // BT : instinct cognitif
        self.phase_game_algorithms(&mut ctx);       // GA2 : influence map, FSM, steering, GOAP
        self.phase_homeostasis(&mut ctx);

        // Filtrage des termes techniques internes avant affichage
        ctx.thought_text = super::conversation::strip_internal_jargon(&ctx.thought_text);

        Some(ctx.thought_text)
    }
}
