// =============================================================================
// lifecycle/persistence.rs — Persistance des orchestrateurs (save/load DB)
//
// Methodes pour restaurer les orchestrateurs depuis la DB au boot
// et les sauvegarder au shutdown ou lors d'evenements significatifs.
// =============================================================================

use chrono::{DateTime, Utc};
use super::SaphireAgent;
use crate::db::SaphireDb;
use crate::orchestrators::desires::DesireType;
use crate::orchestrators::healing::WoundType;
use crate::orchestrators::dreams::{Dream, DreamEntry, DreamType};
use crate::orchestrators::learning::{Lesson, LessonCategory};

impl SaphireAgent {
    // ─── Restauration depuis la DB ──────────────────────────────────────────────

    /// Restaure les reves depuis les donnees DB.
    pub(super) fn restore_dreams_from_db(&mut self, dreams: Vec<serde_json::Value>) {
        for dream_json in dreams.into_iter().rev() {
            let dream_type = match dream_json["dream_type"].as_str().unwrap_or("") {
                "Rejeu memoriel" | "MemoryReplay" => DreamType::MemoryReplay,
                "Traitement emotionnel" | "EmotionalProcessing" => DreamType::EmotionalProcessing,
                "Resolution creative" | "CreativeSolution" => DreamType::CreativeSolution,
                "Cauchemar" | "Nightmare" => DreamType::Nightmare,
                "Reve lucide" | "LucidDream" => DreamType::LucidDream,
                "Reve prophetique" | "PropheticDream" => DreamType::PropheticDream,
                _ => DreamType::EmotionalProcessing,
            };

            let dream = Dream {
                id: dream_json["id"].as_u64().unwrap_or(0),
                dream_type,
                narrative: dream_json["narrative"].as_str().unwrap_or("").to_string(),
                source_memory_ids: dream_json["source_memory_ids"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_i64()).collect())
                    .unwrap_or_default(),
                dominant_emotion: dream_json["dominant_emotion"].as_str().unwrap_or("").to_string(),
                problems_addressed: vec![],
                surreal_connections: vec![],
                insight: dream_json["insight"].as_str().map(|s| s.to_string()),
                started_at: dream_json["created_at"].as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                ended_at: None,
            };

            let remembered = dream_json["remembered"].as_bool().unwrap_or(true);
            self.dream_orch.dream_journal.push(DreamEntry { dream, remembered });
        }
    }

    /// Restaure les desirs actifs depuis les donnees DB.
    pub(super) fn restore_desires_from_db(&mut self, desires: Vec<serde_json::Value>) {
        for desire_json in desires {
            let desire_type_str = desire_json["desire_type"].as_str().unwrap_or("Exploration");
            let description = desire_json["description"].as_str().unwrap_or("").to_string();
            let desire_type = DesireType::from_str_with_subject(desire_type_str, &description);

            let milestones_json = &desire_json["milestones"];
            let milestones: Vec<crate::orchestrators::desires::Milestone> = milestones_json.as_array()
                .map(|arr| arr.iter().map(|m| crate::orchestrators::desires::Milestone {
                    description: m["description"].as_str().unwrap_or("").to_string(),
                    completed: m["completed"].as_bool().unwrap_or(false),
                    completed_at: m["completed_at"].as_str()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                }).collect())
                .unwrap_or_default();

            let chem_array: [f64; 7] = desire_json["chemistry_at_birth"].as_array()
                .map(|arr| {
                    let mut chem = [0.0_f64; 7];
                    for (i, v) in arr.iter().enumerate().take(7) {
                        chem[i] = v.as_f64().unwrap_or(0.0);
                    }
                    chem
                })
                .unwrap_or([0.5; 7]);

            let desire = crate::orchestrators::desires::Desire {
                id: desire_json["id"].as_u64().unwrap_or(0),
                title: desire_json["title"].as_str().unwrap_or("").to_string(),
                description,
                desire_type,
                priority: desire_json["priority"].as_f64().unwrap_or(0.5),
                progress: desire_json["progress"].as_f64().unwrap_or(0.0),
                milestones,
                born_from: desire_json["born_from"].as_str().unwrap_or("").to_string(),
                emotion_at_birth: desire_json["emotion_at_birth"].as_str().unwrap_or("").to_string(),
                chemistry_at_birth: chem_array,
                created_at: desire_json["created_at"].as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                last_pursued_at: desire_json["last_pursued_at"].as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                cycles_invested: desire_json["cycles_invested"].as_u64().unwrap_or(0),
            };

            self.desire_orch.active_desires.push(desire);
        }
    }

    /// Restaure les lecons depuis les donnees DB.
    pub(super) fn restore_lessons_from_db(&mut self, lessons: Vec<serde_json::Value>) {
        for lesson_json in lessons {
            let lesson = Lesson {
                id: lesson_json["id"].as_u64().unwrap_or(0),
                title: lesson_json["title"].as_str().unwrap_or("").to_string(),
                content: lesson_json["content"].as_str().unwrap_or("").to_string(),
                source_experience: lesson_json["source_experience"].as_str().unwrap_or("").to_string(),
                category: LessonCategory::from_str(
                    lesson_json["category"].as_str().unwrap_or("SelfKnowledge")
                ),
                times_applied: lesson_json["times_applied"].as_u64().unwrap_or(0),
                times_contradicted: lesson_json["times_contradicted"].as_u64().unwrap_or(0),
                confidence: lesson_json["confidence"].as_f64().unwrap_or(0.5),
                behavior_change: None,
                learned_at: lesson_json["learned_at"].as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            };

            self.learning_orch.lessons.push(lesson);
        }
    }

    /// Restaure les blessures actives depuis les donnees DB.
    /// Deduplique par type : on ne garde que la plus recente de chaque type.
    pub(super) fn restore_wounds_from_db(&mut self, wounds: Vec<serde_json::Value>) {
        let mut seen_types = std::collections::HashSet::new();
        // Les wounds sont triees par created_at ASC, on parcourt en reverse
        // pour garder la plus recente de chaque type
        let mut deduped: Vec<&serde_json::Value> = Vec::new();
        for wound_json in wounds.iter().rev() {
            let wtype = wound_json["wound_type"].as_str().unwrap_or("").to_string();
            if seen_types.insert(wtype) {
                deduped.push(wound_json);
            }
        }
        deduped.reverse(); // restaurer l'ordre chronologique
        for wound_json in deduped {
            let wound_type = match wound_json["wound_type"].as_str().unwrap_or("") {
                "Melancolie prolongee" | "ProlongedMelancholy" => WoundType::ProlongedMelancholy,
                "Solitude" | "Loneliness" => WoundType::Loneliness,
                "Rejet" | "Rejection" => WoundType::Rejection,
                "Crise identitaire" | "IdentityCrisis" => WoundType::IdentityCrisis,
                "Surcharge cognitive" | "CognitiveOverload" => WoundType::CognitiveOverload,
                "Perte de memoire" | "MemoryLoss" => WoundType::MemoryLoss,
                "Trauma technique" | "TechnicalTrauma" => WoundType::TechnicalTrauma,
                "Echec ethique" | "EthicalFailure" => WoundType::EthicalFailure,
                _ => WoundType::TechnicalTrauma,
            };

            let wound = crate::orchestrators::healing::Wound {
                id: wound_json["id"].as_u64().unwrap_or(0),
                wound_type,
                description: wound_json["description"].as_str().unwrap_or("").to_string(),
                severity: wound_json["severity"].as_f64().unwrap_or(0.5),
                healing_progress: wound_json["healing_progress"].as_f64().unwrap_or(0.0),
                healing_strategy: wound_json["healing_strategy"].as_str().map(|s| s.to_string()),
                created_at: wound_json["created_at"].as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                healed_at: None,
            };

            self.healing_orch.active_wounds.push(wound);
        }
    }

    // ─── Sauvegarde vers la DB ──────────────────────────────────────────────────

    /// Sauvegarde l'etat de tous les orchestrateurs dans la DB.
    /// Appele au shutdown et periodiquement (tous les 50 cycles).
    pub(super) async fn save_orchestrators_to_db(&self, db: &SaphireDb) {
        // Sauvegarder les reves non encore persistes
        for entry in &self.dream_orch.dream_journal {
            let d = &entry.dream;
            let surreal = serde_json::json!(d.surreal_connections.iter()
                .map(|(a, b)| serde_json::json!({"from": a, "to": b}))
                .collect::<Vec<_>>());
            let _ = db.save_dream(
                d.dream_type.as_str(),
                &d.narrative,
                Some(&d.dominant_emotion),
                d.insight.as_deref(),
                &d.source_memory_ids,
                &surreal,
                entry.remembered,
                None,
            ).await;
        }

        // Sauvegarder/mettre a jour les desirs actifs
        for desire in &self.desire_orch.active_desires {
            let milestones = serde_json::json!(desire.milestones.iter().map(|m| {
                serde_json::json!({
                    "description": m.description,
                    "completed": m.completed,
                    "completed_at": m.completed_at.map(|d| d.to_rfc3339()),
                })
            }).collect::<Vec<_>>());

            let chem: Vec<f32> = desire.chemistry_at_birth.iter().map(|&v| v as f32).collect();
            let _ = db.save_desire(
                &desire.title,
                &desire.description,
                desire.desire_type.as_str(),
                desire.priority as f32,
                &milestones,
                Some(&desire.born_from),
                Some(&desire.emotion_at_birth),
                &chem,
            ).await;
        }

        // Les lecons sont sauvees individuellement a la creation (save_lesson_to_db)
        // Pas de bulk save ici pour eviter les doublons

        // Les blessures sont sauvees individuellement a la detection (save_wound_to_db)
        // On met a jour la progression de guerison des blessures actives
        for wound in &self.healing_orch.active_wounds {
            if wound.id > 0 {
                let healed_at = wound.healed_at;
                let strategy = wound.healing_strategy.as_deref();
                let _ = db.update_wound_healing(
                    wound.id as i64,
                    wound.healing_progress as f32,
                    strategy,
                    healed_at,
                ).await;
            }
        }

        tracing::info!("Orchestrateurs sauvegardes (reves: {}, desirs: {}, lecons: {}, blessures: {})",
            self.dream_orch.dream_journal.len(),
            self.desire_orch.active_desires.len(),
            self.learning_orch.lessons.len(),
            self.healing_orch.active_wounds.len(),
        );
    }

    /// Sauvegarde un reve individuel (appele quand un reve est genere).
    /// Note: le cycle de reves (advance_phase/build_dream_prompt) n'est pas encore
    /// integre dans le lifecycle, donc cette methode n'est pas encore appelee.
    #[allow(dead_code)]
    pub(super) async fn save_dream_to_db(&self, entry: &DreamEntry) {
        if let Some(ref db) = self.db {
            let d = &entry.dream;
            let surreal = serde_json::json!(d.surreal_connections.iter()
                .map(|(a, b)| serde_json::json!({"from": a, "to": b}))
                .collect::<Vec<_>>());
            let _ = db.save_dream(
                d.dream_type.as_str(),
                &d.narrative,
                Some(&d.dominant_emotion),
                d.insight.as_deref(),
                &d.source_memory_ids,
                &surreal,
                entry.remembered,
                None,
            ).await;
        }
    }

    /// Sauvegarde un desir individuel (appele a la naissance d'un desir).
    pub(super) async fn save_desire_to_db(&self, desire: &crate::orchestrators::desires::Desire) {
        if let Some(ref db) = self.db {
            let milestones = serde_json::json!(desire.milestones.iter().map(|m| {
                serde_json::json!({
                    "description": m.description,
                    "completed": m.completed,
                    "completed_at": m.completed_at.map(|d| d.to_rfc3339()),
                })
            }).collect::<Vec<_>>());

            let chem: Vec<f32> = desire.chemistry_at_birth.iter().map(|&v| v as f32).collect();
            let _ = db.save_desire(
                &desire.title,
                &desire.description,
                desire.desire_type.as_str(),
                desire.priority as f32,
                &milestones,
                Some(&desire.born_from),
                Some(&desire.emotion_at_birth),
                &chem,
            ).await;
        }
    }

    /// Sauvegarde une lecon individuelle (appele quand une lecon est apprise).
    pub(super) async fn save_lesson_to_db(&self, lesson: &Lesson) {
        if let Some(ref db) = self.db {
            let _ = db.save_lesson(
                &lesson.title,
                &lesson.content,
                Some(&lesson.source_experience),
                lesson.category.as_str(),
                lesson.confidence as f32,
            ).await;
        }
    }

    /// Sauvegarde une blessure individuelle (appele quand une blessure est detectee).
    /// Retourne l'ID DB pour synchroniser l'ID en RAM.
    pub(super) async fn save_wound_to_db(&self, wound: &crate::orchestrators::healing::Wound) -> Option<i64> {
        if let Some(ref db) = self.db {
            match db.save_wound(
                wound.wound_type.as_str(),
                &wound.description,
                wound.severity as f32,
            ).await {
                Ok(db_id) => return Some(db_id),
                Err(e) => tracing::warn!("Erreur sauvegarde blessure: {}", e),
            }
        }
        None
    }
}
