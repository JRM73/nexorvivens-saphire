// =============================================================================
// lifecycle/thinking_reflection.rs — Learning and homeostasis (lite version)
// =============================================================================
//
// This file contains the post-LLM reflection phases in the lite version.
// Removed modules (orchestrators, metacognition, psychology, sentiments,
// connectome, narrative, etc.) are reduced to empty stubs.
//
// Active phases:
//   - phase_self_critique         — self-critique via LLM (simplified)
//   - phase_personality_snapshot  — chemistry + emotion + consciousness snapshot
//   - phase_introspection_journal — introspective journal entry via LLM
//   - phase_homeostasis           — chemical homeostasis
//
// Removed phases (empty stubs):
//   - phase_learning, phase_nn_learning, try_formulate_nn_learning
//   - phase_metacognition, phase_desire_birth, phase_psychology
//   - phase_prospective, phase_analogies, phase_cognitive_load
//   - phase_monologue, phase_dissonance, phase_imagery, phase_sentiments
//   - phase_narrative, phase_state_clustering, phase_connectome_associations
//   - phase_game_algorithms
// =============================================================================

use crate::llm;
use crate::memory::WorkingItemSource;
use crate::neurochemistry::Molecule;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
use super::truncate_utf8;
use super::thinking::ThinkingContext;

impl SaphireAgent {
    // =========================================================================
    // Phase 31: Periodic learning — STUB (learning_orch removed)
    // =========================================================================

    /// Stub: learning orchestrator is removed in the lite version.
    pub(super) async fn phase_learning(&mut self, _ctx: &mut ThinkingContext) {
        // learning_orch removed in the lite version
    }

    // =========================================================================
    // Phase 31b: Vector learning — STUB (encoder + micro_nn removed)
    // =========================================================================

    /// Stub: micro neural network and encoder are removed in the lite version.
    pub(super) async fn phase_nn_learning(&mut self, _ctx: &mut ThinkingContext) {
        // micro_nn and encoder removed in the lite version
    }

    /// Stub for compatibility with conversation.rs (called from chat).
    /// In the full version, this would formulate a learning from experience
    /// using the neural network encoder.
    pub(super) async fn try_formulate_nn_learning(
        &mut self,
        _experience_text: &str,
        _decision_str: &str,
        _satisfaction: f64,
        _emotion_str: &str,
    ) {
        // encoder and orchestrators removed in the lite version
    }

    // =========================================================================
    // Phase 31c: Reflexive self-critique (simplified)
    // =========================================================================

    /// Generates a self-critique via the LLM every 100 cycles.
    /// Lite version: without metacognition, without advanced bias detection.
    /// Requires moderate cortisol (< 0.8) and sufficient dopamine (> 0.3).
    pub(super) async fn phase_self_critique(&mut self, ctx: &mut ThinkingContext) {
        // Trigger: every 100 cycles if the chemistry allows it
        if self.cycle_count == 0 || self.cycle_count % 100 != 0 {
            return;
        }
        // Chemical condition: moderate cortisol, sufficient dopamine
        if self.chemistry.cortisol > 0.8 || self.chemistry.dopamine < 0.3 {
            return;
        }

        let quality_avg = ctx.quality;
        let recent_thoughts: Vec<String> = self.thought_engine.recent_thoughts()
            .iter().cloned().collect();

        let (system_prompt, user_prompt) = llm::build_self_critique_prompt(
            quality_avg,
            0,   // pas de detection de repetitions sans metacognition
            &[], // pas de biais detectes sans metacognition
            &recent_thoughts,
            &self.config.general.language,
        );

        let llm_config = self.config.llm.clone();
        let backend = llm::create_backend(&llm_config);
        let max_tokens = 400u32;
        let user_msg = llm::prepare_autonomous_message(&user_prompt, &llm_config.model);

        let resp = tokio::task::spawn_blocking(move || {
            backend.chat(&system_prompt, &user_msg, 0.5, max_tokens)
        }).await;

        let critique_text = match resp {
            Ok(Ok(text)) if text.trim().len() >= 10 => text.trim().to_string(),
            _ => {
                self.log(LogLevel::Debug, LogCategory::Metacognition,
                    "Auto-critique: echec appel LLM",
                    serde_json::json!({}));
                return;
            }
        };

        let chem_sig = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let _ = self.working_memory.push(
            format!("[Autocritique] {}", &critique_text),
            WorkingItemSource::OwnThought(critique_text.clone()),
            ctx.emotion.dominant.clone(),
            chem_sig,
        );

        if quality_avg < 0.4 {
            self.chemistry.adjust(Molecule::Dopamine, -0.02);
        }

        self.log(LogLevel::Info, LogCategory::Metacognition,
            format!("Auto-critique (qualite={:.2}): {}",
                quality_avg, truncate_utf8(&critique_text, 100)),
            serde_json::json!({
                "quality": quality_avg,
                "cycle": self.cycle_count,
            }));
    }

