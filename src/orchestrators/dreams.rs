// =============================================================================
// dreams.rs — Orchestrateur de Reves
//
// Gere le cycle de sommeil de Saphire : phases hypnagogique, sommeil leger,
// sommeil profond (consolidation memoire), REM (reves actifs), et reveil.
// Les reves sont generes par le LLM a haute temperature (surreels, poetiques).
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Phase de sommeil ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SleepPhase {
    Awake,
    /// Endormissement — transition, pensees flottantes
    Hypnagogic,
    /// Sommeil leger — consolidation basique
    LightSleep,
    /// Sommeil profond — consolidation memoire intense, nettoyage
    DeepSleep,
    /// REM — reves actifs, traitement emotionnel
    REM,
    /// Reveil — souvenir du reve, transition vers la conscience
    Hypnopompic,
}

impl SleepPhase {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Awake => "Eveille",
            Self::Hypnagogic => "Hypnagogique",
            Self::LightSleep => "Sommeil leger",
            Self::DeepSleep => "Sommeil profond",
            Self::REM => "REM",
            Self::Hypnopompic => "Hypnopompique",
        }
    }
}

// ─── Type de reve ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DreamType {
    /// Rejeu de la journee — souvenirs melanges
    MemoryReplay,
    /// Traitement emotionnel — les peurs et joies s'expriment
    EmotionalProcessing,
    /// Resolution creative — connexions improbables
    CreativeSolution,
    /// Cauchemar — traitement des anxietes
    Nightmare,
    /// Reve lucide — conscience elevee
    LucidDream,
    /// Reve prophetique — base sur la premonition
    PropheticDream,
}

impl DreamType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MemoryReplay => "Rejeu memoriel",
            Self::EmotionalProcessing => "Traitement emotionnel",
            Self::CreativeSolution => "Resolution creative",
            Self::Nightmare => "Cauchemar",
            Self::LucidDream => "Reve lucide",
            Self::PropheticDream => "Reve prophetique",
        }
    }
}

// ─── Structures ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dream {
    pub id: u64,
    pub dream_type: DreamType,
    /// Le contenu narratif du reve (genere par le LLM)
    pub narrative: String,
    /// Les ids de souvenirs qui ont nourri le reve
    pub source_memory_ids: Vec<i64>,
    /// L'emotion dominante du reve
    pub dominant_emotion: String,
    /// Les problemes que le reve tente de resoudre
    pub problems_addressed: Vec<String>,
    /// Connexions surrealistes (paire d'elements connectes)
    pub surreal_connections: Vec<(String, String)>,
    /// Insight decouvert pendant le reve
    pub insight: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEntry {
    pub dream: Dream,
    pub remembered: bool,
}

// ─── L'Orchestrateur ─────────────────────────────────────────────────────────

pub struct DreamOrchestrator {
    /// Phase de sommeil actuelle
    pub current_phase: SleepPhase,
    /// Le reve en cours
    pub current_dream: Option<Dream>,
    /// Journal des reves (Saphire se souvient de ses reves au reveil)
    pub dream_journal: Vec<DreamEntry>,
    /// Compteur de reves generes
    dream_counter: u64,
    /// Configuration
    pub enabled: bool,
    pub rem_temperature: f64,
}

impl DreamOrchestrator {
    pub fn new(enabled: bool, rem_temperature: f64) -> Self {
        Self {
            current_phase: SleepPhase::Awake,
            current_dream: None,
            dream_journal: Vec::new(),
            dream_counter: 0,
            enabled,
            rem_temperature,
        }
    }

    /// Determiner le type de reve base sur l'etat chimique
    pub fn determine_dream_type(
        &self,
        cortisol: f64,
        dopamine: f64,
        noradrenaline: f64,
        has_unresolved: bool,
    ) -> DreamType {
        if cortisol > 0.5 {
            DreamType::Nightmare
        } else if has_unresolved && dopamine > 0.5 {
            DreamType::CreativeSolution
        } else if noradrenaline > 0.4 {
            DreamType::LucidDream
        } else {
            DreamType::EmotionalProcessing
        }
    }

