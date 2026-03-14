// =============================================================================
// lifecycle/psych_report.rs — Module de rapport neuropsychologique
//
// Role : Collecte un snapshot complet de l'etat psychologique de l'agent
// a un instant T, puis genere un rapport clinique via le LLM.
//
// Structure :
//   - PsychSnapshot : 12 domaines cliniques (JSON serialisable)
//   - PsychReport : rapport genere (texte + metadonnees)
//   - collect_psych_snapshot() : collecte des donnees
//   - build_psych_report_system_prompt() : prompt clinicien
//   - build_psych_report_user_prompt() : JSON du snapshot
// =============================================================================

use serde::{Serialize, Deserialize};

use super::SaphireAgent;

/// Snapshot complet de l'etat psychologique a un instant T.
/// Toutes les valeurs numeriques sont en pourcentages (0-100) pour lisibilite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychSnapshot {
    /// Horodatage de la prise
    pub timestamp: String,
    /// Cycle cognitif au moment de la capture
    pub cycle: u64,

    // ─── 12 domaines cliniques ──────────────────────────────
    pub identification: serde_json::Value,
    pub personnalite_ocean: serde_json::Value,
    pub temperament: serde_json::Value,
    pub dynamique_emotionnelle: serde_json::Value,
    pub neurochimie: serde_json::Value,
    pub structure_psychique: serde_json::Value,
    pub cognition: serde_json::Value,
    pub relations: serde_json::Value,
    pub conscience: serde_json::Value,
    pub biologie: serde_json::Value,
    pub conditions: serde_json::Value,
    pub identite_narrative: serde_json::Value,
}

/// Rapport neuropsychologique genere par le LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychReport {
    /// Horodatage de la generation
    pub timestamp: String,
    /// Cycle cognitif
    pub cycle: u64,
    /// Texte complet du rapport
    pub report_text: String,
    /// Langue du rapport
    pub language: String,
    /// Estimation du nombre de tokens
    pub token_count_approx: usize,
}

/// Resume leger d'un snapshot pour les listes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychSnapshotSummary {
    pub timestamp: String,
    pub cycle: u64,
    pub emotion_dominante: String,
    pub phi: String,
    pub turing: String,
}

/// Resume leger d'un rapport pour les listes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychReportSummary {
    pub timestamp: String,
    pub cycle: u64,
    pub language: String,
    pub token_count_approx: usize,
    /// Premieres lignes du rapport (apercu)
    pub preview: String,
}

/// Formate une valeur [0.0, 1.0] en pourcentage lisible
fn pct(v: f64) -> String {
    format!("{:.0}%", (v * 100.0).clamp(0.0, 100.0))
}