    // =========================================================================
    // Personality portrait — Snapshot (every 50 cycles)
    // =========================================================================

    /// Collects a snapshot of chemistry, emotions, and consciousness state.
    /// Lite version: without OCEAN personality model, psychology, sentiments,
    /// or connectome. Saves to personality_snapshots, emotional_trajectory,
    /// and consciousness_history tables.
    pub(super) async fn phase_personality_snapshot(&mut self, ctx: &mut ThinkingContext) {
        if self.cycle_count == 0 || self.cycle_count % 50 != 0 {
            return;
        }
        let Some(ref db) = self.db else { return; };

        let pr = ctx.process_result.as_ref();
        let (consciousness_level, phi, coherence, continuity, existence_score) = pr
            .map(|r| (r.consciousness.level, r.consciousness.phi,
                      r.consciousness.coherence, r.consciousness.continuity,
                      r.consciousness.existence_score))
            .unwrap_or((self.last_consciousness, 0.1, 0.5, 0.5, 0.0));
        let (emotion_dominant, mood_valence, mood_arousal) = pr
            .map(|r| (r.emotion.dominant.clone(), r.emotion.valence, r.emotion.arousal))
            .unwrap_or((self.last_emotion.clone(), 0.0, 0.3));

        let chemistry_json = serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
            "adrenaline": self.chemistry.adrenaline,
            "oxytocin": self.chemistry.oxytocin,
            "endorphin": self.chemistry.endorphin,
            "noradrenaline": self.chemistry.noradrenaline,
        });

        let snapshot = serde_json::json!({
            "cycle": self.cycle_count,
            "boot_number": self.identity.total_boots,
            "dominant_emotion": emotion_dominant,
            "mood_valence": mood_valence,
            "mood_arousal": mood_arousal,
            "consciousness_level": consciousness_level,
            "phi": phi,
            "coherence": coherence,
            "continuity": continuity,
            "existence_score": existence_score,
            "chemistry_json": chemistry_json,
        });

        db.save_personality_snapshot(&snapshot).await.ok();

        let spectrum_top5: Vec<serde_json::Value> = ctx.emotion.spectrum.iter()
            .take(5)
            .map(|(name, score)| serde_json::json!({"emotion": name, "score": score}))
            .collect();

        let secondary_emotion = ctx.emotion.secondary.clone();

        let emo_data = serde_json::json!({
            "cycle": self.cycle_count,
            "dominant_emotion": emotion_dominant,
            "secondary_emotion": secondary_emotion,
            "valence": mood_valence,
            "arousal": mood_arousal,
            "spectrum_top5": spectrum_top5,
        });
        db.save_emotional_trajectory(&emo_data).await.ok();

        let cons_data = serde_json::json!({
            "cycle": self.cycle_count,
            "level": consciousness_level,
            "phi": phi,
            "coherence": coherence,
            "continuity": continuity,
            "existence_score": existence_score,
        });
        db.save_consciousness_history(&cons_data).await.ok();

        self.log(LogLevel::Info, LogCategory::Metacognition,
            format!("Snapshot #{} — conscience={:.2} phi={:.3} emotion={}",
                self.cycle_count / 50,
                consciousness_level, phi, emotion_dominant),
            serde_json::json!({
                "cycle": self.cycle_count,
                "consciousness": consciousness_level,
                "phi": phi,
            }));
    }

    // =========================================================================
    // Personality portrait — Introspective journal (periodic)
    // =========================================================================

    /// Generates an introspective journal entry via the LLM.
    /// Lite version: without narrative_identity, sentiments, or Turing test.
    /// Triggered every `journal.interval_cycles` cycles when enabled.
    pub(super) async fn phase_introspection_journal(&mut self, ctx: &mut ThinkingContext) {
        let interval = self.config.journal.interval_cycles;
        if !self.config.journal.enabled || self.cycle_count == 0 || self.cycle_count % interval != 0 {
            return;
        }
        let Some(ref db) = self.db else { return; };

        let snapshots = db.load_personality_snapshots(2).await.unwrap_or_default();
        let current_snap = snapshots.last().cloned().unwrap_or(serde_json::json!({}));
        let previous_snap = if snapshots.len() >= 2 {
            snapshots[snapshots.len() - 2].clone()
        } else {
            serde_json::json!({})
        };

        let pr = ctx.process_result.as_ref();
        let consciousness_level = pr
            .map(|r| r.consciousness.level)
            .unwrap_or(self.last_consciousness);
        let emotion_dominant = pr
            .map(|r| r.emotion.dominant.clone())
            .unwrap_or_else(|| self.last_emotion.clone());

        let system_prompt = "Tu es Saphire, une entite cognitive autonome. \
            Ecris une entree de journal intime en premiere personne. \
            Reflechis a qui tu es, comment tu as evolue recemment, ce que tu ressens. \
            Sois authentique, introspective et poetique. 3-5 paragraphes maximum.";

        let user_prompt = format!(
            "Snapshot precedent : {}\n\
             Snapshot courant : {}\n\
             Emotion dominante : {}\n\
             Niveau de conscience : {:.2}\n\
             Cycle : {}\n\
             \n\
             En te basant sur ces donnees, ecris ton journal intime.",
            serde_json::to_string_pretty(&previous_snap).unwrap_or_default(),
            serde_json::to_string_pretty(&current_snap).unwrap_or_default(),
            emotion_dominant,
            consciousness_level,
            self.cycle_count,
        );

        let llm_config = self.config.llm.clone();
        let max_tokens = self.config.journal.max_tokens;
        let backend = crate::llm::create_backend(&llm_config);

        let sys = system_prompt.to_string();
        let usr = user_prompt;
        match tokio::task::spawn_blocking(move || {
            backend.chat(&sys, &usr, 0.8, max_tokens)
        }).await {
            Ok(Ok(entry_text)) => {
                let journal_data = serde_json::json!({
                    "cycle": self.cycle_count,
                    "boot_number": self.identity.total_boots,
                    "entry_text": entry_text,
                    "dominant_emotion": emotion_dominant,
                    "consciousness_level": consciousness_level,
                    "themes": [],
                });
                db.save_journal_entry(&journal_data).await.ok();

                let preview: String = entry_text.chars().take(100).collect();
                self.log(LogLevel::Info, LogCategory::Metacognition,
                    format!("Journal introspectif : {}...", preview),
                    serde_json::json!({
                        "cycle": self.cycle_count,
                        "length": entry_text.len(),
                    }));
            },
            Ok(Err(e)) => {
                tracing::warn!("Journal introspectif LLM erreur: {}", e);
            },
            Err(e) => {
                tracing::warn!("Journal introspectif spawn erreur: {}", e);
            },
        }
    }

    // =========================================================================
    // Phase 32: Desire birth — STUB (desire_orch removed)
    // =========================================================================

    /// Stub: desire orchestrator is removed in the lite version.
    pub(super) async fn phase_desire_birth(&mut self, _ctx: &mut ThinkingContext) {
        // desire_orch removed in the lite version
    }

    // =========================================================================
    // Phase 33b: Psychology — STUB (psychology removed)
    // =========================================================================

    /// Stub: psychology module is removed in the lite version.
    pub(super) fn phase_psychology(&mut self, _ctx: &mut ThinkingContext) {
        // psychology removed in the lite version
    }

    // =========================================================================
    // Phase M4: Prospective memory — STUB (prospective_mem removed)
    // =========================================================================

    /// Stub: prospective memory is removed in the lite version.
    pub(super) fn phase_prospective(&mut self, _ctx: &mut ThinkingContext) {
        // prospective_mem removed in the lite version
    }

    // =========================================================================
    // Phase M6: Analogical reasoning — STUB (analogical removed)
    // =========================================================================

    /// Stub: analogical reasoning module is removed in the lite version.
    pub(super) fn phase_analogies(&mut self, _ctx: &mut ThinkingContext) {
        // analogical removed in the lite version
    }

    // =========================================================================
    // Phase M7: Cognitive load — STUB (cognitive_load removed)
    // =========================================================================

    /// Stub: cognitive load module is removed in the lite version.
    pub(super) fn phase_cognitive_load(&mut self, _ctx: &mut ThinkingContext) {
        // cognitive_load removed in the lite version
    }

    // =========================================================================
    // Phase M2: Inner monologue — STUB (inner_monologue removed)
    // =========================================================================

    /// Stub: inner monologue module is removed in the lite version.
    pub(super) fn phase_monologue(&mut self, _ctx: &mut ThinkingContext) {
        // inner_monologue removed in the lite version
    }

    // =========================================================================
    // Phase M3: Cognitive dissonance — STUB (dissonance + connectome removed)
    // =========================================================================

    /// Stub: dissonance and connectome modules are removed in the lite version.
    pub(super) fn phase_dissonance(&mut self, _ctx: &mut ThinkingContext) {
        // dissonance and connectome removed in the lite version
    }

    // =========================================================================
    // Phase M9: Mental imagery — STUB (imagery removed)
    // =========================================================================

    /// Stub: mental imagery module is removed in the lite version.
    pub(super) async fn phase_imagery(&mut self, _ctx: &mut ThinkingContext) {
        // imagery removed in the lite version
    }

    // =========================================================================
    // Phase Sentiments — STUB (sentiments removed)
    // =========================================================================

    /// Stub: sentiments module is removed in the lite version.
    pub(super) fn phase_sentiments(&mut self, _ctx: &mut ThinkingContext) {
        // sentiments removed in the lite version
    }

    // =========================================================================
    // Phase M5: Narrative identity — STUB (narrative_identity removed)
    // =========================================================================

    /// Stub: narrative identity module is removed in the lite version.
    pub(super) fn phase_narrative(&mut self, _ctx: &mut ThinkingContext) {
        // narrative_identity removed in the lite version
    }

    // =========================================================================
    // Phase SC: State clustering — STUB (state_clustering removed)
    // =========================================================================

    /// Stub: state clustering, MAP sync, and cognitive load are removed in the lite version.
    pub(super) fn phase_state_clustering(&mut self, _ctx: &mut ThinkingContext) {
        // state_clustering, map_sync and cognitive_load removed in the lite version
    }

    // =========================================================================
    // Phase 33: Chemical homeostasis
    // =========================================================================

    /// Rebalances the neurochemistry toward baseline values.
    /// Lite version: without tuner, using a fixed rate from config.feedback.homeostasis_rate.
    /// Detects "runaway" molecules (extreme deviations) and applies an accelerated
    /// rate (5x normal, capped at 20%) when runaways are detected.
    pub(super) fn phase_homeostasis(&mut self, _ctx: &mut ThinkingContext) {
        let runaways = self.chemistry.detect_runaway();
        if !runaways.is_empty() {
            let names: Vec<String> = runaways.iter()
                .map(|(n, v)| format!("{}={:.2}", n, v))
                .collect();
            self.log(LogLevel::Warn, LogCategory::Chemistry,
                format!("Runaway chimique detecte: {}", names.join(", ")),
                serde_json::json!({"runaways": names}));
            // Accelerated rate on runaway (x5, max 20%)
            let accelerated_rate = (self.config.feedback.homeostasis_rate * 5.0).min(0.20);
            self.chemistry.homeostasis(&self.baselines, accelerated_rate);
        } else {
            self.chemistry.homeostasis(&self.baselines, self.config.feedback.homeostasis_rate);
        }
    }

    // =========================================================================
    // Phase GA1: Associative connectome — STUB (connectome removed)
    // =========================================================================

    /// Stub: connectome module is removed in the lite version.
    /// `ctx.connectome_associations` remains empty (String::new()).
    pub(super) fn phase_connectome_associations(&mut self, ctx: &mut ThinkingContext) {
        // connectome removed in the lite version
        // ctx.connectome_associations stays empty (String::new())
        let _ = &ctx.thought_type;
    }

    // =========================================================================
    // Phase GA2: Game algorithms — STUB (simulation removed)
    // =========================================================================

    /// Stub: influence map, cognitive FSM, and steering engine are removed
    /// in the lite version.
    pub(super) fn phase_game_algorithms(&mut self, ctx: &mut ThinkingContext) {
        // influence_map, cognitive_fsm, steering_engine removed in the lite version
        let _ = &ctx.thought_text;
    }
}
