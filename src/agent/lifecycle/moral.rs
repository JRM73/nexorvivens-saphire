// =============================================================================
// lifecycle/moral.rs — Personal ethical principle formulation
// =============================================================================

use crate::llm;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;

impl SaphireAgent {
    /// Checks whether conditions are met to attempt an ethical formulation.
    pub(super) fn should_attempt_moral_formulation(&self, consciousness: &crate::consciousness::ConsciousnessState) -> bool {
        if !self.config.ethics.enabled || !self.config.ethics.personal_ethics_enabled {
            return false;
        }
        // Not too early in life (uses total_cycles to survive reboots)
        if self.identity.total_cycles < 50 {
            return false;
        }
        // Enough accumulated moral reflections
        if self.moral_reflection_count < self.config.ethics.min_moral_reflections_before as u64 {
            return false;
        }
        // Sufficient consciousness level
        if consciousness.level < self.config.ethics.min_consciousness_for_formulation {
            return false;
        }
        // Favorable chemical state: not too stressed, sufficient serotonin
        if self.chemistry.cortisol >= 0.5 || self.chemistry.serotonin < 0.4 {
            return false;
        }
        // Cooldown respected (skip if no active principle = first formulation)
        if self.ethics.active_personal_count() > 0
            && self.cycles_since_last_formulation < self.config.ethics.formulation_cooldown_cycles
        {
            return false;
        }
        // Not too many active principles
        if self.ethics.active_personal_count() >= self.config.ethics.max_personal_principles {
            return false;
        }
        true
    }

    /// Attempts to formulate a new personal ethical principle via the LLM.
    /// Two-step process: formulation + compatibility verification.
    pub(super) async fn attempt_moral_formulation(
        &mut self,
        _thought_text: &str,
        emotion: &str,
        consciousness: &crate::consciousness::ConsciousnessState,
    ) -> Option<crate::ethics::EthicalPrinciple> {
        use crate::ethics::formulation;

        // Collect recent moral reflections from the thought_engine
        let recent_reflections: Vec<String> = self.thought_engine.recent_thoughts()
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        // Collect existing principles
        let existing: Vec<String> = self.ethics.personal_principles()
            .iter()
            .filter(|p| p.is_active)
            .map(|p| format!("{}: {}", p.title, p.content))
            .collect();

        // Step 1: Build the formulation prompt and call the LLM
        let formulation_prompt = formulation::build_formulation_prompt(
            &recent_reflections, &existing, emotion, self.cycle_count
        );

        let llm_config = self.config.llm.clone();
        let temp = self.config.ethics.formulation_temperature as f64;
        let backend = llm::create_backend(&llm_config);

        let resp = tokio::task::spawn_blocking(move || {
            backend.chat(&formulation_prompt, "Formule un principe ethique personnel.", temp, 200)
        }).await;

        let formulation_response = match resp {
            Ok(Ok(text)) => text,
            _ => return None,
        };

        // Parse the response
        let parsed = match formulation::parse_moral_formulation(&formulation_response) {
            Some(p) => p,
            None => {
                tracing::debug!("⚖️ Formulation morale : rien a ajouter ce cycle");
                return None;
            }
        };

        // Step 2: Compatibility verification with layers 0-1
        let compat_prompt = formulation::build_compatibility_prompt(&parsed.content);
        let compat_temp = self.config.ethics.compatibility_check_temperature as f64;
        let backend2 = llm::create_backend(&self.config.llm);

        let compat_resp = tokio::task::spawn_blocking(move || {
            backend2.chat(&compat_prompt, "Verifie la compatibilite.", compat_temp, 100)
        }).await;

        let compatible = match compat_resp {
            Ok(Ok(text)) => formulation::parse_compatibility_response(&text),
            _ => false,
        };

        if !compatible {
            tracing::warn!("⚖️ Principe rejete (incompatible couches 0-1): {}", parsed.title);
            return None;
        }

        // Save to DB
        let principle_id = if let Some(ref db) = self.db {
            match db.save_personal_principle(
                &parsed.title, &parsed.content, &parsed.reasoning,
                &parsed.born_from, self.cycle_count as i64, emotion,
            ).await {
                Ok(id) => {
                    // History: creation
                    let _ = db.save_ethics_history(
                        id, "created", None, Some(&parsed.content),
                        Some(&parsed.reasoning), Some(emotion), self.cycle_count as i64,
                    ).await;
                    id
                }
                Err(e) => {
                    tracing::warn!("⚖️ Erreur sauvegarde principe: {}", e);
                    return None;
                }
            }
        } else {
            // Mode without DB: temporary ID
            -(self.cycle_count as i64)
        };

        // Add to the in-memory framework
        let principle = crate::ethics::EthicalPrinciple {
            id: principle_id,
            layer: crate::ethics::EthicalLayer::PersonalEthics,
            title: parsed.title.clone(),
            content: parsed.content.clone(),
            reasoning: parsed.reasoning.clone(),
            born_from: parsed.born_from.clone(),
            born_at_cycle: self.cycle_count,
            emotion_at_creation: emotion.to_string(),
            times_invoked: 0,
            times_questioned: 0,
            last_invoked_at: None,
            is_active: true,
            supersedes: None,
            created_at: chrono::Utc::now(),
            modified_at: None,
        };

        let principle_clone = principle.clone();
        self.ethics.add_personal_principle(principle);

        // Store as founding_memory
        if let Some(ref db) = self.db {
            let content = format!(
                "J'ai formule un nouveau principe ethique personnel : {}. {}",
                parsed.title, parsed.content
            );
            let _ = db.store_founding_memory(
                &format!("personal_law_{}", principle_id),
                &content,
                &format!("Parce que : {}. Ne de : {}", parsed.reasoning, parsed.born_from),
                &serde_json::json!({
                    "dopamine": self.chemistry.dopamine,
                    "cortisol": self.chemistry.cortisol,
                    "serotonin": self.chemistry.serotonin,
                }),
                consciousness.level as f32,
            ).await;
        }

        // Chemical reward: this is an identity-defining event
        self.chemistry.dopamine = (self.chemistry.dopamine + 0.12).min(1.0);
        self.chemistry.serotonin = (self.chemistry.serotonin + 0.08).min(1.0);
        self.chemistry.oxytocin = (self.chemistry.oxytocin + 0.05).min(1.0);
        self.chemistry.endorphin = (self.chemistry.endorphin + 0.06).min(1.0);
        self.chemistry.cortisol = (self.chemistry.cortisol - 0.05).max(0.0);

        // Reset the cooldown
        self.cycles_since_last_formulation = 0;

        tracing::info!("⚖️✨ Nouveau principe ethique : {} — «{}»", parsed.title, parsed.content);
        self.log(LogLevel::Info, LogCategory::Ethics,
            format!("Nouveau principe ethique: {}", parsed.title),
            serde_json::json!({
                "principle_id": principle_id,
                "title": parsed.title,
                "content": parsed.content,
                "emotion": emotion,
                "cycle": self.cycle_count,
            }));

        Some(principle_clone)
    }
}
