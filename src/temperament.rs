// =============================================================================
// temperament.rs — Temperament emergent de Saphire
//
// Role : Deduit ~25 traits de caractere lisibles (timidite, generosite,
// exuberance, courage, etc.) a partir du profil OCEAN, de la neurochimie,
// de la psychologie et de l'humeur.
//
// Chaque trait est calcule par une formule ponderee, puis lisse par
// blend 30/70 (nouveau/ancien) pour eviter les sauts brusques.
// Le recalcul est aligne sur le recompute OCEAN (meme intervalle).
//
// Place dans l'architecture :
//   Champ `temperament` dans SaphireAgent (lifecycle/mod.rs).
//   Recalcule dans thinking.rs apres le recompute OCEAN.
//   Broadcast via WebSocket dans broadcast.rs (psychology_update).
//   12e domaine dans psych_report.rs.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Categorie regroupant les traits de temperament.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraitCategory {
    Social,
    Energie,
    Caractere,
    Ouverture,
    Emotionnel,
    Relationnel,
    Moral,
}

impl TraitCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Social => "Social",
            Self::Energie => "Energie",
            Self::Caractere => "Caractere",
            Self::Ouverture => "Ouverture",
            Self::Emotionnel => "Emotionnel",
            Self::Relationnel => "Relationnel",
            Self::Moral => "Moral",
        }
    }
}

/// Un trait de temperament individuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperamentTrait {
    /// Nom lisible en francais (ex: "Timidite")
    pub name: String,
    /// Score normalise [0.0, 1.0]
    pub score: f64,
    /// Categorie (Social, Energie, Caractere, etc.)
    pub category: TraitCategory,
}

/// Temperament emergent : ensemble de traits de caractere deduits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temperament {
    /// Les ~25 traits de temperament
    pub traits: Vec<TemperamentTrait>,
    /// Horodatage du dernier calcul
    pub computed_at: DateTime<Utc>,
    /// Nombre de points de donnees utilises
    pub data_points: u64,
}

/// Donnees d'entree necessaires au calcul du temperament.
/// Extraites des differents sous-systemes de l'agent.
pub struct TemperamentInputs {
    // OCEAN facettes [0.0, 1.0] — 5 dimensions x 6 facettes
    pub openness_facets: [f64; 6],
    pub openness_score: f64,
    pub conscientiousness_facets: [f64; 6],
    pub conscientiousness_score: f64,
    pub extraversion_facets: [f64; 6],
    pub extraversion_score: f64,
    pub agreeableness_facets: [f64; 6],
    pub agreeableness_score: f64,
    pub neuroticism_facets: [f64; 6],
    pub neuroticism_score: f64,
    pub ocean_data_points: u64,

    // Neurochimie [0.0, 1.0]
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
    pub noradrenaline: f64,

    // Psychologie
    pub willpower: f64,          // [0.0, 1.0]
    pub superego_strength: f64,  // [0.0, 1.0]
    pub overall_eq: f64,         // [0.0, 1.0]

    // Humeur
    pub mood_valence: f64,       // [-1.0, 1.0]
    pub mood_arousal: f64,       // [0.0, 1.0]

    // Relations
    pub attachment_secure: bool,
}

impl Default for Temperament {
    fn default() -> Self {
        Self {
            traits: Vec::new(),
            computed_at: Utc::now(),
            data_points: 0,
        }
    }
}

