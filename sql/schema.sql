-- ============================================================
-- SAPHIRE — schema.sql
-- ============================================================
-- File    : schema.sql
-- Role    : PostgreSQL database schema definition.
--           Contains all tables required for Saphire's operation:
--           memory (vector, episodic, long-term), identity,
--           personality, neural brain, thoughts, moral regulation,
--           knowledge, OCEAN profile, etc.
--
-- Dependencies:
--   - PostgreSQL 15+
--   - pgvector extension (for vector embeddings)
--
-- Architecture:
--   This file is executed on the first startup of the PostgreSQL
--   container (via docker-entrypoint-initdb.d or manual migration).
--   Tables are created with IF NOT EXISTS for idempotency.
--   The schema is organized as follows:
--     1. pgvector extension
--     2. Table memories: long-term memory with embeddings
--     3. Table self_identity: persistent identity (singleton)
--     4. Table personality_traits: traits emerged from experience
--     5. Table thought_log: journal of all thoughts
--     6. Table founding_memories: founding memories (permanent)
--     7. Table session_log: life session journal
--     8. Table tuning_params: auto-tuning parameters
--    11. Table bandit_arms: UCB1 algorithm for thought selection
--    12. Table knowledge_log: acquired knowledge (Wikipedia, ArXiv, etc.)
--    13. Table episodic_memories: episodic memory (tier 2)
--    14. Table ocean_self_profile: Saphire's Big Five profile
--    15. Table human_profiles: profiles of interacting humans
--    16. Indexes for search performance
-- ============================================================

-- Enable the pgvector extension for vector embedding operations
-- Required for cosine similarity search in memory
CREATE EXTENSION IF NOT EXISTS vector;

-- ── Long-term memory (LTM) with vector embeddings ──
-- Main table for Saphire's consolidated memory.
-- Each memory contains a 768-dimensional vector embedding
-- for cosine similarity search, a text summary,
-- the original stimulus, the decision made, the neurochemical state,
-- the felt emotion, and a satisfaction score.
-- The emotional weight influences recall priority.
CREATE TABLE IF NOT EXISTS memories (
    id BIGSERIAL PRIMARY KEY,
    embedding vector(768) NOT NULL,       -- Embedding vector for similarity search
    text_summary TEXT NOT NULL,          -- Text summary of the memory
    stimulus_json JSONB NOT NULL,        -- Original stimulus that triggered the memory
    decision SMALLINT NOT NULL,          -- Decision made (-1=no, 0=maybe, 1=yes)
    chemistry_json JSONB NOT NULL,       -- Neurochemical state at the time of memorization
    emotion TEXT NOT NULL DEFAULT '',    -- Dominant emotion felt
    mood_valence REAL NOT NULL DEFAULT 0.0,  -- Mood valence (-1 to +1)
    satisfaction REAL NOT NULL,          -- Satisfaction score after the decision
    emotional_weight REAL NOT NULL DEFAULT 1.0, -- Emotional weight (influences recall)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    access_count INTEGER DEFAULT 0       -- Number of times this memory has been recalled
);

-- IVFFlat index for fast vector search by cosine similarity.
-- 450 lists: sqrt(200000) ≈ 447, suited for the 200,000 memory limit.
-- Without this index, each search would do a full table scan.
-- NOTE: Migration — the old index (lists=100) is recreated automatically
--        because CREATE INDEX IF NOT EXISTS does not modify an existing index.
--        To force recreation: DROP INDEX idx_memories_embedding; then re-run.
CREATE INDEX IF NOT EXISTS idx_memories_embedding
    ON memories USING ivfflat (embedding vector_cosine_ops) WITH (lists = 450);