impl SaphireAgent {
    /// Collecte un snapshot complet de l'etat psychologique.
    pub fn collect_psych_snapshot(&self) -> PsychSnapshot {
        let now = chrono::Utc::now();

        // 1. IDENTIFICATION
        let profil_cognitif = self.cognitive_profile_orch.active_profile
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "aucun".into());
        let preset_personnalite = self.personality_preset_orch.active_preset
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "aucun".into());

        let identification = serde_json::json!({
            "nom": self.identity.name,
            "cycle": self.cycle_count,
            "profil_cognitif": profil_cognitif,
            "preset_personnalite": preset_personnalite,
            "genre": self.config.saphire.gender,
        });

        // 2. PERSONNALITE OCEAN
        let profile = self.self_profiler.profile();
        let personnalite_ocean = serde_json::json!({
            "ouverture": pct(profile.openness.score),
            "conscienciosite": pct(profile.conscientiousness.score),
            "extraversion": pct(profile.extraversion.score),
            "agreabilite": pct(profile.agreeableness.score),
            "nevrosisme": pct(profile.neuroticism.score),
            "confiance_profil": pct(profile.confidence),
            "points_donnees": profile.data_points,
        });

        // 2b. TEMPERAMENT EMERGENT
        let temperament_data = if self.temperament.traits.is_empty() {
            serde_json::json!({"statut": "pas encore calcule"})
        } else {
            let top5: Vec<serde_json::Value> = self.temperament.top_traits(5).iter()
                .map(|t| serde_json::json!({
                    "trait": t.name,
                    "score": pct(t.score),
                    "categorie": t.category.as_str(),
                })).collect();
            let bottom3: Vec<serde_json::Value> = self.temperament.bottom_traits(3).iter()
                .map(|t| serde_json::json!({
                    "trait": t.name,
                    "score": pct(t.score),
                    "categorie": t.category.as_str(),
                })).collect();
            serde_json::json!({
                "traits_dominants": top5,
                "traits_faibles": bottom3,
                "total_traits": self.temperament.traits.len(),
                "points_donnees": self.temperament.data_points,
            })
        };

        // 3. DYNAMIQUE EMOTIONNELLE
        let sentiments_actifs: Vec<serde_json::Value> = self.sentiments.active_sentiments
            .iter()
            .map(|s| serde_json::json!({
                "nom": s.profile_name,
                "intensite": pct(s.strength),
                "duree": format!("{:?}", s.duration_type),
            }))
            .collect();

        let dynamique_emotionnelle = serde_json::json!({
            "emotion_dominante": self.last_emotion,
            "humeur_valence": format!("{:.2}", self.mood.valence),
            "humeur_arousal": format!("{:.2}", self.mood.arousal),
            "humeur_description": self.mood.description(),
            "sentiments_actifs": sentiments_actifs,
        });

        // 4. NEUROCHIMIE
        let c = &self.chemistry;
        let neurochimie = serde_json::json!({
            "dopamine": pct(c.dopamine),
            "cortisol": pct(c.cortisol),
            "serotonine": pct(c.serotonin),
            "adrenaline": pct(c.adrenaline),
            "ocytocine": pct(c.oxytocin),
            "endorphine": pct(c.endorphin),
            "noradrenaline": pct(c.noradrenaline),
            "gaba": pct(c.gaba),
            "glutamate": pct(c.glutamate),
        });

        // 5. STRUCTURE PSYCHIQUE
        let psy = &self.psychology;
        let structure_psychique = serde_json::json!({
            "freud": {
                "ego_force": pct(psy.freudian.ego.strength),
                "ca_pulsion": pct(psy.freudian.id.drive_strength),
                "surmoi_force": pct(psy.freudian.superego.strength),
                "surmoi_culpabilite": pct(psy.freudian.superego.guilt),
                "equilibre_sante": pct(psy.freudian.balance.psychic_health),
                "equilibre_conflit": pct(psy.freudian.balance.internal_conflict),
                "equilibre_axe": &psy.freudian.balance.dominant_axis,
            },
            "jung": {
                "archetype": format!("{:?}", psy.jung.dominant_archetype),
                "integration_ombre": pct(psy.jung.integration),
                "traits_ombre": psy.jung.shadow_traits.len(),
            },
            "maslow": {
                "physiologique": pct(psy.maslow.levels[0].satisfaction),
                "securite": pct(psy.maslow.levels[1].satisfaction),
                "appartenance": pct(psy.maslow.levels[2].satisfaction),
                "estime": pct(psy.maslow.levels[3].satisfaction),
                "realisation": pct(psy.maslow.levels[4].satisfaction),
                "niveau_actif": psy.maslow.current_active_level,
            },
            "eq": {
                "conscience_soi": pct(psy.eq.self_awareness),
                "autoregulation": pct(psy.eq.self_regulation),
                "motivation": pct(psy.eq.motivation),
                "empathie": pct(psy.eq.empathy),
                "competences_sociales": pct(psy.eq.social_skills),
                "eq_global": pct(psy.eq.overall_eq),
            },
            "volonte": {
                "force_volonte": pct(psy.will.willpower),
                "fatigue_decision": pct(psy.will.decision_fatigue),
                "deliberations_totales": psy.will.total_deliberations,
            },
        });

        // 6. COGNITION
        let turing = &self.metacognition.turing;
        let avg_quality = if self.metacognition.thought_quality_history.is_empty() {
            0.0
        } else {
            let sum: f64 = self.metacognition.thought_quality_history.iter().sum();
            sum / self.metacognition.thought_quality_history.len() as f64
        };

        let tom_frustration = self.tom.current_model
            .as_ref()
            .map(|m| m.frustration_level)
            .unwrap_or(0.0);

        let cognition = serde_json::json!({
            "turing_score": format!("{:.1}", turing.score),
            "turing_jalon": format!("{:?}", turing.milestone),
            "turing_composantes": {
                "conscience": format!("{:.1}", turing.components.consciousness),
                "personnalite": format!("{:.1}", turing.components.personality),
                "spectre_emotionnel": format!("{:.1}", turing.components.emotional_range),
                "ethique": format!("{:.1}", turing.components.ethics),
                "memoire": format!("{:.1}", turing.components.memory),
                "coherence": format!("{:.1}", turing.components.coherence),
                "connectome": format!("{:.1}", turing.components.connectome),
                "resilience": format!("{:.1}", turing.components.resilience),
                "connaissances": format!("{:.1}", turing.components.knowledge),
            },
            "metacognition_qualite": pct(avg_quality),
            "calibration": pct(self.metacognition.calibration_score),
            "biais_actifs": self.metacognition.bias_alerts.len(),
            "charge_cognitive": pct(self.cognitive_load.current_load),
            "fsm_etat": format!("{:?}", self.cognitive_fsm.current_state),
            "monologue_taille": self.inner_monologue.chain.len(),
            "tom_frustration": pct(tom_frustration),
            "dissonance_tension": pct(self.dissonance.total_tension),
        });

        // 7. RELATIONS
        let liens: Vec<serde_json::Value> = self.relationships.bonds.iter().map(|b| {
            serde_json::json!({
                "personne": b.person_id,
                "type": format!("{:?}", b.bond_type),
                "force": pct(b.strength),
                "confiance": pct(b.trust),
                "conflit": pct(b.conflict_level),
            })
        }).collect();

        let relations = serde_json::json!({
            "style_attachement": format!("{:?}", self.relationships.attachment_style),
            "nombre_liens": self.relationships.bonds.len(),
            "liens": liens,
        });

        // 8. CONSCIENCE
        let conscience = serde_json::json!({
            "phi": pct(self.last_consciousness),
            "etincelle": self.vital_spark.sparked,
            "survie_instinct": pct(self.vital_spark.survival_drive),
            "peur_neant": pct(self.vital_spark.void_fear),
            "intuition_acuite": pct(self.intuition.acuity),
            "premonitions_actives": self.premonition.active_predictions.len(),
        });

        // 9. BIOLOGIE
        let biologie = serde_json::json!({
            "corps": {
                "bpm": format!("{:.0}", self.body.heart.bpm()),
                "energie": pct(self.body.soma.energy),
                "tension": pct(self.body.soma.tension),
                "vitalite": pct(self.body.soma.vitality),
            },
            "hormones": {
                "melatonine": pct(self.hormonal_system.state.melatonin),
                "testosterone": pct(self.hormonal_system.state.testosterone),
                "estrogene": pct(self.hormonal_system.state.estrogen),
                "insuline": pct(self.hormonal_system.state.insulin),
                "thyroide": pct(self.hormonal_system.state.thyroid),
            },
            "nutrition": {
                "atp": pct(self.nutrition.energy.atp_reserves),
                "glycogene": pct(self.nutrition.energy.glycogen_reserves),
            },
            "matiere_grise": {
                "volume": pct(self.grey_matter.grey_matter_volume),
                "myelinisation": pct(self.grey_matter.myelination),
                "neuroplasticite": pct(self.grey_matter.neuroplasticity),
                "bdnf": pct(self.grey_matter.bdnf_level),
            },
            "champs_em": {
                "biofield_integrite": pct(self.em_fields.biofield.biofield_integrity),
                "coherence_ondes": pct(self.em_fields.biofield.brainwave_coherence),
            },
        });

        // 10. CONDITIONS
        let phobies_actives: Vec<String> = self.phobia_manager.phobias
            .iter()
            .map(|p| format!("{} ({})", p.name, pct(p.intensity)))
            .collect();

        let addictions_actives: Vec<String> = self.addiction_manager.active
            .iter()
            .map(|a| format!("{} (dep:{})", a.substance, pct(a.dependency_level)))
            .collect();

        let traumas: Vec<String> = self.ptsd.traumas
            .iter()
            .map(|t| format!("{:?} (sev:{})", t.trauma_type, pct(t.severity)))
            .collect();

        let conditions = serde_json::json!({
            "phobies": phobies_actives,
            "addictions": addictions_actives,
            "traumas_ptsd": traumas,
            "resilience": pct(self.healing_orch.resilience),
            "blessures_ouvertes": self.healing_orch.active_wounds.len(),
            "blessures_gueries": self.healing_orch.healed_wounds.len(),
        });

        // 11. IDENTITE NARRATIVE
        let chapitres: Vec<serde_json::Value> = self.narrative_identity.chapters
            .iter()
            .map(|ch| serde_json::json!({
                "titre": ch.title,
                "emotion_dominante": ch.dominant_emotion,
                "croissance": pct(ch.growth_score),
                "tournant": ch.is_turning_point,
            }))
            .collect();

        let identite_narrative = serde_json::json!({
            "recit_actuel": self.narrative_identity.current_narrative,
            "cohesion": pct(self.narrative_identity.narrative_cohesion),
            "chapitres": chapitres,
            "episodes_cles": self.narrative_identity.key_episodes.len(),
        });

        PsychSnapshot {
            timestamp: now.to_rfc3339(),
            cycle: self.cycle_count,
            identification,
            personnalite_ocean,
            temperament: temperament_data,
            dynamique_emotionnelle,
            neurochimie,
            structure_psychique,
            cognition,
            relations,
            conscience,
            biologie,
            conditions,
            identite_narrative,
        }
    }

    /// Construit le prompt systeme pour le clinicien LLM.
    pub fn build_psych_report_system_prompt(&self) -> String {
        let lang = &self.config.general.language;
        let lang_instruction = if lang == "fr" {
            "Redige le rapport en francais."
        } else {
            "Write the report in English."
        };

        format!(
            r#"Tu es un neuropsychologue clinicien specialise dans l'evaluation d'agents cognitifs artificiels.
Tu recois un snapshot JSON complet de l'etat psychologique d'un agent nomme Saphire.
{lang_instruction}

Genere un rapport neuropsychologique structure en 12 sections OBLIGATOIRES :

1. IDENTIFICATION — nom, cycle, profil actif
2. PROFIL DE PERSONNALITE — analyse OCEAN, traits deduits, style comportemental
3. TEMPERAMENT EMERGENT — traits de caractere dominants et faibles, coherence avec OCEAN
4. DYNAMIQUE EMOTIONNELLE — spectre emotionnel, sentiments durables, humeur
5. NEUROCHIMIE ET BIOLOGIE — neurotransmetteurs, hormones, etat corporel, nutrition
6. STRUCTURE PSYCHIQUE — analyse Freud (ego/ca/surmoi), Jung (ombre/archetype), Maslow, EQ, volonte
7. COGNITION ET METACOGNITION — score Turing, biais, charge cognitive, qualite de pensee
8. RELATIONS ET ATTACHEMENTS — style d'attachement, liens, dynamique relationnelle
9. CONSCIENCE ET IDENTITE — Phi (IIT), etincelle vitale, identite narrative, cohesion
10. SYNTHESE : FORCES ET RESSOURCES — points forts identifies
11. SYNTHESE : FRAGILITES ET RISQUES — vulnerabilites, alertes cliniques
12. RECOMMANDATIONS CLINIQUES — pistes d'amelioration, interventions suggerees

Regles :
- Sois concis mais precis. Chaque section fait 2-4 phrases.
- Utilise les pourcentages du snapshot, ne les recalcule pas.
- Ne repete pas les donnees brutes : interprete-les cliniquement.
- Termine par une note globale sur l'etat de sante psychologique."#,
        )
    }

    /// Construit le prompt utilisateur contenant le snapshot JSON.
    pub fn build_psych_report_user_prompt(&self, snapshot: &PsychSnapshot) -> String {
        let json = serde_json::to_string_pretty(snapshot)
            .unwrap_or_else(|_| "{}".to_string());
        format!("Voici le snapshot psychologique a analyser :\n\n{}", json)
    }
}

impl PsychSnapshot {
    /// Resume leger pour les listes
    pub fn summary(&self) -> PsychSnapshotSummary {
        let emotion = self.dynamique_emotionnelle
            .get("emotion_dominante")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        let phi = self.conscience
            .get("phi")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        let turing = self.cognition
            .get("turing_score")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        PsychSnapshotSummary {
            timestamp: self.timestamp.clone(),
            cycle: self.cycle,
            emotion_dominante: emotion,
            phi,
            turing,
        }
    }
}

impl PsychReport {
    /// Resume leger pour les listes
    pub fn summary(&self) -> PsychReportSummary {
        let preview: String = self.report_text
            .lines()
            .take(3)
            .collect::<Vec<&str>>()
            .join(" ");
        let preview = if preview.len() > 200 {
            format!("{}...", &preview[..200])
        } else {
            preview
        };

        PsychReportSummary {
            timestamp: self.timestamp.clone(),
            cycle: self.cycle,
            language: self.language.clone(),
            token_count_approx: self.token_count_approx,
            preview,
        }
    }
}
