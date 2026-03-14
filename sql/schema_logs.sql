-- ============================================================
-- SAPHIRE — schema_logs.sql
-- ============================================================
-- Schema de la base de donnees de logs separee.
-- 4 tables : system_logs, cognitive_traces, llm_history, metric_snapshots.
-- ============================================================

-- ── Logs systeme ──
-- Journal centralise de tous les evenements de Saphire.
CREATE TABLE IF NOT EXISTS system_logs (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level TEXT NOT NULL DEFAULT 'INFO',
    category TEXT NOT NULL DEFAULT 'general',
    message TEXT NOT NULL,
    details JSONB DEFAULT '{}',
    cycle BIGINT DEFAULT 0,
    session_id BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON system_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_level ON system_logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_category ON system_logs(category);
CREATE INDEX IF NOT EXISTS idx_logs_cycle ON system_logs(cycle);

-- ── Traces cognitives ──
-- Trace complete d'un cycle cognitif : NLP, brain, consensus, chimie,
-- emotion, conscience, regulation, LLM, memoire.
CREATE TABLE IF NOT EXISTS cognitive_traces (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_type TEXT NOT NULL DEFAULT 'autonomous',
    input_text TEXT DEFAULT '',
    nlp_data JSONB DEFAULT '{}',
    brain_data JSONB DEFAULT '{}',
    consensus_data JSONB DEFAULT '{}',
    chemistry_before JSONB DEFAULT '{}',
    chemistry_after JSONB DEFAULT '{}',
    emotion_data JSONB DEFAULT '{}',
    consciousness_data JSONB DEFAULT '{}',
    regulation_data JSONB DEFAULT '{}',
    llm_data JSONB DEFAULT '{}',
    memory_data JSONB DEFAULT '{}',
    heart_data JSONB DEFAULT '{}',
    body_data JSONB DEFAULT '{}',
    ethics_data JSONB DEFAULT '{}',
    duration_ms REAL DEFAULT 0.0,
    session_id BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_traces_cycle ON cognitive_traces(cycle DESC);
CREATE INDEX IF NOT EXISTS idx_traces_timestamp ON cognitive_traces(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_traces_source ON cognitive_traces(source_type);

-- ── Historique LLM ──
-- Chaque requete/reponse au LLM avec timings et metadonnees.
CREATE TABLE IF NOT EXISTS llm_history (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cycle BIGINT DEFAULT 0,
    request_type TEXT NOT NULL DEFAULT 'chat',
    model TEXT DEFAULT '',
    system_prompt TEXT DEFAULT '',
    user_prompt TEXT DEFAULT '',
    response TEXT DEFAULT '',
    temperature REAL DEFAULT 0.7,
    max_tokens INTEGER DEFAULT 300,
    duration_ms REAL DEFAULT 0.0,
    token_estimate INTEGER DEFAULT 0,
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT DEFAULT '',
    session_id BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_llm_timestamp ON llm_history(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_llm_cycle ON llm_history(cycle);
CREATE INDEX IF NOT EXISTS idx_llm_type ON llm_history(request_type);

-- ── Snapshots metriques ──
-- Un snapshot par cycle avec toutes les metriques cles.
CREATE TABLE IF NOT EXISTS metric_snapshots (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cycle BIGINT NOT NULL,
    dopamine REAL DEFAULT 0.0,
    cortisol REAL DEFAULT 0.0,
    serotonin REAL DEFAULT 0.0,
    adrenaline REAL DEFAULT 0.0,
    oxytocin REAL DEFAULT 0.0,
    endorphin REAL DEFAULT 0.0,
    noradrenaline REAL DEFAULT 0.0,
    emotion TEXT DEFAULT '',
    valence REAL DEFAULT 0.0,
    arousal REAL DEFAULT 0.0,
    dominance REAL DEFAULT 0.0,
    consciousness_level REAL DEFAULT 0.0,
    phi REAL DEFAULT 0.0,
    consensus_score REAL DEFAULT 0.0,
    decision TEXT DEFAULT '',
    satisfaction REAL DEFAULT 0.0,
    thought_type TEXT DEFAULT '',
    llm_response_time_ms REAL DEFAULT 0.0,
    heart_bpm REAL DEFAULT 0.0,
    heart_beat_count BIGINT DEFAULT 0,
    heart_hrv REAL DEFAULT 0.0,
    heart_is_racing BOOLEAN DEFAULT FALSE,
    body_energy REAL DEFAULT 0.0,
    body_tension REAL DEFAULT 0.0,
    body_warmth REAL DEFAULT 0.0,
    body_comfort REAL DEFAULT 0.0,
    body_pain REAL DEFAULT 0.0,
    body_vitality REAL DEFAULT 0.0,
    body_breath_rate REAL DEFAULT 0.0,
    body_awareness REAL DEFAULT 0.0,
    ethics_active_count INTEGER DEFAULT 0,
    session_id BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metric_snapshots(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_metrics_cycle ON metric_snapshots(cycle);

-- Migration : ajouter les colonnes heart/body si elles n'existent pas
DO $$ BEGIN
    -- cognitive_traces
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS heart_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS body_data JSONB DEFAULT '{}';
    -- metric_snapshots
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS heart_bpm REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS heart_beat_count BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS heart_hrv REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS heart_is_racing BOOLEAN DEFAULT FALSE;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_energy REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_tension REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_warmth REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_comfort REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_pain REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_vitality REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_breath_rate REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS body_awareness REAL DEFAULT 0.0;
    -- ethics
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS ethics_data JSONB DEFAULT '{}';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS ethics_active_count INTEGER DEFAULT 0;
    -- vital / intuition / premonition
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS vital_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS intuition_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS premonition_data JSONB DEFAULT '{}';
    -- senses (Sensorium)
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS senses_data JSONB DEFAULT '{}';
    -- metric_snapshots enrichi (vital, intuition, premonition, senses, knowledge)
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS survival_drive REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS existence_attachment REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS intuition_acuity REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS intuition_accuracy REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS premonition_accuracy REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS active_predictions INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS senses_richness REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS senses_dominant TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS reading_beauty REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS ambiance_scent TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS contact_warmth REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS emergent_senses_germinated INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS knowledge_sources_used JSONB DEFAULT '{}';
    -- Orchestrateurs : colonnes traces cognitives
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS attention_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS algorithm_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS desire_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS learning_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS healing_data JSONB DEFAULT '{}';
    -- Orchestrateurs : colonnes metric_snapshots
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS attention_focus TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS attention_depth REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS attention_fatigue REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS attention_concentration REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS desires_active INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS desires_fulfilled_total INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS desires_top_priority TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS needs_comprehension REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS needs_connection REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS needs_expression REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS needs_growth REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS needs_meaning REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS lessons_total INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS lessons_confirmed INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS lessons_contradicted INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS behavior_changes_total INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS wounds_active INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS wounds_healed_total INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS resilience REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS dreams_total INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS dreams_insights_total INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS last_dream_type TEXT DEFAULT '';
    -- Psychologie : colonnes traces cognitives
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS psychology_data JSONB DEFAULT '{}';
    -- Psychologie : colonnes metric_snapshots
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_id_drive REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_id_frustration REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_ego_strength REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_ego_anxiety REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_superego_guilt REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_superego_pride REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_conflict REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_health REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_ceiling INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_level1 REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_level2 REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_level3 REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_level4 REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_level5 REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS shadow_archetype TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS shadow_integration REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS eq_score REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS eq_self_awareness REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS eq_self_regulation REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS eq_motivation REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS eq_empathy REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS eq_social REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS in_flow BOOLEAN DEFAULT FALSE;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS flow_intensity REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS flow_total_cycles BIGINT DEFAULT 0;
    -- Colonnes supplementaires psychologie
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS psyche_defense TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS maslow_priority_need TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS toltec_invocations BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS toltec_violations BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS shadow_leaking BOOLEAN DEFAULT FALSE;
    -- Volonte : colonnes cognitive_traces
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS will_data JSONB DEFAULT '{}';
    -- Apprentissages vectoriels (nn_learning)
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS nn_learnings_count INTEGER DEFAULT 0;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS nn_learning_data JSONB DEFAULT '{}';
    -- Volonte : colonnes metric_snapshots
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS willpower REAL DEFAULT 0.5;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS decision_fatigue REAL DEFAULT 0.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS total_deliberations BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS proud_decisions BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS regretted_decisions BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS deliberation_this_cycle BOOLEAN DEFAULT FALSE;
    -- Sommeil et subconscient : traces cognitives
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS is_sleeping BOOLEAN DEFAULT FALSE;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS sleep_phase TEXT;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS sleep_progress REAL;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS subconscious_activation REAL;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS subconscious_insight TEXT;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS subconscious_priming TEXT;
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS inner_monologue TEXT;
    -- Sommeil et subconscient : snapshots metriques
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS is_sleeping BOOLEAN DEFAULT FALSE;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS sleep_phase TEXT;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS sleep_pressure REAL;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS awake_cycles BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS subconscious_activation REAL;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS pending_associations INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS repressed_content_count INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS incubating_problems INTEGER DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS neural_connections_total BIGINT DEFAULT 0;

    -- GABA et Glutamate (2 molecules manquantes pour chimie complete 9D)
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS gaba REAL DEFAULT 0.5;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS glutamate REAL DEFAULT 0.45;

    -- Sensibilite des recepteurs (9 molecules)
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_dopamine_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_serotonin_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_noradrenaline_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_endorphin_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_oxytocin_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_adrenaline_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_cortisol_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_gaba_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS receptor_glutamate_sensitivity REAL DEFAULT 1.0;

    -- BDNF et matiere grise
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS bdnf_level REAL DEFAULT 0.5;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS neuroplasticity REAL DEFAULT 0.65;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS synaptic_density REAL DEFAULT 0.6;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS grey_matter_volume REAL DEFAULT 0.7;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS myelination REAL DEFAULT 0.6;

    -- Donnees recepteurs et BDNF dans les traces cognitives
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS receptor_data JSONB DEFAULT '{}';
    ALTER TABLE cognitive_traces ADD COLUMN IF NOT EXISTS bdnf_data JSONB DEFAULT '{}';

    -- Colonne vertebrale (spine)
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS spine_reflexes_triggered BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS spine_signals_processed BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS spine_reflex_sensitivity REAL DEFAULT 1.0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS spine_last_route TEXT DEFAULT '';

    -- Curiosite
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS curiosity_global REAL DEFAULT 0.3;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS curiosity_hungriest_domain TEXT DEFAULT '';
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS curiosity_total_discoveries BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS curiosity_cycles_since_discovery BIGINT DEFAULT 0;
    ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS curiosity_pending_questions INTEGER DEFAULT 0;
END $$;