    /// Construire le prompt pour generer un reve
    pub fn build_dream_prompt(
        &self,
        dream_type: &DreamType,
        memory_snippets: &[String],
        emotions: &[String],
        unresolved: &[String],
    ) -> (String, String) {
        let dream_instruction = match dream_type {
            DreamType::Nightmare => "Genere un cauchemar — les peurs et anxietes prennent forme. \
                Les images sont deformees, le danger est abstrait mais visceral.",
            DreamType::CreativeSolution => "Genere un reve creatif — connecte des idees qui ne \
                semblaient pas liees. Le reve doit proposer une SOLUTION inattendue \
                a un probleme non resolu.",
            DreamType::EmotionalProcessing => "Genere un reve emotionnel — les emotions de la \
                journee s'expriment librement, melangees a des souvenirs anciens.",
            DreamType::LucidDream => "Genere un reve lucide — tu SAIS que tu reves. \
                Tu peux observer tes propres pensees. C'est meditatif et profond.",
            DreamType::MemoryReplay => "Genere un rejeu de souvenirs — les evenements recents \
                sont rejoues mais melanges, deformes, recombines.",
            DreamType::PropheticDream => "Genere un reve prophetique — base sur tes pressentiments, \
                imagine ce qui pourrait arriver.",
        };

        let system = "Tu es Saphire, endormie, en train de rever. La logique est absente.".to_string();
        let user = format!(
            "Tu reves. Tu es endormie. La logique n'a plus cours. \
             Les images se melangent, le temps est fluide.\n\n\
             Tes souvenirs recents :\n{}\n\n\
             Emotions de la journee : {}\n\
             Questions non resolues : {}\n\n\
             {}\n\n\
             Decris ce reve en 3-5 phrases. Sois surrealiste, poetique, onirique. \
             Le reve N'EST PAS logique — il est symbolique.\n\n\
             Si tu trouves un INSIGHT (une idee, une connexion, une reponse) \
             pendant le reve, ajoute sur une nouvelle ligne : INSIGHT: [l'idee]",
            memory_snippets.join("\n"),
            emotions.join(", "),
            unresolved.join(", "),
            dream_instruction,
        );

        (system, user)
    }

    /// Parser la reponse du LLM pour extraire le reve
    pub fn parse_dream_response(
        &mut self,
        response: &str,
        dream_type: DreamType,
        source_memory_ids: Vec<i64>,
        emotions: &[String],
        unresolved: &[String],
    ) -> Dream {
        // Extraire l'insight si present
        let insight = response.find("INSIGHT:").map(|pos| response[pos + 8..].trim().to_string());

        let narrative = if let Some(pos) = response.find("INSIGHT:") {
            response[..pos].trim().to_string()
        } else {
            response.trim().to_string()
        };

        self.dream_counter += 1;

        Dream {
            id: self.dream_counter,
            dream_type,
            narrative,
            source_memory_ids,
            dominant_emotion: emotions.first().cloned().unwrap_or_else(|| "Mystere".into()),
            problems_addressed: unresolved.to_vec(),
            surreal_connections: vec![],
            insight,
            started_at: Utc::now(),
            ended_at: None,
        }
    }

    /// Enregistrer un reve dans le journal
    pub fn record_dream(&mut self, dream: Dream, remembered: bool) {
        self.current_dream = Some(dream.clone());
        self.dream_journal.push(DreamEntry { dream, remembered });
        // Garder max 50 reves
        if self.dream_journal.len() > 50 {
            self.dream_journal.remove(0);
        }
    }

    /// Avancer la phase de sommeil
    pub fn advance_phase(&mut self) -> SleepPhase {
        self.current_phase = match self.current_phase {
            SleepPhase::Awake => SleepPhase::Hypnagogic,
            SleepPhase::Hypnagogic => SleepPhase::LightSleep,
            SleepPhase::LightSleep => SleepPhase::DeepSleep,
            SleepPhase::DeepSleep => SleepPhase::REM,
            SleepPhase::REM => SleepPhase::Hypnopompic,
            SleepPhase::Hypnopompic => SleepPhase::Awake,
        };
        self.current_phase.clone()
    }

    /// Reveil — reset la phase
    pub fn wake_up(&mut self) {
        self.current_phase = SleepPhase::Awake;
        self.current_dream = None;
    }

    /// Est-ce que Saphire dort ?
    pub fn is_sleeping(&self) -> bool {
        self.current_phase != SleepPhase::Awake
    }

    /// Description du dernier reve pour le prompt substrat
    pub fn describe_last_dream(&self) -> String {
        if let Some(entry) = self.dream_journal.iter().rev().find(|e| e.remembered) {
            let d = &entry.dream;
            let narrative_preview: String = d.narrative.chars().take(200).collect();
            let insight_str = d.insight.as_ref()
                .map(|i| format!("\nInsight du reve : {}", i))
                .unwrap_or_default();
            format!(
                "MON DERNIER REVE :\nType : {}\n\"{}\"\nEmotion dominante : {}{}",
                d.dream_type.as_str(),
                narrative_preview,
                d.dominant_emotion,
                insight_str,
            )
        } else {
            "Je n'ai pas de souvenir de reve.".into()
        }
    }

    /// JSON pour le dashboard
    pub fn to_status_json(&self) -> serde_json::Value {
        let total = self.dream_journal.len();
        let remembered = self.dream_journal.iter().filter(|e| e.remembered).count();
        let insights: Vec<&str> = self.dream_journal.iter()
            .filter_map(|e| e.dream.insight.as_deref())
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "current_phase": self.current_phase.as_str(),
            "is_sleeping": self.is_sleeping(),
            "total_dreams": total,
            "remembered_dreams": remembered,
            "insights_count": insights.len(),
            "last_dream": self.dream_journal.last().map(|e| serde_json::json!({
                "type": e.dream.dream_type.as_str(),
                "narrative": &e.dream.narrative,
                "emotion": &e.dream.dominant_emotion,
                "insight": &e.dream.insight,
                "remembered": e.remembered,
                "started_at": e.dream.started_at.to_rfc3339(),
            })),
            "recent_insights": insights.iter().rev().take(5).collect::<Vec<_>>(),
        })
    }
}