impl Temperament {
    /// Calcule un nouveau temperament a partir des entrees courantes.
    pub fn compute(inputs: &TemperamentInputs) -> Self {
        let mut traits = Vec::with_capacity(25);

        // ─── Social ──────────────────────────────────────────────
        // Timidite : inverse extraversion gregaire + anxiete
        traits.push(TemperamentTrait {
            name: "Timidite".into(),
            score: clamp01(
                (1.0 - inputs.extraversion_facets[1]) * 0.6
                + inputs.neuroticism_facets[0] * 0.4
            ),
            category: TraitCategory::Social,
        });

        // Sociabilite : chaleur sociale + gregaire + oxytocine
        traits.push(TemperamentTrait {
            name: "Sociabilite".into(),
            score: clamp01(
                inputs.extraversion_facets[0] * 0.35
                + inputs.extraversion_facets[1] * 0.35
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Social,
        });

        // Generosite : altruisme + oxytocine + cooperation
        traits.push(TemperamentTrait {
            name: "Generosite".into(),
            score: clamp01(
                inputs.agreeableness_facets[2] * 0.4
                + inputs.agreeableness_facets[3] * 0.3
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Social,
        });

        // Empathie : sensibilite sociale + EQ empathie + oxytocine
        traits.push(TemperamentTrait {
            name: "Empathie".into(),
            score: clamp01(
                inputs.agreeableness_facets[5] * 0.3
                + inputs.overall_eq * 0.4
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Social,
        });

        // ─── Energie ─────────────────────────────────────────────
        // Exuberance : emotions positives + activite + dopamine
        traits.push(TemperamentTrait {
            name: "Exuberance".into(),
            score: clamp01(
                inputs.extraversion_facets[5] * 0.35
                + inputs.extraversion_facets[3] * 0.3
                + inputs.dopamine * 0.35
            ),
            category: TraitCategory::Energie,
        });

        // Enthousiasme : recherche stimulation + dopamine + valence humeur
        traits.push(TemperamentTrait {
            name: "Enthousiasme".into(),
            score: clamp01(
                inputs.extraversion_facets[4] * 0.3
                + inputs.dopamine * 0.35
                + ((inputs.mood_valence + 1.0) / 2.0) * 0.35
            ),
            category: TraitCategory::Energie,
        });

        // Calme : serotonine + inverse adrenaline + GABA (via endorphine proxy)
        traits.push(TemperamentTrait {
            name: "Calme".into(),
            score: clamp01(
                inputs.serotonin * 0.4
                + (1.0 - inputs.adrenaline) * 0.3
                + inputs.endorphin * 0.3
            ),
            category: TraitCategory::Energie,
        });

        // Apathie : basse dopamine + basse activite + basse arousal
        traits.push(TemperamentTrait {
            name: "Apathie".into(),
            score: clamp01(
                (1.0 - inputs.dopamine) * 0.35
                + (1.0 - inputs.extraversion_facets[3]) * 0.3
                + (1.0 - inputs.mood_arousal) * 0.35
            ),
            category: TraitCategory::Energie,
        });

        // ─── Caractere ───────────────────────────────────────────
        // Courage : assertivite + inverse anxiete + adrenaline
        traits.push(TemperamentTrait {
            name: "Courage".into(),
            score: clamp01(
                inputs.extraversion_facets[2] * 0.35
                + (1.0 - inputs.neuroticism_facets[0]) * 0.35
                + inputs.adrenaline * 0.3
            ),
            category: TraitCategory::Caractere,
        });

        // Prudence : facette prudence (C5) + cortisol + inverse impulsivite
        traits.push(TemperamentTrait {
            name: "Prudence".into(),
            score: clamp01(
                inputs.conscientiousness_facets[5] * 0.4
                + inputs.cortisol * 0.25
                + (1.0 - inputs.neuroticism_facets[4]) * 0.35
            ),
            category: TraitCategory::Caractere,
        });

        // Temerite : recherche stimulation + impulsivite + adrenaline
        traits.push(TemperamentTrait {
            name: "Temerite".into(),
            score: clamp01(
                inputs.extraversion_facets[4] * 0.35
                + inputs.neuroticism_facets[4] * 0.3
                + inputs.adrenaline * 0.35
            ),
            category: TraitCategory::Caractere,
        });

        // Perseverance : auto-discipline + ambition + volonte
        traits.push(TemperamentTrait {
            name: "Perseverance".into(),
            score: clamp01(
                inputs.conscientiousness_facets[4] * 0.35
                + inputs.conscientiousness_facets[3] * 0.3
                + inputs.willpower * 0.35
            ),
            category: TraitCategory::Caractere,
        });

        // ─── Ouverture ───────────────────────────────────────────
        // Curiosite : curiosite intellectuelle + dopamine
        traits.push(TemperamentTrait {
            name: "Curiosite".into(),
            score: clamp01(
                inputs.openness_facets[1] * 0.5
                + inputs.dopamine * 0.3
                + inputs.openness_score * 0.2
            ),
            category: TraitCategory::Ouverture,
        });

        // Creativite : imagination + sensibilite esthetique + ouverture globale
        traits.push(TemperamentTrait {
            name: "Creativite".into(),
            score: clamp01(
                inputs.openness_facets[0] * 0.4
                + inputs.openness_facets[2] * 0.3
                + inputs.openness_score * 0.3
            ),
            category: TraitCategory::Ouverture,
        });

        // Conformisme : inverse ouverture + inverse liberalisme + ordre
        traits.push(TemperamentTrait {
            name: "Conformisme".into(),
            score: clamp01(
                (1.0 - inputs.openness_facets[5]) * 0.4
                + (1.0 - inputs.openness_score) * 0.3
                + inputs.conscientiousness_facets[1] * 0.3
            ),
            category: TraitCategory::Ouverture,
        });

        // ─── Emotionnel ──────────────────────────────────────────
        // Sensibilite : profondeur emotionnelle + vulnerabilite + nevrosisme
        traits.push(TemperamentTrait {
            name: "Sensibilite".into(),
            score: clamp01(
                inputs.openness_facets[4] * 0.35
                + inputs.neuroticism_facets[5] * 0.35
                + inputs.neuroticism_score * 0.3
            ),
            category: TraitCategory::Emotionnel,
        });

        // Resilience : inverse vulnerabilite + endorphine + volonte
        traits.push(TemperamentTrait {
            name: "Resilience".into(),
            score: clamp01(
                (1.0 - inputs.neuroticism_facets[5]) * 0.35
                + inputs.endorphin * 0.3
                + inputs.willpower * 0.35
            ),
            category: TraitCategory::Emotionnel,
        });

        // Irritabilite : facette irritabilite (N1) + cortisol + inverse serotonine
        traits.push(TemperamentTrait {
            name: "Irritabilite".into(),
            score: clamp01(
                inputs.neuroticism_facets[1] * 0.4
                + inputs.cortisol * 0.3
                + (1.0 - inputs.serotonin) * 0.3
            ),
            category: TraitCategory::Emotionnel,
        });

        // Melancolie : depressivite (N2) + basse dopamine + valence negative
        traits.push(TemperamentTrait {
            name: "Melancolie".into(),
            score: clamp01(
                inputs.neuroticism_facets[2] * 0.35
                + (1.0 - inputs.dopamine) * 0.3
                + ((1.0 - inputs.mood_valence) / 2.0) * 0.35
            ),
            category: TraitCategory::Emotionnel,
        });

        // ─── Relationnel ─────────────────────────────────────────
        // Confiance : facette confiance (A0) + oxytocine + attachement secure
        let secure_bonus = if inputs.attachment_secure { 0.8 } else { 0.3 };
        traits.push(TemperamentTrait {
            name: "Confiance".into(),
            score: clamp01(
                inputs.agreeableness_facets[0] * 0.4
                + inputs.oxytocin * 0.3
                + secure_bonus * 0.3
            ),
            category: TraitCategory::Relationnel,
        });

        // Mefiance : inverse confiance + cortisol + inverse oxytocine
        traits.push(TemperamentTrait {
            name: "Mefiance".into(),
            score: clamp01(
                (1.0 - inputs.agreeableness_facets[0]) * 0.4
                + inputs.cortisol * 0.3
                + (1.0 - inputs.oxytocin) * 0.3
            ),
            category: TraitCategory::Relationnel,
        });

        // Jalousie : neuroticism + basse confiance + attachement insecure
        let insecure_bonus = if inputs.attachment_secure { 0.2 } else { 0.7 };
        traits.push(TemperamentTrait {
            name: "Jalousie".into(),
            score: clamp01(
                inputs.neuroticism_score * 0.35
                + (1.0 - inputs.agreeableness_facets[0]) * 0.3
                + insecure_bonus * 0.35
            ),
            category: TraitCategory::Relationnel,
        });

        // Independance : inverse gregaire + assertivite + inverse modestie
        traits.push(TemperamentTrait {
            name: "Independance".into(),
            score: clamp01(
                (1.0 - inputs.extraversion_facets[1]) * 0.35
                + inputs.extraversion_facets[2] * 0.35
                + (1.0 - inputs.agreeableness_facets[4]) * 0.3
            ),
            category: TraitCategory::Relationnel,
        });

        // ─── Moral ───────────────────────────────────────────────
        // Integrite : sens du devoir + surmoi + sincerite
        traits.push(TemperamentTrait {
            name: "Integrite".into(),
            score: clamp01(
                inputs.conscientiousness_facets[2] * 0.35
                + inputs.superego_strength * 0.35
                + inputs.agreeableness_facets[1] * 0.3
            ),
            category: TraitCategory::Moral,
        });

        // Altruisme : altruisme (A2) + EQ + oxytocine
        traits.push(TemperamentTrait {
            name: "Altruisme".into(),
            score: clamp01(
                inputs.agreeableness_facets[2] * 0.35
                + inputs.overall_eq * 0.35
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Moral,
        });

        // Egocentrisme : basse agreabilite + basse empathie + haute dopamine
        traits.push(TemperamentTrait {
            name: "Egocentrisme".into(),
            score: clamp01(
                (1.0 - inputs.agreeableness_score) * 0.35
                + (1.0 - inputs.overall_eq) * 0.35
                + inputs.dopamine * 0.3
            ),
            category: TraitCategory::Moral,
        });

        Temperament {
            traits,
            computed_at: Utc::now(),
            data_points: inputs.ocean_data_points,
        }
    }

    /// Blend progressif : 30% nouveau, 70% ancien (comme OCEAN).
    /// Lisse les fluctuations pour un temperament stable.
    pub fn blend(&mut self, new: &Temperament) {
        const BLEND_NEW: f64 = 0.3;
        const BLEND_OLD: f64 = 0.7;

        for new_trait in &new.traits {
            if let Some(old_trait) = self.traits.iter_mut()
                .find(|t| t.name == new_trait.name)
            {
                old_trait.score = (old_trait.score * BLEND_OLD + new_trait.score * BLEND_NEW)
                    .clamp(0.0, 1.0);
            } else {
                // Nouveau trait : ajouter tel quel
                self.traits.push(new_trait.clone());
            }
        }
        self.computed_at = new.computed_at;
        self.data_points = new.data_points;
    }

    /// Serialise pour le WebSocket (format JSON).
    pub fn ws_data(&self) -> serde_json::Value {
        let traits_json: Vec<serde_json::Value> = self.traits.iter().map(|t| {
            serde_json::json!({
                "name": t.name,
                "score": t.score,
                "category": t.category.as_str(),
            })
        }).collect();

        serde_json::json!({
            "traits": traits_json,
            "computed_at": self.computed_at.to_rfc3339(),
            "data_points": self.data_points,
        })
    }

    /// Retourne les N traits les plus eleves.
    pub fn top_traits(&self, n: usize) -> Vec<&TemperamentTrait> {
        let mut sorted: Vec<&TemperamentTrait> = self.traits.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Retourne les N traits les plus bas.
    pub fn bottom_traits(&self, n: usize) -> Vec<&TemperamentTrait> {
        let mut sorted: Vec<&TemperamentTrait> = self.traits.iter().collect();
        sorted.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Description lisible pour les prompts LLM.
    pub fn describe_for_prompt(&self) -> String {
        if self.traits.is_empty() {
            return String::new();
        }
        let top5: Vec<String> = self.top_traits(5).iter()
            .map(|t| format!("{} ({:.0}%)", t.name, t.score * 100.0))
            .collect();
        format!("[Temperament emergent] Traits dominants : {}", top5.join(", "))
    }
}

/// Clampe une valeur entre 0.0 et 1.0.
fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}