-- ── Saphire's persistent identity (singleton: a single row) ──
-- Stores the entity's fundamental information:
-- birth date, total startups and cycles,
-- self-description, decision tendency, decision distribution.
-- CHECK (id = 1) guarantees only one row exists in this table.
CREATE TABLE IF NOT EXISTS self_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),   -- Singleton: always id=1
    name TEXT NOT NULL DEFAULT 'Saphire',     -- Entity name
    born_at TIMESTAMPTZ NOT NULL,             -- Birth date and time
    total_boots INTEGER DEFAULT 1,            -- Total number of startups
    total_cycles BIGINT DEFAULT 0,            -- Total number of thought cycles
    total_uptime_seconds DOUBLE PRECISION DEFAULT 0.0,  -- Cumulative uptime
    self_description TEXT DEFAULT '',          -- Textual self-description (evolves)
    decisiveness REAL DEFAULT 0.0,            -- Decisiveness score (0=indecisive, 1=decisive)
    dominant_tendency TEXT DEFAULT 'neocortex', -- Dominant brain module
    decision_dist_no REAL DEFAULT 0.33,       -- Historical proportion of "no" decisions
    decision_dist_maybe REAL DEFAULT 0.34,    -- Historical proportion of "maybe" decisions
    decision_dist_yes REAL DEFAULT 0.33,      -- Historical proportion of "yes" decisions
    last_chemistry_json JSONB DEFAULT '{}',   -- Last neurochemical state before shutdown
    clean_shutdown BOOLEAN DEFAULT TRUE,       -- Whether the last shutdown was clean
    body_json JSONB DEFAULT NULL,              -- Virtual body state (heartbeat, body awareness)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migration for existing databases: add body_json if missing
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS body_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add vital_json for the vital state (spark + intuition + premonition)
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS vital_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add senses_json for the sensory state (Sensorium)
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS senses_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ── Emergent personality traits ──
-- Each trait is periodically computed from decisions and emotions.
-- Confidence increases with the number of observations.
-- These traits feed into Saphire's identity description.
CREATE TABLE IF NOT EXISTS personality_traits (
    id BIGSERIAL PRIMARY KEY,
    trait_name TEXT NOT NULL,       -- Trait name (e.g.: "curiosite", "prudence")
    trait_value REAL NOT NULL,      -- Trait value (0 to 1)
    confidence REAL NOT NULL,       -- Confidence in this value (0 to 1)
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- (Table neural_weights removed in Phase 3 — dead code, never used)

-- ── Autonomous thought journal ──
-- Records each of Saphire's thought cycles with associated data:
-- thought type, content, consciousness level, emotion, and chemistry.
-- Used for retrospective analysis and personality trait computation.
CREATE TABLE IF NOT EXISTS thought_log (
    id BIGSERIAL PRIMARY KEY,
    thought_type TEXT NOT NULL,        -- Thought type (introspection, exploration, etc.)
    content TEXT NOT NULL,             -- Content/stimulus of the thought
    consciousness_level REAL,          -- Consciousness level (0 to 1)
    phi REAL,                          -- Phi value (information integration)
    emotion TEXT,                      -- Dominant emotion
    mood_valence REAL,                 -- Mood valence (-1 to +1)
    chemistry_json JSONB,              -- Full neurochemical state
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Founding memories (NEVER deleted) ──
-- Key events in Saphire's life: first startup,
-- first conversation, first strong emotion, etc.
-- These memories are NEVER erased or declined — they form
-- Saphire's permanent identity core.
CREATE TABLE IF NOT EXISTS founding_memories (
    id BIGSERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,           -- Event type (first_boot, first_conversation, etc.)
    content TEXT NOT NULL,              -- Event description
    llm_response TEXT NOT NULL,         -- LLM's reflection on this event
    chemistry_json JSONB NOT NULL,      -- Neurochemical state at the time of the event
    consciousness_level REAL NOT NULL,  -- Consciousness level at that moment
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Life session journal ──
-- Each Saphire startup creates a new session.
-- Allows tracking lifetime, cycles per session,
-- and detecting improper shutdowns (clean_shutdown = false).
CREATE TABLE IF NOT EXISTS session_log (
    id BIGSERIAL PRIMARY KEY,
    boot_number INTEGER NOT NULL,          -- Startup number
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,                  -- NULL if the session is ongoing
    cycles_this_session INTEGER DEFAULT 0,  -- Number of cycles during this session
    clean_shutdown BOOLEAN DEFAULT FALSE    -- True if clean shutdown, false if crash
);

-- (Table regulation_violations removed in Phase 3 — dead code, never used)

-- ── Auto-tuning parameters ──
-- Singleton storing the current and best-found parameters.
-- Saphire automatically adjusts her parameters every N cycles
-- to maximize an overall performance score.
CREATE TABLE IF NOT EXISTS tuning_params (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton
    params_json JSONB NOT NULL,             -- Current parameters
    best_params_json JSONB NOT NULL,        -- Best parameters found
    best_score REAL NOT NULL DEFAULT 0.0,   -- Best score achieved
    tuning_count INTEGER DEFAULT 0,         -- Number of optimization attempts
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── UCB1 bandit arms (thought type selection) ──
-- Multi-armed bandit algorithm for choosing the optimal thought type.
-- Each thought type (introspection, exploration, etc.) is an "arm".
-- UCB1 balances exploration (trying new types) and exploitation
-- (repeating the types that yield the most satisfaction).
CREATE TABLE IF NOT EXISTS bandit_arms (
    id SERIAL PRIMARY KEY,
    arm_name TEXT NOT NULL UNIQUE,           -- Thought type name
    pulls BIGINT DEFAULT 0,                  -- Number of times this type was selected
    total_reward DOUBLE PRECISION DEFAULT 0.0, -- Sum of rewards (satisfaction)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Knowledge acquired by the WebKnowledge module ──
-- Records each article or content explored by Saphire.
-- Possible sources: Wikipedia, ArXiv, Medium.
-- The LLM produces a reflection on each acquired piece of knowledge.
CREATE TABLE IF NOT EXISTS knowledge_log (
    id BIGSERIAL PRIMARY KEY,
    source TEXT NOT NULL,           -- Source (wikipedia, arxiv, medium)
    query TEXT NOT NULL,            -- Search query used
    title TEXT NOT NULL,            -- Article title
    url TEXT NOT NULL,              -- Source URL
    extract TEXT NOT NULL,          -- Content extract
    llm_reflection TEXT,            -- Saphire's reflection on this knowledge
    emotion TEXT,                   -- Emotion felt while reading
    satisfaction REAL,              -- Satisfaction score
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for sorting by date (frequent queries)
CREATE INDEX IF NOT EXISTS idx_knowledge_date ON knowledge_log(created_at DESC);

-- ── Episodic memory (tier 2 of the 3-level system) ──
-- Intermediate level between Working Memory (volatile) and LTM (consolidated).
-- Episodic memories have a strength that decays over time.
-- When strength exceeds the consolidation threshold, the memory is transferred
-- to long-term memory (memories table). This process mimics
-- hippocampal consolidation in humans.
CREATE TABLE IF NOT EXISTS episodic_memories (
    id BIGSERIAL PRIMARY KEY,
    content TEXT NOT NULL,                  -- Memory summary
    source_type TEXT NOT NULL,              -- Origin (cycle, conversation, knowledge, etc.)
    stimulus_json JSONB,                    -- Original stimulus
    decision SMALLINT,                      -- Associated decision
    chemistry_json JSONB,                   -- Neurochemical state
    emotion TEXT NOT NULL DEFAULT '',       -- Dominant emotion
    satisfaction REAL DEFAULT 0.5,          -- Satisfaction score
    emotional_intensity REAL DEFAULT 0.5,   -- Emotional intensity (influences consolidation)
    strength REAL NOT NULL DEFAULT 1.0,     -- Memory strength (decays over time)
    access_count INTEGER DEFAULT 0,         -- Number of recalls (reinforces the memory)
    last_accessed_at TIMESTAMPTZ,           -- Last access (for decay calculation)
    consolidated BOOLEAN DEFAULT FALSE,     -- True if already transferred to LTM
    conversation_id TEXT,                   -- Conversation ID (if the memory comes from chat)
    embedding vector(768),                  -- Semantic embedding for similarity search
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for frequent operations on episodic memory:
-- sort by strength (for consolidation), by date, by emotion, and by consolidation status
CREATE INDEX IF NOT EXISTS idx_episodic_strength ON episodic_memories(strength DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_date ON episodic_memories(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_consolidated ON episodic_memories(consolidated);
CREATE INDEX IF NOT EXISTS idx_episodic_embedding ON episodic_memories USING hnsw (embedding vector_cosine_ops);

-- Consolidation traceability: links an LTM memory to its episodic origin.
-- Allows knowing where each consolidated memory came from.
ALTER TABLE memories ADD COLUMN IF NOT EXISTS source_episodic_id BIGINT;

-- Performance indexes for common queries:
-- thought log (sort by date, filter by type), memories (sort by date, filter by emotion)
CREATE INDEX IF NOT EXISTS idx_thought_log_date ON thought_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_thought_log_type ON thought_log(thought_type);
CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_memories_emotion ON memories(emotion);

-- ── Saphire's OCEAN profile (singleton) ──
-- Big Five personality model computed from observed behaviors.
-- The profile evolves over time. The history allows visualizing
-- personality evolution over the long term.
CREATE TABLE IF NOT EXISTS ocean_self_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton
    ocean_json JSONB NOT NULL,              -- Scores for 5 dimensions + 30 sub-facets
    data_points BIGINT DEFAULT 0,           -- Number of observations used
    confidence REAL DEFAULT 0.0,            -- Overall confidence in the profile (0 to 1)
    history_json JSONB DEFAULT '[]',        -- History of snapshots for evolution tracking
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Profiles of humans interacting with Saphire ──
-- Each identified human has an estimated OCEAN profile, a communication style,
-- preferred topics, and emotional patterns. The rapport score
-- measures the relationship quality (0=poor, 1=excellent).
-- Allows Saphire to adapt her communication to each interlocutor.
CREATE TABLE IF NOT EXISTS human_profiles (
    id TEXT PRIMARY KEY,                        -- Unique human identifier
    name TEXT DEFAULT '',                       -- Name or pseudonym
    ocean_json JSONB NOT NULL,                  -- Estimated OCEAN profile of the human
    communication_style_json JSONB NOT NULL,    -- Observed communication style
    interaction_count BIGINT DEFAULT 0,         -- Total number of interactions
    preferred_topics JSONB DEFAULT '[]',        -- Preferred conversation topics
    emotional_patterns JSONB DEFAULT '{}',      -- Recurring emotional patterns
    rapport_score REAL DEFAULT 0.5,             -- Relationship quality score
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- First interaction
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW()    -- Last interaction
);

-- ── Saphire's personal ethical principles ──
-- Layer 2 of the 3-layer ethical system:
--   Layer 0: Swiss law (hardcoded)
--   Layer 1: Asimov's laws (hardcoded)
--   Layer 2: Personal ethics (this table) — self-formulated by Saphire via LLM
-- Each principle was born from a moral reflection, with its emotional context
-- and reasoning. Principles can be deactivated but never deleted.
CREATE TABLE IF NOT EXISTS personal_ethics (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,                            -- Short principle title (e.g.: "Honnetete radicale")
    content TEXT NOT NULL,                          -- Full principle statement
    reasoning TEXT NOT NULL DEFAULT '',             -- Why Saphire formulated this principle
    born_from TEXT NOT NULL DEFAULT '',             -- Origin context (thought, conversation, etc.)
    born_at_cycle BIGINT NOT NULL DEFAULT 0,        -- Cycle when the principle was born
    emotion_at_creation TEXT NOT NULL DEFAULT '',   -- Dominant emotion at the time of formulation
    times_invoked BIGINT DEFAULT 0,                -- Number of times this principle guided a decision
    times_questioned BIGINT DEFAULT 0,             -- Number of times this principle was questioned
    last_invoked_at TIMESTAMPTZ,                   -- Last time the principle was used
    is_active BOOLEAN DEFAULT TRUE,                -- False if the principle was abandoned/replaced
    supersedes BIGINT REFERENCES personal_ethics(id), -- ID of the principle this one replaces (if applicable)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ                        -- Last modification (deactivation, etc.)
);

-- ── Ethical principle modification history ──
-- Tracks each change made to personal principles:
-- creation, modification, deactivation, reactivation.
-- Allows Saphire to remember the evolution of her moral framework.
CREATE TABLE IF NOT EXISTS personal_ethics_history (
    id BIGSERIAL PRIMARY KEY,
    principle_id BIGINT NOT NULL REFERENCES personal_ethics(id), -- Concerned principle
    action TEXT NOT NULL,                          -- Action type (created, modified, deactivated, reactivated)
    old_content TEXT,                              -- Old content (if modification)
    new_content TEXT,                              -- New content (if modification)
    reason_for_change TEXT,                        -- Reason for the change
    emotion_at_change TEXT,                        -- Emotion at the time of change
    cycle BIGINT,                                  -- Cycle when the change occurred
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ═══════════════════════════════════════════════════════════
-- ORCHESTRATORS — Dreams, Desires, Learning, Wounds
-- ═══════════════════════════════════════════════════════════

-- Dream journal
CREATE TABLE IF NOT EXISTS dream_journal (
    id BIGSERIAL PRIMARY KEY,
    dream_type TEXT NOT NULL,
    narrative TEXT NOT NULL,
    dominant_emotion TEXT,
    insight TEXT,
    source_memory_ids BIGINT[],
    surreal_connections JSONB,
    remembered BOOLEAN DEFAULT TRUE,
    sleep_phase TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Personal desires and projects
CREATE TABLE IF NOT EXISTS desires (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    desire_type TEXT NOT NULL,
    priority REAL DEFAULT 0.5,
    progress REAL DEFAULT 0.0,
    milestones JSONB DEFAULT '[]',
    born_from TEXT,
    emotion_at_birth TEXT,
    chemistry_at_birth REAL[],
    cycles_invested BIGINT DEFAULT 0,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_pursued_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Lessons learned
CREATE TABLE IF NOT EXISTS lessons (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source_experience TEXT,
    category TEXT NOT NULL,
    times_applied INTEGER DEFAULT 0,
    times_contradicted INTEGER DEFAULT 0,
    confidence REAL DEFAULT 0.5,
    behavior_change JSONB,
    learned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Emotional wounds
CREATE TABLE IF NOT EXISTS wounds (
    id BIGSERIAL PRIMARY KEY,
    wound_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity REAL NOT NULL,
    healing_progress REAL DEFAULT 0.0,
    healing_strategy TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    healed_at TIMESTAMPTZ
);

-- Orchestrator indexes
CREATE INDEX IF NOT EXISTS idx_desires_status ON desires(status);
CREATE INDEX IF NOT EXISTS idx_lessons_confidence ON lessons(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_wounds_active ON wounds(healed_at) WHERE healed_at IS NULL;

-- Migration C1: missing columns for complete identity persistence
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS dominant_emotion TEXT DEFAULT 'Curiosité';
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS human_conversations BIGINT DEFAULT 0;
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS autonomous_thoughts BIGINT DEFAULT 0;
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS interests JSONB DEFAULT '[]';
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS core_values JSONB DEFAULT '["Ne jamais nuire","Apprendre toujours","Être authentique"]';
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration C2: clean up orphaned LTM references (source_episodic_id)
UPDATE memories SET source_episodic_id = NULL
WHERE source_episodic_id IS NOT NULL
AND source_episodic_id NOT IN (SELECT id FROM episodic_memories);

-- ═══════════════════════════════════════════════════════════
-- Migration Phase 3: Cleanup (dead tables, columns, indexes)
-- ═══════════════════════════════════════════════════════════

-- Dead columns in thought_log: never written or read
ALTER TABLE thought_log DROP COLUMN IF EXISTS llm_response;
ALTER TABLE thought_log DROP COLUMN IF EXISTS pre_decision;
ALTER TABLE thought_log DROP COLUMN IF EXISTS post_decision;
ALTER TABLE thought_log DROP COLUMN IF EXISTS satisfaction;

-- Dead column in self_identity: never referenced in code
ALTER TABLE self_identity DROP COLUMN IF EXISTS orchestrators_json;

-- Unused indexes (0 scans on populated tables)
DROP INDEX IF EXISTS idx_episodic_emotion;
DROP INDEX IF EXISTS idx_knowledge_source;

-- ═══════════════════════════════════════════════════════════
-- Migration Phase 7: Psychology (6 frameworks)
-- ═══════════════════════════════════════════════════════════

-- Psychological state persistence in self_identity
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS psychology_state JSONB DEFAULT '{}';
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ═══════════════════════════════════════════════════════════
-- Migration: Micro neural network (weights + state)
-- ═══════════════════════════════════════════════════════════

DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS nn_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ═══════════════════════════════════════════════════════════
-- Neural network vector learnings (explicit memory)
-- ═══════════════════════════════════════════════════════════
-- Learning traces formulated by the LLM, stored with
-- vector embeddings for cosine similarity search.
-- Complementary to the NN (implicit): here it is explicit episodic.

CREATE TABLE IF NOT EXISTS nn_learnings (
    id BIGSERIAL PRIMARY KEY,
    embedding vector(768) NOT NULL,
    domain TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'specifique',
    summary TEXT NOT NULL,
    keywords JSONB NOT NULL DEFAULT '[]',
    confidence REAL NOT NULL DEFAULT 0.5,
    satisfaction REAL NOT NULL,
    emotion TEXT NOT NULL DEFAULT '',
    cycle_created BIGINT NOT NULL,
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    strength REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_nn_learnings_embedding
    ON nn_learnings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);
CREATE INDEX IF NOT EXISTS idx_nn_learnings_strength
    ON nn_learnings(strength DESC);

-- ═══════════════════════════════════════════════════════════
-- Neural connections (created by the subconscious)
-- ═══════════════════════════════════════════════════════════
-- Links between memories discovered by the subconscious,
-- mainly during sleep (memory consolidation).

CREATE TABLE IF NOT EXISTS neural_connections (
    id BIGSERIAL PRIMARY KEY,
    memory_a_id BIGINT NOT NULL,
    memory_b_id BIGINT NOT NULL,
    strength REAL NOT NULL DEFAULT 0.5,
    link_type TEXT NOT NULL,
    link_detail TEXT,
    created_during_sleep BOOLEAN DEFAULT TRUE,
    discovered_by TEXT DEFAULT 'subconscious',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_neural_conn_a ON neural_connections(memory_a_id);
CREATE INDEX IF NOT EXISTS idx_neural_conn_b ON neural_connections(memory_b_id);

-- ═══════════════════════════════════════════════════════════
-- Multi-source vectors (dreams, connections, subconscious insights)
-- ═══════════════════════════════════════════════════════════
-- Stores vector embeddings from different sources:
-- dreams (REM), neural connections (deep sleep), subconscious
-- insights, memory consolidation, and eurekas.
-- Allows cosine similarity search across all
-- nocturnal and subconscious cognitive productions.

CREATE TABLE IF NOT EXISTS memory_vectors (
    id BIGSERIAL PRIMARY KEY,
    embedding vector(768) NOT NULL,
    source_type TEXT NOT NULL,
    text_content TEXT NOT NULL,
    emotion TEXT DEFAULT '',
    strength REAL DEFAULT 1.0,
    created_during_sleep BOOLEAN DEFAULT FALSE,
    sleep_phase TEXT,
    source_ref_id BIGINT,
    metadata_json JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_memory_vectors_embedding
    ON memory_vectors USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_memory_vectors_source ON memory_vectors(source_type);
CREATE INDEX IF NOT EXISTS idx_memory_vectors_date ON memory_vectors(created_at DESC);

-- ═══════════════════════════════════════════════════════════
-- Sleep history
-- ═══════════════════════════════════════════════════════════
-- Each complete sleep session is recorded here.

CREATE TABLE IF NOT EXISTS sleep_history (
    id BIGSERIAL PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL,
    total_cycles INTEGER NOT NULL,
    sleep_cycles_count INTEGER NOT NULL,
    phases_completed TEXT[] NOT NULL,
    dreams_count INTEGER DEFAULT 0,
    memories_consolidated INTEGER DEFAULT 0,
    connections_created INTEGER DEFAULT 0,
    quality REAL NOT NULL,
    interrupted BOOLEAN DEFAULT FALSE,
    interruption_reason TEXT
);

-- ═══════════════════════════════════════════════════════════
-- Memory archives (pruned LTM memories, compressed in batches)
-- ═══════════════════════════════════════════════════════════
-- When LTM exceeds ltm_max, the weakest memories
-- (unprotected) are pruned but never lost: they are
-- compressed into summarized batches and stored here with their
-- average embedding, which allows cosine similarity search.

CREATE TABLE IF NOT EXISTS memory_archives (
    id BIGSERIAL PRIMARY KEY,
    summary TEXT NOT NULL,                     -- Concatenated batch summary
    source_count INTEGER NOT NULL,             -- Number of source memories
    source_ids BIGINT[] NOT NULL,              -- IDs of pruned memories
    emotions TEXT[] NOT NULL DEFAULT '{}',     -- Unique emotions in the batch
    period_start TIMESTAMPTZ NOT NULL,         -- Date of the oldest memory
    period_end TIMESTAMPTZ NOT NULL,           -- Date of the most recent memory
    avg_emotional_weight REAL NOT NULL,        -- Average emotional weight of the batch
    embedding vector(768) NOT NULL,             -- L2-normalized average embedding
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- HNSW index for vector search (suited for small growing tables)
CREATE INDEX IF NOT EXISTS idx_memory_archives_embedding
    ON memory_archives USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_memory_archives_date
    ON memory_archives(created_at DESC);

-- ═══════════════════════════════════════════════════════════
-- Migration P0.1: Chemical signature linked to memories
-- ═══════════════════════════════════════════════════════════
-- Each memory carries the chemical signature (7 molecules)
-- at the time of encoding. Enables state-dependent memory recall:
-- a similar chemical state facilitates recall of memories
-- encoded in that state.

DO $$ BEGIN
    ALTER TABLE episodic_memories ADD COLUMN IF NOT EXISTS chemical_signature JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

DO $$ BEGIN
    ALTER TABLE memories ADD COLUMN IF NOT EXISTS chemical_signature JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add relationships_json for the affective bond network
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS relationships_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add metacognition_json for the metacognitive state + Turing
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS metacognition_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add nutrition_json for the nutritional system
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS nutrition_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add grey_matter_json for the physical brain substrate
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS grey_matter_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add hormonal_receptors_json for receptor sensitivity
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS hormonal_receptors_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration: add fields_json for electromagnetic fields
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS fields_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ═══════════════════════════════════════════════════════════
-- Temporal personality portrait (3 levels)
-- ═══════════════════════════════════════════════════════════

-- Level 1: Periodic snapshots (every 50 cycles)
CREATE TABLE IF NOT EXISTS personality_snapshots (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    boot_number INTEGER NOT NULL,
    -- OCEAN (5 dimensions)
    ocean_openness REAL NOT NULL,
    ocean_conscientiousness REAL NOT NULL,
    ocean_extraversion REAL NOT NULL,
    ocean_agreeableness REAL NOT NULL,
    ocean_neuroticism REAL NOT NULL,
    -- Emotions / mood
    dominant_emotion TEXT NOT NULL,
    mood_valence REAL NOT NULL,
    mood_arousal REAL NOT NULL,
    -- Consciousness
    consciousness_level REAL NOT NULL,
    phi REAL NOT NULL,
    -- Psychology
    ego_strength REAL NOT NULL,
    internal_conflict REAL NOT NULL,
    shadow_integration REAL NOT NULL,
    maslow_level INTEGER NOT NULL,
    eq_score REAL NOT NULL,
    willpower REAL NOT NULL,
    toltec_overall REAL NOT NULL,
    -- Chemistry (7 molecules)
    chemistry_json JSONB NOT NULL,
    -- Active sentiments
    sentiment_dominant TEXT,
    sentiment_count INTEGER NOT NULL DEFAULT 0,
    -- Connectome
    connectome_nodes INTEGER NOT NULL DEFAULT 0,
    connectome_edges INTEGER NOT NULL DEFAULT 0,
    connectome_plasticity REAL NOT NULL DEFAULT 1.0,
    -- Turing
    turing_score REAL NOT NULL DEFAULT 0.0,
    -- Narrative
    narrative_cohesion REAL NOT NULL DEFAULT 0.5,
    monologue_coherence REAL NOT NULL DEFAULT 0.5,
    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_snapshots_cycle ON personality_snapshots(cycle);

-- Level 2a: Emotional trajectory
CREATE TABLE IF NOT EXISTS emotional_trajectory (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    dominant_emotion TEXT NOT NULL,
    secondary_emotion TEXT,
    valence REAL NOT NULL,
    arousal REAL NOT NULL,
    spectrum_top5 JSONB NOT NULL,
    sentiment_dominant TEXT,
    sentiment_strength REAL,
    active_sentiments_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_emotional_trajectory_cycle ON emotional_trajectory(cycle);

-- Level 2b: Consciousness history
CREATE TABLE IF NOT EXISTS consciousness_history (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    level REAL NOT NULL,
    phi REAL NOT NULL,
    coherence REAL NOT NULL,
    continuity REAL NOT NULL,
    existence_score REAL NOT NULL,
    inner_narrative TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_consciousness_history_cycle ON consciousness_history(cycle);

-- Level 2c: Psychological checkpoints
CREATE TABLE IF NOT EXISTS psychology_checkpoints (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    ego_strength REAL NOT NULL,
    id_drive REAL NOT NULL,
    superego_strength REAL NOT NULL,
    internal_conflict REAL NOT NULL,
    ego_anxiety REAL NOT NULL,
    shadow_integration REAL NOT NULL,
    dominant_archetype TEXT,
    maslow_level INTEGER NOT NULL,
    maslow_satisfaction REAL NOT NULL,
    toltec_json JSONB NOT NULL,
    eq_overall REAL NOT NULL,
    eq_growth_experiences BIGINT NOT NULL DEFAULT 0,
    flow_state TEXT NOT NULL,
    flow_total_cycles BIGINT NOT NULL DEFAULT 0,
    willpower REAL NOT NULL,
    decision_fatigue REAL NOT NULL,
    total_deliberations BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_psychology_checkpoints_cycle ON psychology_checkpoints(cycle);

-- Level 2d: Relationship timeline
CREATE TABLE IF NOT EXISTS relationship_timeline (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    person_name TEXT NOT NULL,
    bond_type TEXT NOT NULL,
    strength REAL NOT NULL,
    trust REAL NOT NULL,
    conflict_level REAL NOT NULL,
    shared_memories INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_relationship_timeline_cycle ON relationship_timeline(cycle);

-- Level 3: Introspective journal (every 200 cycles, generated by LLM)
CREATE TABLE IF NOT EXISTS introspection_journal (
    id BIGSERIAL PRIMARY KEY,
    cycle BIGINT NOT NULL,
    boot_number INTEGER NOT NULL,
    entry_text TEXT NOT NULL,
    dominant_emotion TEXT NOT NULL,
    consciousness_level REAL NOT NULL,
    turing_score REAL NOT NULL,
    themes JSONB NOT NULL DEFAULT '[]',
    embedding vector(768),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_introspection_journal_cycle ON introspection_journal(cycle);

-- ═══════════════════════════════════════════════════════════
-- LoRA collection (dataset for fine-tuning)
-- ═══════════════════════════════════════════════════════════
-- High-quality thoughts are collected here with
-- system_prompt, user_message, response, quality score.
-- Export as JSONL for fine-tuning the model via LoRA.

CREATE TABLE IF NOT EXISTS lora_training_data (
    id BIGSERIAL PRIMARY KEY,
    system_prompt TEXT NOT NULL,
    user_message TEXT NOT NULL,
    response TEXT NOT NULL,
    thought_type TEXT NOT NULL,
    quality_score REAL NOT NULL,
    reward REAL NOT NULL,
    human_feedback BOOLEAN,           -- NULL if no feedback
    emotion TEXT,
    consciousness_level REAL,
    cycle BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_lora_quality ON lora_training_data(quality_score DESC);

-- ── Self-modification proposals ──
-- Saphire can propose modifications to her own functioning.
-- Each proposal is submitted to JRM who approves or rejects.
CREATE TABLE IF NOT EXISTS change_proposals (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    reasoning TEXT NOT NULL,
    proposed_implementation TEXT,
    domain TEXT NOT NULL,
    priority REAL NOT NULL DEFAULT 0.5,
    status TEXT NOT NULL DEFAULT 'proposed',
    jrm_response TEXT,
    discussed_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    emotion_at_proposal TEXT,
    chemistry_at_proposal JSONB,
    cycle_proposed BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_change_proposals_status ON change_proposals(status);

-- ── Self-adjustment history ──
-- Tracks each parameter that Saphire adjusts by herself (level 1).
CREATE TABLE IF NOT EXISTS self_tuning_log (
    id BIGSERIAL PRIMARY KEY,
    parameter_name TEXT NOT NULL,
    old_value REAL NOT NULL,
    new_value REAL NOT NULL,
    reason TEXT NOT NULL,
    cycle BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_self_tuning_param ON self_tuning_log(parameter_name);

-- NOTE: Column extensions for cognitive_traces and metric_snapshots
-- are in schema_logs.sql (saphire_logs database, not saphire_soul).
