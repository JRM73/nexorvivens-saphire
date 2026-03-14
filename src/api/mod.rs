// =============================================================================
// api/mod.rs — Module API : routeur, etat partage, handlers, securite
//
// Role : Ce module regroupe tous les handlers HTTP/WebSocket de l'application
// Saphire. Il expose AppState, ControlMessage et la fonction build_router()
// qui construit le routeur axum complet avec toutes les routes.
//
// Securite :
//   - middleware.rs : authentification Bearer token, rate limiting, CORS
//   - Les routes statiques et /api/health sont publiques
//   - Les WebSockets ont une validation d'origine
//   - Toutes les routes /api/* sont protegees par auth + rate limit
//
// Architecture :
//   - state.rs : AppState + ControlMessage
//   - middleware.rs : auth, rate limit, CORS
//   - static_files.rs : fichiers embarques (HTML, CSS, JS, SVG)
//   - system.rs : sante, config, statut systeme, backup, purge
//   - chemistry.rs : neurochimie
//   - memory.rs : memoire 4 niveaux
//   - brain.rs : monde + connaissances basiques
//   - body.rs : corps virtuel + coeur
//   - vital.rs : VitalSpark, Intuition, Premonition
//   - ethics.rs : ethique personnelle 3 couches
//   - senses.rs : Sensorium + sens emergents
//   - knowledge.rs : sources + stats connaissances
//   - algorithms.rs : orchestrateur d'algorithmes
//   - orchestrators.rs : reves, desirs, apprentissage, attention, guerison
//   - metrics.rs : toutes les metriques temporelles
//   - logs.rs : logs, traces, historique LLM
//   - factory.rs : valeurs d'usine, diff, reset
//   - websocket.rs : WebSocket principal + dashboard
// =============================================================================

pub mod state;
pub mod middleware;
pub mod static_files;
pub mod system;
pub mod chemistry;
pub mod memory;
pub mod brain;
pub mod body;
pub mod vital;
pub mod ethics;
pub mod senses;
pub mod knowledge;
pub mod algorithms;
pub mod orchestrators;
pub mod nn_learnings;
pub mod metrics;
pub mod logs;
pub mod factory;
pub mod psychology;
pub mod sleep;
pub mod subconscious;
pub mod profiles;
pub mod personalities;
pub mod needs;
pub mod hormones;
pub mod mortality;
pub mod conditions;
pub mod relationships;
pub mod cognition;
pub mod personality;
pub mod neuroscience;
pub mod nutrition;
pub mod grey_matter;
pub mod fields;
pub mod psych_report;
pub mod temperament;
pub mod sensoria;
pub mod spine;
pub mod curiosity;
pub mod drift;
pub mod websocket;

// Re-exports pour acces direct depuis main.rs
pub use state::{AppState, ControlMessage};

use axum::{Router, routing::{get, post}};

/// Construit le routeur axum complet avec toutes les routes de l'application.
/// Les routes sont separees en publiques (statiques, health, ws) et protegees
/// (toutes les /api/* sauf health) avec auth Bearer + rate limiting.
///
/// # Parametres
/// - `state` : etat partage de l'application (AppState)
///
/// # Retour
/// Le routeur axum pret a etre utilise par le serveur web.
pub fn build_router(state: AppState) -> Router {
    // ─── CORS configurable ──────────────────────────────────────────────
    let cors = middleware::build_cors_layer(&state.allowed_origins);

    // ─── Routes API protegees (auth + rate limit) ───────────────────────
    let protected_api = Router::new()
        .route("/api/config", get(system::api_get_config).post(system::api_post_config))
        .route("/api/chemistry", get(chemistry::api_get_chemistry))
        .route("/api/stabilize", post(system::api_stabilize))
        .route("/api/knowledge", get(brain::api_get_knowledge))
        .route("/api/world", get(brain::api_get_world))
        .route("/api/memory", get(memory::api_get_memory))
        // ─── API Dashboard : Logs ───
        .route("/api/logs", get(logs::api_get_logs))
        .route("/api/logs/export", get(logs::api_export_logs))
        .route("/api/logs/:id", get(logs::api_get_log_by_id))
        // ─── API Dashboard : Memoire ───
        .route("/api/memory/working", get(memory::api_get_working_memory))
        .route("/api/memory/episodic", get(memory::api_list_episodic))
        .route("/api/memory/episodic/:id", get(memory::api_get_episodic_by_id))
        .route("/api/memory/ltm", get(memory::api_list_ltm))
        .route("/api/memory/ltm/:id", get(memory::api_get_ltm_by_id))
        .route("/api/memory/founding", get(memory::api_list_founding))
        .route("/api/memory/stats", get(memory::api_memory_stats))
        .route("/api/memory/archives", get(memory::api_list_archives))
        .route("/api/memory/archives/stats", get(memory::api_archive_stats))
        // ─── API Dashboard : Traces ───
        .route("/api/trace/last", get(logs::api_get_trace_last))
        .route("/api/trace/:cycle", get(logs::api_get_trace))
        .route("/api/traces", get(logs::api_list_traces))
        // ─── API Dashboard : Metriques ───
        .route("/api/metrics/chemistry", get(metrics::api_metrics_chemistry))
        .route("/api/metrics/emotions", get(metrics::api_metrics_emotions))
        .route("/api/metrics/decisions", get(metrics::api_metrics_decisions))
        .route("/api/metrics/satisfaction", get(metrics::api_metrics_satisfaction))
        .route("/api/metrics/llm", get(metrics::api_metrics_llm))
        .route("/api/metrics/ocean_history", get(metrics::api_metrics_ocean_history))
        .route("/api/metrics/thought_types", get(metrics::api_metrics_thought_types))
        // ─── API Dashboard : LLM History ───
        .route("/api/llm/history", get(logs::api_llm_history))
        .route("/api/llm/history/:id", get(logs::api_llm_history_by_id))
        // ─── API Dashboard : Corps & Coeur ───
        .route("/api/body/status", get(body::api_body_status))
        .route("/api/body/heart", get(body::api_body_heart))
        .route("/api/body/heart/history", get(body::api_body_heart_history))
        .route("/api/body/history", get(body::api_body_history))
        .route("/api/body/vitals", get(body::api_body_vitals))
        .route("/api/body/milestones", get(body::api_body_milestones))
        .route("/api/metrics/heart", get(metrics::api_metrics_heart))
        .route("/api/metrics/body", get(metrics::api_metrics_body))
        // ─── API Dashboard : Systeme ───
        .route("/api/system/status", get(system::api_system_status))
        .route("/api/identity", get(system::api_identity))
        .route("/api/hardware", get(system::api_hardware))
        .route("/api/genome", get(system::api_genome))
        .route("/api/connectome", get(system::api_connectome))
        .route("/api/connectome/metrics", get(system::api_connectome_metrics))
        // ─── API Dashboard : Metacognition + Turing ───
        .route("/api/metacognition", get(system::api_metacognition))
        .route("/api/turing", get(system::api_turing))
        // ─── API Dashboard : Relations et Famille ───
        .route("/api/relationships", get(relationships::api_relationships))
        .route("/api/relationships/chemistry", get(relationships::api_relationships_chemistry))
        .route("/api/family", get(relationships::api_family))
        .route("/api/family/chemistry", get(relationships::api_family_chemistry))
        // ─── API Dashboard : Mortalite ───
        .route("/api/mortality", get(mortality::api_mortality_status))
        .route("/api/mortality/poison", post(mortality::api_mortality_poison))
        .route("/api/mortality/revive", post(mortality::api_mortality_revive))
        // ─── API Dashboard : Conditions et afflictions ───
        .route("/api/conditions/phobias", get(conditions::api_phobias_status))
        .route("/api/conditions/motion_sickness", get(conditions::api_motion_sickness_status))
        .route("/api/conditions/motion_sickness/trigger", post(conditions::api_motion_sickness_trigger))
        .route("/api/conditions/eating", get(conditions::api_eating_disorder_status))
        .route("/api/conditions/disabilities", get(conditions::api_disabilities_status))
        .route("/api/conditions/extreme", get(conditions::api_extreme_condition_status))
        .route("/api/conditions/addictions", get(conditions::api_addictions_status))
        .route("/api/conditions/trauma", get(conditions::api_trauma_status))
        .route("/api/conditions/nde", get(conditions::api_nde_status))
        .route("/api/conditions/drugs", get(conditions::api_drugs_status))
        .route("/api/conditions/drugs/administer", post(conditions::api_drugs_administer))
        .route("/api/conditions/iq", get(conditions::api_iq_status))
        .route("/api/conditions/sexuality", get(conditions::api_sexuality_status))
        .route("/api/conditions/degenerative", get(conditions::api_degenerative_status))
        .route("/api/conditions/medical", get(conditions::api_medical_status))
        .route("/api/conditions/culture", get(conditions::api_culture_status))
        .route("/api/conditions/precarity", get(conditions::api_precarity_status))
        .route("/api/conditions/employment", get(conditions::api_employment_status))
        // ─── API Dashboard : LoRA (collecte + export) ───
        .route("/api/lora/stats", get(system::api_lora_stats))
        .route("/api/lora/export", get(system::api_lora_export))
        .route("/api/system/db/tables", get(system::api_db_tables))
        .route("/api/system/backup", post(system::api_backup))
        .route("/api/system/consolidate", post(system::api_consolidate))
        .route("/api/system/purge_logs", post(system::api_purge_logs))
        // ─── API Dashboard : Vital / Intuition / Premonition ───
        .route("/api/vital/status", get(vital::api_vital_status))
        .route("/api/vital/threats", get(vital::api_vital_threats))
        .route("/api/intuition/status", get(vital::api_intuition_status))
        .route("/api/intuition/history", get(vital::api_intuition_history))
        .route("/api/premonition/active", get(vital::api_premonition_active))
        .route("/api/premonition/history", get(vital::api_premonition_history))
        // ─── API Dashboard : Ethique personnelle ───
        .route("/api/ethics/layers", get(ethics::api_ethics_layers))
        .route("/api/ethics/personal", get(ethics::api_ethics_personal))
        .route("/api/ethics/personal/:id", get(ethics::api_ethics_personal_by_id))
        .route("/api/ethics/readiness", get(ethics::api_ethics_readiness))
        // ─── API Dashboard : Sens ───
        .route("/api/senses/status", get(senses::api_senses_status))
        .route("/api/senses/emergent", get(senses::api_senses_emergent))
        // ─── API Dashboard : Connaissances ───
        .route("/api/knowledge/sources", get(knowledge::api_knowledge_sources))
        .route("/api/knowledge/stats", get(knowledge::api_knowledge_stats))
        // ─── API Dashboard : Metriques enrichies ───
        .route("/api/metrics/vital", get(metrics::api_metrics_vital))
        .route("/api/metrics/intuition", get(metrics::api_metrics_intuition))
        .route("/api/metrics/premonition", get(metrics::api_metrics_premonition))
        .route("/api/metrics/ethics", get(metrics::api_metrics_ethics))
        .route("/api/metrics/senses", get(metrics::api_metrics_senses))
        .route("/api/metrics/senses_acuity", get(metrics::api_metrics_senses_acuity))
        .route("/api/metrics/emergent", get(metrics::api_metrics_emergent))
        .route("/api/metrics/knowledge", get(metrics::api_metrics_knowledge))
        // ─── API Dashboard : Algorithmes ───
        .route("/api/algorithms/status", get(algorithms::api_algorithms_status))
        .route("/api/algorithms/catalog", get(algorithms::api_algorithms_catalog))
        .route("/api/algorithms/history", get(algorithms::api_algorithms_history))
        // ─── API Dashboard : Orchestrateurs ───
        .route("/api/dreams/status", get(orchestrators::api_dreams_status))
        .route("/api/dreams/journal", get(orchestrators::api_dreams_journal))
        .route("/api/desires/status", get(orchestrators::api_desires_status))
        .route("/api/desires/active", get(orchestrators::api_desires_active))
        .route("/api/desires/needs", get(orchestrators::api_desires_needs))
        .route("/api/learning/status", get(orchestrators::api_learning_status))
        .route("/api/learning/lessons", get(orchestrators::api_learning_lessons))
        .route("/api/learning/stats", get(orchestrators::api_learning_stats))
        .route("/api/attention/status", get(orchestrators::api_attention_status))
        .route("/api/healing/status", get(orchestrators::api_healing_status))
        .route("/api/healing/wounds", get(orchestrators::api_healing_wounds))
        .route("/api/healing/strategies", get(orchestrators::api_healing_strategies))
        // ─── API Dashboard : Apprentissages vectoriels ───
        .route("/api/nn-learnings/recent", get(nn_learnings::api_nn_learnings_recent))
        .route("/api/nn-learnings/stats", get(nn_learnings::api_nn_learnings_stats))
        // ─── API Dashboard : Profils cognitifs neurodivergents ───
        .route("/api/profiles", get(profiles::api_list_profiles))
        .route("/api/profiles/current", get(profiles::api_current_profile))
        .route("/api/profiles/load", post(profiles::api_load_profile))
        .route("/api/profiles/reset", post(profiles::api_reset_profile))
        .route("/api/profiles/compare", get(profiles::api_compare_profiles))
        // ─── API Dashboard : Presets de personnalite (archetypes) ───
        .route("/api/personalities", get(personalities::api_list_personalities))
        .route("/api/personalities/current", get(personalities::api_current_personality))
        .route("/api/personalities/load", post(personalities::api_load_personality))
        .route("/api/personalities/reset", post(personalities::api_reset_personality))
        .route("/api/personalities/compare", get(personalities::api_compare_personalities))
        // ─── API Dashboard : Hormones et recepteurs ───
        .route("/api/hormones", get(hormones::api_hormones_status))
        .route("/api/hormones/receptors", get(hormones::api_hormones_receptors))
        .route("/api/hormones/cycle", get(hormones::api_hormones_cycle))
        // ─── API Dashboard : Besoins primaires (faim, soif) ───
        .route("/api/needs", get(needs::api_needs_status))
        .route("/api/needs/eat", post(needs::api_needs_eat))
        .route("/api/needs/drink", post(needs::api_needs_drink))
        // ─── Metriques orchestrateurs ───
        .route("/api/metrics/attention", get(metrics::api_metrics_attention))
        .route("/api/metrics/desires", get(metrics::api_metrics_desires))
        .route("/api/metrics/learning", get(metrics::api_metrics_learning))
        .route("/api/metrics/healing", get(metrics::api_metrics_healing))
        .route("/api/metrics/dreams", get(metrics::api_metrics_dreams))
        .route("/api/metrics/nn_learnings", get(metrics::api_metrics_nn_learnings))
        .route("/api/metrics/chemical_health", get(metrics::api_metrics_chemical_health))
        .route("/api/metrics/receptors", get(metrics::api_metrics_receptors))
        .route("/api/metrics/bdnf", get(metrics::api_metrics_bdnf))
        .route("/api/metrics/spine", get(metrics::api_metrics_spine))
        .route("/api/metrics/curiosity", get(metrics::api_metrics_curiosity))
        // Factory defaults (valeurs d'usine)
        .route("/api/factory/defaults", get(factory::api_factory_defaults))
        .route("/api/factory/diff", get(factory::api_factory_diff))
        .route("/api/factory/reset", post(factory::api_factory_reset))
        // Psychologie (6 cadres) — vues d'ensemble
        .route("/api/psychology/status", get(psychology::api_psychology_status))
        .route("/api/psychology/freudian", get(psychology::api_psychology_freudian))
        .route("/api/psychology/maslow", get(psychology::api_psychology_maslow))
        .route("/api/psychology/toltec", get(psychology::api_psychology_toltec))
        .route("/api/psychology/shadow", get(psychology::api_psychology_shadow))
        .route("/api/psychology/eq", get(psychology::api_psychology_eq))
        .route("/api/psychology/flow", get(psychology::api_psychology_flow))
        // ─── Psyche (Freud) ───
        .route("/api/psyche/status", get(psychology::api_psyche_status))
        .route("/api/psyche/defenses", get(psychology::api_psyche_defenses))
        .route("/api/psyche/drives", get(psychology::api_psyche_drives))
        // ─── Maslow ───
        .route("/api/maslow/pyramid", get(psychology::api_maslow_pyramid))
        .route("/api/maslow/ceiling", get(psychology::api_maslow_ceiling))
        .route("/api/maslow/needs", get(psychology::api_maslow_needs))
        // ─── Tolteques ───
        .route("/api/toltec/agreements", get(psychology::api_toltec_agreements))
        .route("/api/toltec/history", get(psychology::api_toltec_history))
        // ─── Ombre (Jung) ───
        .route("/api/shadow/status", get(psychology::api_shadow_status))
        .route("/api/shadow/archetype_history", get(psychology::api_shadow_archetype_history))
        // ─── EQ (Goleman) ───
        .route("/api/eq/status", get(psychology::api_eq_status))
        .route("/api/eq/history", get(psychology::api_eq_history))
        // ─── Flow ───
        .route("/api/flow/status", get(psychology::api_flow_status))
        .route("/api/flow/history", get(psychology::api_flow_history))
        // ─── Volonte ───
        .route("/api/will/status", get(psychology::api_will_status))
        .route("/api/will/last", get(psychology::api_will_last))
        .route("/api/will/history", get(psychology::api_will_history))
        .route("/api/will/stats", get(psychology::api_will_stats))
        .route("/api/monologue/current", get(psychology::api_monologue_current))
        .route("/api/metrics/will", get(psychology::api_metrics_will))
        // ─── Modele LLM ───
        .route("/api/model/info", get(psychology::api_model_info))
        // ─── Sommeil ───
        .route("/api/sleep/status", get(sleep::api_sleep_status))
        .route("/api/sleep/history", get(sleep::api_sleep_history))
        .route("/api/sleep/drive", get(sleep::api_sleep_drive))
        .route("/api/sleep/force", post(sleep::api_sleep_force))
        .route("/api/sleep/wake", post(sleep::api_sleep_wake))
        // ─── Subconscient ───
        .route("/api/subconscious/status", get(subconscious::api_subconscious_status))
        .route("/api/subconscious/associations", get(subconscious::api_subconscious_associations))
        .route("/api/subconscious/insights", get(subconscious::api_subconscious_insights))
        // ─── Connexions neuronales ───
        .route("/api/connections/list", get(subconscious::api_connections_list))
        .route("/api/connections/stats", get(subconscious::api_connections_stats))
        // ─── Metriques sommeil + subconscient ───
        .route("/api/metrics/sleep", get(subconscious::api_metrics_sleep))
        .route("/api/metrics/subconscious", get(subconscious::api_metrics_subconscious))
        // ─── Metriques psychologiques ───
        .route("/api/metrics/psyche", get(metrics::api_metrics_psyche))
        .route("/api/metrics/maslow", get(metrics::api_metrics_maslow))
        .route("/api/metrics/eq", get(metrics::api_metrics_eq))
        .route("/api/metrics/flow", get(metrics::api_metrics_flow))
        .route("/api/metrics/shadow", get(metrics::api_metrics_shadow))
        // ─── API Dashboard : Modules cognitifs avances ───
        .route("/api/tom/status", get(cognition::api_tom_status))
        .route("/api/monologue/chain", get(cognition::api_monologue_chain))
        .route("/api/dissonance/status", get(cognition::api_dissonance_status))
        .route("/api/dissonance/beliefs", get(cognition::api_dissonance_beliefs))
        .route("/api/prospective/intentions", get(cognition::api_prospective_intentions))
        .route("/api/narrative/identity", get(cognition::api_narrative_identity))
        .route("/api/narrative/chapters", get(cognition::api_narrative_chapters))
        .route("/api/analogies/recent", get(cognition::api_analogies_recent))
        .route("/api/cognitive-load/status", get(cognition::api_cognitive_load_status))
        .route("/api/imagery/active", get(cognition::api_imagery_active))
        .route("/api/source-monitor/traced", get(cognition::api_source_monitor_traced))
        .route("/api/metacognition/biases", get(cognition::api_metacognition_biases))
        .route("/api/sentiments/status", get(cognition::api_sentiments_status))
        .route("/api/sentiments/history", get(cognition::api_sentiments_history))
        // ─── API Dashboard : Portrait de personnalite temporel ───
        .route("/api/personality/timeline", get(personality::api_personality_timeline))
        .route("/api/personality/emotions", get(personality::api_personality_emotions))
        .route("/api/personality/consciousness", get(personality::api_personality_consciousness))
        .route("/api/personality/psychology", get(personality::api_personality_psychology))
        .route("/api/personality/relationships", get(personality::api_personality_relationships))
        .route("/api/personality/journal", get(personality::api_personality_journal))
        // ─── API Dashboard : Neurosciences avancees ───
        .route("/api/receptors", get(neuroscience::api_receptors_status))
        .route("/api/receptors/sensitivity", get(neuroscience::api_receptors_sensitivity))
        .route("/api/brain-regions", get(neuroscience::api_brain_regions_status))
        .route("/api/predictive", get(neuroscience::api_predictive_status))
        .route("/api/reconsolidation", get(neuroscience::api_reconsolidation_status))
        .route("/api/consciousness-metrics", get(neuroscience::api_consciousness_metrics))
        // ─── API Dashboard : Nutrition, Matiere Grise, Champs EM ───
        .route("/api/nutrition/status", get(nutrition::api_nutrition_status))
        .route("/api/nutrition/deficiencies", get(nutrition::api_nutrition_deficiencies))
        .route("/api/grey-matter/status", get(grey_matter::api_grey_matter_status))
        .route("/api/grey-matter/bdnf", get(grey_matter::api_grey_matter_bdnf))
        .route("/api/fields/status", get(fields::api_fields_status))
        .route("/api/fields/biofield", get(fields::api_fields_biofield))
        // ─── API Dashboard : Rapport neuropsychologique ───
        .route("/api/psych/snapshot", get(psych_report::api_psych_snapshot))
        .route("/api/psych/report", post(psych_report::api_psych_report))
        .route("/api/psych/snapshots", get(psych_report::api_psych_snapshots))
        .route("/api/psych/reports", get(psych_report::api_psych_reports))
        // ─── API Dashboard : Temperament emergent ───
        .route("/api/temperament", get(temperament::api_temperament_status))
        // ─── API Dashboard : Game AI (influence map, FSM cognitive, steering) ───
        .route("/api/influence-map/status", get(cognition::api_influence_map_status))
        .route("/api/cognitive-fsm/status", get(cognition::api_cognitive_fsm_status))
        .route("/api/steering/status", get(cognition::api_steering_status))
        // ─── API Dashboard : MAP Sync ───
        .route("/api/map-sync/status", get(cognition::api_map_sync_status))
        // ─── API Dashboard : Colonne vertebrale ───
        .route("/api/spine/status", get(spine::api_spine_status))
        // ─── API Dashboard : Curiosite ───
        .route("/api/curiosity/status", get(curiosity::api_curiosity_status))
        // ─── API Dashboard : Derive persona ───
        .route("/api/drift/status", get(drift::api_drift_status))
        // ─── Sensoria : service sensoriel (oreilles, bouche, yeux) ───
        .route("/api/hear", post(sensoria::api_hear))
        .route("/api/speak", post(sensoria::api_speak))
        .route("/api/sensoria", get(sensoria::api_sensoria_status))
        // ─── Middlewares de securite (ordre : rate_limit → auth → handler) ───
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::auth_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::rate_limit_middleware,
        ));

    // ─── Routeur complet ────────────────────────────────────────────────
    Router::new()
        // Routes publiques (pas d'auth, pas de rate limit)
        .route("/", get(static_files::index_handler))
        .route("/style.css", get(static_files::css_handler))
        .route("/app.js", get(static_files::js_handler))
        .route("/auth.js", get(static_files::auth_js_handler))
        .route("/favicon.svg", get(static_files::favicon_handler))
        .route("/i18n.js", get(static_files::i18n_js_handler))
        .route("/i18n/:lang", get(static_files::i18n_handler))
        .route("/dashboard", get(static_files::dashboard_handler))
        .route("/pipeline-editor", get(static_files::pipeline_editor_handler))
        .route("/brain-map", get(static_files::brain_map_handler))
        // Health check (public, pour Docker/sondes)
        .route("/api/health", get(system::health_handler))
        // WebSocket (validation d'origine dans le handler)
        .route("/ws", get(websocket::ws_handler))
        .route("/ws/dashboard", get(websocket::ws_dashboard_handler))
        // Routes API protegees
        .merge(protected_api)
        // CORS configurable (layer global)
        .layer(cors)
        .with_state(state)
}
