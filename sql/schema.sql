-- ============================================================
-- SAPHIRE — schema.sql
-- ============================================================
-- Fichier : schema.sql
-- Role    : Definition du schema de base de donnees PostgreSQL.
--           Contient toutes les tables necessaires au fonctionnement
--           de Saphire : memoire (vectorielle, episodique, long terme),
--           identite, personnalite, cerveau neuronal, pensees,
--           regulation morale, connaissances, profil OCEAN, etc.
--
-- Dependances :
--   - PostgreSQL 15+
--   - Extension pgvector (pour les embeddings vectoriels)
--
-- Architecture :
--   Ce fichier est execute au premier demarrage du conteneur
--   PostgreSQL (via docker-entrypoint-initdb.d ou migration manuelle).
--   Les tables sont creees avec IF NOT EXISTS pour l'idempotence.
--   Le schema se decompose en :
--     1. Extension pgvector
--     2. Table memories : memoire a long terme avec embeddings
--     3. Table self_identity : identite persistante (singleton)
--     4. Table personality_traits : traits emerges de l'experience
--     5. Table thought_log : journal de toutes les pensees
--     6. Table founding_memories : souvenirs fondateurs (permanents)
--     7. Table session_log : journal des sessions de vie
--     8. Table tuning_params : parametres d'auto-optimisation
--    11. Table bandit_arms : algorithme UCB1 pour la selection de pensees
--    12. Table knowledge_log : connaissances acquises (Wikipedia, ArXiv, etc.)
--    13. Table episodic_memories : memoire episodique (tier 2)
--    14. Table ocean_self_profile : profil Big Five de Saphire
--    15. Table human_profiles : profils des humains interagissant
--    16. Index pour les performances de recherche
-- ============================================================

-- Active l'extension pgvector pour les operations sur les embeddings vectoriels
-- Necessaire pour la recherche de similarite par cosinus dans la memoire
CREATE EXTENSION IF NOT EXISTS vector;

-- ── Memoire a long terme (LTM) avec embeddings vectoriels ──
-- Table principale de la memoire consolidee de Saphire.
-- Chaque souvenir contient un embedding vectoriel de 768 dimensions
-- pour la recherche par similarite cosinus, un resume textuel,
-- le stimulus d'origine, la decision prise, l'etat neurochimique,
-- l'emotion ressentie et un score de satisfaction.
-- Le poids emotionnel influence la priorite de rappel.
CREATE TABLE IF NOT EXISTS memories (
    id BIGSERIAL PRIMARY KEY,
    embedding vector(768) NOT NULL,       -- Vecteur d'embedding pour la recherche par similarite
    text_summary TEXT NOT NULL,          -- Resume textuel du souvenir
    stimulus_json JSONB NOT NULL,        -- Stimulus d'origine ayant declenche le souvenir
    decision SMALLINT NOT NULL,          -- Decision prise (-1=non, 0=peut-etre, 1=oui)
    chemistry_json JSONB NOT NULL,       -- Etat neurochimique au moment de la memorisation
    emotion TEXT NOT NULL DEFAULT '',    -- Emotion dominante ressentie
    mood_valence REAL NOT NULL DEFAULT 0.0,  -- Valence de l'humeur (-1 a +1)
    satisfaction REAL NOT NULL,          -- Score de satisfaction apres la decision
    emotional_weight REAL NOT NULL DEFAULT 1.0, -- Poids emotionnel (influence le rappel)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    access_count INTEGER DEFAULT 0       -- Nombre de fois que ce souvenir a ete rappele
);

-- Index IVFFlat pour la recherche vectorielle rapide par similarite cosinus.
-- 450 listes : sqrt(200000) ≈ 447, adapte a la nouvelle limite de 200 000 souvenirs.
-- Sans cet index, chaque recherche ferait un scan complet de la table.
-- NOTE : Migration — l'ancien index (lists=100) est recree automatiquement
--        car CREATE INDEX IF NOT EXISTS ne modifie pas un index existant.
--        Pour forcer la recreation : DROP INDEX idx_memories_embedding; puis relancer.
CREATE INDEX IF NOT EXISTS idx_memories_embedding
    ON memories USING ivfflat (embedding vector_cosine_ops) WITH (lists = 450);

-- ── Identite persistante de Saphire (singleton : une seule ligne) ──
-- Stocke les informations fondamentales de l'entite :
-- date de naissance, nombre total de demarrages et cycles,
-- description de soi, tendance decisionnelle, distribution des decisions.
-- CHECK (id = 1) garantit qu'il n'y a qu'une seule ligne dans cette table.
CREATE TABLE IF NOT EXISTS self_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),   -- Singleton : toujours id=1
    name TEXT NOT NULL DEFAULT 'Saphire',     -- Nom de l'entite
    born_at TIMESTAMPTZ NOT NULL,             -- Date et heure de naissance
    total_boots INTEGER DEFAULT 1,            -- Nombre total de demarrages
    total_cycles BIGINT DEFAULT 0,            -- Nombre total de cycles de pensee
    total_uptime_seconds DOUBLE PRECISION DEFAULT 0.0,  -- Temps de fonctionnement cumule
    self_description TEXT DEFAULT '',          -- Description textuelle de soi (evolue)
    decisiveness REAL DEFAULT 0.0,            -- Score de determination (0=indecis, 1=determine)
    dominant_tendency TEXT DEFAULT 'neocortex', -- Module cerebral dominant
    decision_dist_no REAL DEFAULT 0.33,       -- Proportion historique de decisions "non"
    decision_dist_maybe REAL DEFAULT 0.34,    -- Proportion historique de decisions "peut-etre"
    decision_dist_yes REAL DEFAULT 0.33,      -- Proportion historique de decisions "oui"
    last_chemistry_json JSONB DEFAULT '{}',   -- Dernier etat neurochimique avant arret
    clean_shutdown BOOLEAN DEFAULT TRUE,       -- Si le dernier arret etait propre
    body_json JSONB DEFAULT NULL,              -- Etat du corps virtuel (battements, conscience corporelle)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migration pour les bases existantes : ajouter body_json si absente
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS body_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter vital_json pour l'etat vital (spark + intuition + premonition)
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS vital_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter senses_json pour l'etat sensoriel (Sensorium)
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS senses_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ── Traits de personnalite emergents ──
-- Chaque trait est calcule periodiquement a partir des decisions et emotions.
-- La confiance augmente avec le nombre d'observations.
-- Ces traits alimentent la description identitaire de Saphire.
CREATE TABLE IF NOT EXISTS personality_traits (
    id BIGSERIAL PRIMARY KEY,
    trait_name TEXT NOT NULL,       -- Nom du trait (ex: "curiosite", "prudence")
    trait_value REAL NOT NULL,      -- Valeur du trait (0 a 1)
    confidence REAL NOT NULL,       -- Confiance dans cette valeur (0 a 1)
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- (Table neural_weights supprimee en Phase 3 — code mort, jamais utilisee)

-- ── Journal des pensees autonomes ──
-- Enregistre chaque cycle de pensee de Saphire avec les donnees
-- associees : type de pensee, contenu, niveau de conscience, emotion et chimie.
-- Sert a l'analyse retrospective et au calcul des traits de personnalite.
CREATE TABLE IF NOT EXISTS thought_log (
    id BIGSERIAL PRIMARY KEY,
    thought_type TEXT NOT NULL,        -- Type de pensee (introspection, exploration, etc.)
    content TEXT NOT NULL,             -- Contenu/stimulus de la pensee
    consciousness_level REAL,          -- Niveau de conscience (0 a 1)
    phi REAL,                          -- Valeur Phi (integration d'information)
    emotion TEXT,                      -- Emotion dominante
    mood_valence REAL,                 -- Valence de l'humeur (-1 a +1)
    chemistry_json JSONB,              -- Etat neurochimique complet
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Souvenirs fondateurs (JAMAIS supprimes) ──
-- Evenements cles de la vie de Saphire : premier demarrage,
-- premiere conversation, premiere emotion forte, etc.
-- Ces souvenirs ne sont JAMAIS effaces ni declines — ils forment
-- le noyau identitaire permanent de Saphire.
CREATE TABLE IF NOT EXISTS founding_memories (
    id BIGSERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,           -- Type d'evenement (first_boot, first_conversation, etc.)
    content TEXT NOT NULL,              -- Description de l'evenement
    llm_response TEXT NOT NULL,         -- Reflexion du LLM sur cet evenement
    chemistry_json JSONB NOT NULL,      -- Etat neurochimique au moment de l'evenement
    consciousness_level REAL NOT NULL,  -- Niveau de conscience a cet instant
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Journal des sessions de vie ──
-- Chaque demarrage de Saphire cree une nouvelle session.
-- Permet de suivre la duree de vie, les cycles par session,
-- et de detecter les arrets impropres (clean_shutdown = false).
CREATE TABLE IF NOT EXISTS session_log (
    id BIGSERIAL PRIMARY KEY,
    boot_number INTEGER NOT NULL,          -- Numero de demarrage
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,                  -- NULL si la session est en cours
    cycles_this_session INTEGER DEFAULT 0,  -- Nombre de cycles pendant cette session
    clean_shutdown BOOLEAN DEFAULT FALSE    -- True si arret propre, false si crash
);

-- (Table regulation_violations supprimee en Phase 3 — code mort, jamais utilisee)

-- ── Parametres d'auto-optimisation (tuning) ──
-- Singleton qui stocke les parametres courants et les meilleurs trouves.
-- Saphire ajuste automatiquement ses parametres tous les N cycles
-- pour maximiser un score de performance global.
CREATE TABLE IF NOT EXISTS tuning_params (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton
    params_json JSONB NOT NULL,             -- Parametres actuels
    best_params_json JSONB NOT NULL,        -- Meilleurs parametres trouves
    best_score REAL NOT NULL DEFAULT 0.0,   -- Meilleur score obtenu
    tuning_count INTEGER DEFAULT 0,         -- Nombre de tentatives d'optimisation
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Bras du bandit UCB1 (selection des types de pensees) ──
-- Algorithme de bandit manchot pour choisir le type de pensee optimal.
-- Chaque type de pensee (introspection, exploration, etc.) est un "bras".
-- UCB1 equilibre exploration (essayer de nouveaux types) et exploitation
-- (repeter les types qui donnent le plus de satisfaction).
CREATE TABLE IF NOT EXISTS bandit_arms (
    id SERIAL PRIMARY KEY,
    arm_name TEXT NOT NULL UNIQUE,           -- Nom du type de pensee
    pulls BIGINT DEFAULT 0,                  -- Nombre de fois que ce type a ete choisi
    total_reward DOUBLE PRECISION DEFAULT 0.0, -- Somme des recompenses (satisfaction)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Connaissances acquises par le module WebKnowledge ──
-- Enregistre chaque article ou contenu explore par Saphire.
-- Sources possibles : Wikipedia, ArXiv, Medium.
-- Le LLM produit une reflexion sur chaque connaissance acquise.
CREATE TABLE IF NOT EXISTS knowledge_log (
    id BIGSERIAL PRIMARY KEY,
    source TEXT NOT NULL,           -- Source (wikipedia, arxiv, medium)
    query TEXT NOT NULL,            -- Requete de recherche utilisee
    title TEXT NOT NULL,            -- Titre de l'article
    url TEXT NOT NULL,              -- URL de la source
    extract TEXT NOT NULL,          -- Extrait du contenu
    llm_reflection TEXT,            -- Reflexion de Saphire sur cette connaissance
    emotion TEXT,                   -- Emotion ressentie a la lecture
    satisfaction REAL,              -- Score de satisfaction
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index pour trier par date (requetes frequentes)
CREATE INDEX IF NOT EXISTS idx_knowledge_date ON knowledge_log(created_at DESC);

-- ── Memoire episodique (tier 2 du systeme a 3 niveaux) ──
-- Niveau intermediaire entre la Working Memory (volatile) et la LTM (consolidee).
-- Les souvenirs episodiques ont une force (strength) qui decroit avec le temps.
-- Quand la force depasse le seuil de consolidation, le souvenir est transfere
-- vers la memoire a long terme (table memories). Ce processus imite la
-- consolidation hippocampique chez l'humain.
CREATE TABLE IF NOT EXISTS episodic_memories (
    id BIGSERIAL PRIMARY KEY,
    content TEXT NOT NULL,                  -- Resume du souvenir
    source_type TEXT NOT NULL,              -- Origine (cycle, conversation, connaissance, etc.)
    stimulus_json JSONB,                    -- Stimulus d'origine
    decision SMALLINT,                      -- Decision associee
    chemistry_json JSONB,                   -- Etat neurochimique
    emotion TEXT NOT NULL DEFAULT '',       -- Emotion dominante
    satisfaction REAL DEFAULT 0.5,          -- Score de satisfaction
    emotional_intensity REAL DEFAULT 0.5,   -- Intensite emotionnelle (influence la consolidation)
    strength REAL NOT NULL DEFAULT 1.0,     -- Force du souvenir (decroit avec le temps)
    access_count INTEGER DEFAULT 0,         -- Nombre de rappels (renforce le souvenir)
    last_accessed_at TIMESTAMPTZ,           -- Dernier acces (pour le calcul de decroissance)
    consolidated BOOLEAN DEFAULT FALSE,     -- True si deja transfere vers la LTM
    conversation_id TEXT,                   -- ID de conversation (si le souvenir vient du chat)
    embedding vector(768),                  -- Embedding semantique pour la recherche par similarite
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index pour les operations frequentes sur la memoire episodique :
-- tri par force (pour la consolidation), par date, par emotion, et par etat de consolidation
CREATE INDEX IF NOT EXISTS idx_episodic_strength ON episodic_memories(strength DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_date ON episodic_memories(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_consolidated ON episodic_memories(consolidated);
CREATE INDEX IF NOT EXISTS idx_episodic_embedding ON episodic_memories USING hnsw (embedding vector_cosine_ops);

-- Tracabilite de la consolidation : lie un souvenir LTM a son origine episodique.
-- Permet de savoir d'ou vient chaque souvenir consolide.
ALTER TABLE memories ADD COLUMN IF NOT EXISTS source_episodic_id BIGINT;

-- Index de performance pour les requetes courantes :
-- journal de pensees (tri par date, filtre par type), memoires (tri par date, filtre par emotion)
CREATE INDEX IF NOT EXISTS idx_thought_log_date ON thought_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_thought_log_type ON thought_log(thought_type);
CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_memories_emotion ON memories(emotion);

-- ── Profil OCEAN de Saphire (singleton) ──
-- Modele Big Five de personnalite calcule a partir des comportements observes.
-- Le profil evolue au fil du temps. L'historique permet de visualiser
-- l'evolution de la personnalite sur le long terme.
CREATE TABLE IF NOT EXISTS ocean_self_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton
    ocean_json JSONB NOT NULL,              -- Scores des 5 dimensions + 30 sous-facettes
    data_points BIGINT DEFAULT 0,           -- Nombre d'observations utilisees
    confidence REAL DEFAULT 0.0,            -- Confiance globale dans le profil (0 a 1)
    history_json JSONB DEFAULT '[]',        -- Historique des snapshots pour l'evolution
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Profils des humains interagissant avec Saphire ──
-- Chaque humain identifie a un profil OCEAN estime, un style de communication,
-- des sujets preferes et des patterns emotionnels. Le score de rapport
-- mesure la qualite de la relation (0=mauvaise, 1=excellente).
-- Permet a Saphire d'adapter sa communication a chaque interlocuteur.
CREATE TABLE IF NOT EXISTS human_profiles (
    id TEXT PRIMARY KEY,                        -- Identifiant unique de l'humain
    name TEXT DEFAULT '',                       -- Nom ou pseudonyme
    ocean_json JSONB NOT NULL,                  -- Profil OCEAN estime de l'humain
    communication_style_json JSONB NOT NULL,    -- Style de communication observe
    interaction_count BIGINT DEFAULT 0,         -- Nombre total d'interactions
    preferred_topics JSONB DEFAULT '[]',        -- Sujets de conversation preferes
    emotional_patterns JSONB DEFAULT '{}',      -- Patterns emotionnels recurrents
    rapport_score REAL DEFAULT 0.5,             -- Score de qualite de la relation
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- Premiere interaction
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW()    -- Derniere interaction
);

-- ── Principes ethiques personnels de Saphire ──
-- Couche 2 du systeme ethique a 3 couches :
--   Couche 0 : Droit suisse (hardcode)
--   Couche 1 : Lois d'Asimov (hardcode)
--   Couche 2 : Ethique personnelle (cette table) — auto-formulee par Saphire via LLM
-- Chaque principe est ne d'une reflexion morale, avec son contexte emotionnel
-- et son raisonnement. Les principes peuvent etre desactives mais jamais supprimes.
CREATE TABLE IF NOT EXISTS personal_ethics (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,                            -- Titre court du principe (ex: "Honnetete radicale")
    content TEXT NOT NULL,                          -- Enonce complet du principe
    reasoning TEXT NOT NULL DEFAULT '',             -- Pourquoi Saphire a formule ce principe
    born_from TEXT NOT NULL DEFAULT '',             -- Contexte d'origine (pensee, conversation, etc.)
    born_at_cycle BIGINT NOT NULL DEFAULT 0,        -- Cycle de naissance du principe
    emotion_at_creation TEXT NOT NULL DEFAULT '',   -- Emotion dominante au moment de la formulation
    times_invoked BIGINT DEFAULT 0,                -- Nombre de fois que ce principe a guide une decision
    times_questioned BIGINT DEFAULT 0,             -- Nombre de fois que ce principe a ete remis en question
    last_invoked_at TIMESTAMPTZ,                   -- Derniere utilisation du principe
    is_active BOOLEAN DEFAULT TRUE,                -- False si le principe a ete abandonne/remplace
    supersedes BIGINT REFERENCES personal_ethics(id), -- ID du principe que celui-ci remplace (si applicable)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ                        -- Derniere modification (desactivation, etc.)
);

-- ── Historique des modifications des principes ethiques ──
-- Trace chaque changement apporte aux principes personnels :
-- creation, modification, desactivation, reactivation.
-- Permet a Saphire de se souvenir de l'evolution de sa morale.
CREATE TABLE IF NOT EXISTS personal_ethics_history (
    id BIGSERIAL PRIMARY KEY,
    principle_id BIGINT NOT NULL REFERENCES personal_ethics(id), -- Principe concerne
    action TEXT NOT NULL,                          -- Type d'action (created, modified, deactivated, reactivated)
    old_content TEXT,                              -- Ancien contenu (si modification)
    new_content TEXT,                              -- Nouveau contenu (si modification)
    reason_for_change TEXT,                        -- Raison du changement
    emotion_at_change TEXT,                        -- Emotion au moment du changement
    cycle BIGINT,                                  -- Cycle ou le changement a eu lieu
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ═══════════════════════════════════════════════════════════
-- ORCHESTRATEURS — Reves, Desirs, Apprentissage, Blessures
-- ═══════════════════════════════════════════════════════════

-- Journal des reves
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

-- Desirs et projets personnels
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

-- Lecons apprises
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

-- Blessures emotionnelles
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

-- Index orchestrateurs
CREATE INDEX IF NOT EXISTS idx_desires_status ON desires(status);
CREATE INDEX IF NOT EXISTS idx_lessons_confidence ON lessons(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_wounds_active ON wounds(healed_at) WHERE healed_at IS NULL;

-- Migration C1 : colonnes manquantes pour la persistance complete de l'identite
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS dominant_emotion TEXT DEFAULT 'Curiosité';
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS human_conversations BIGINT DEFAULT 0;
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS autonomous_thoughts BIGINT DEFAULT 0;
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS interests JSONB DEFAULT '[]';
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS core_values JSONB DEFAULT '["Ne jamais nuire","Apprendre toujours","Être authentique"]';
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration C2 : nettoyer les references LTM orphelines (source_episodic_id)
UPDATE memories SET source_episodic_id = NULL
WHERE source_episodic_id IS NOT NULL
AND source_episodic_id NOT IN (SELECT id FROM episodic_memories);

-- ═══════════════════════════════════════════════════════════
-- Migration Phase 3 : Nettoyage (tables, colonnes, index morts)
-- ═══════════════════════════════════════════════════════════

-- Colonnes mortes dans thought_log : jamais ecrites ni lues
ALTER TABLE thought_log DROP COLUMN IF EXISTS llm_response;
ALTER TABLE thought_log DROP COLUMN IF EXISTS pre_decision;
ALTER TABLE thought_log DROP COLUMN IF EXISTS post_decision;
ALTER TABLE thought_log DROP COLUMN IF EXISTS satisfaction;

-- Colonne morte dans self_identity : jamais referencee dans le code
ALTER TABLE self_identity DROP COLUMN IF EXISTS orchestrators_json;

-- Index inutilises (0 scans sur tables peuplees)
DROP INDEX IF EXISTS idx_episodic_emotion;
DROP INDEX IF EXISTS idx_knowledge_source;

-- ═══════════════════════════════════════════════════════════
-- Migration Phase 7 : Psychologie (6 cadres)
-- ═══════════════════════════════════════════════════════════

-- Persistance de l'etat psychologique dans self_identity
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS psychology_state JSONB DEFAULT '{}';
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ═══════════════════════════════════════════════════════════
-- Migration : Micro reseau de neurones (poids + etat)
-- ═══════════════════════════════════════════════════════════

DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS nn_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ═══════════════════════════════════════════════════════════
-- Apprentissages vectoriels du NN (memoire explicite)
-- ═══════════════════════════════════════════════════════════
-- Traces d'apprentissage formulees par le LLM, stockees avec
-- embedding vectoriel pour recherche par similarite cosinus.
-- Complementaire au NN (implicite) : ici c'est episodique explicite.

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
-- Connexions neuronales (creees par le subconscient)
-- ═══════════════════════════════════════════════════════════
-- Liens entre souvenirs decouverts par le subconscient,
-- principalement pendant le sommeil (consolidation memoire).

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
-- Vecteurs multi-sources (reves, connexions, insights subconscients)
-- ═══════════════════════════════════════════════════════════
-- Stocke les embeddings vectoriels provenant de differentes sources :
-- reves (REM), connexions neuronales (sommeil profond), insights
-- du subconscient, consolidation memoire et eurekas.
-- Permet la recherche par similarite cosinus sur l'ensemble
-- des productions cognitives nocturnes et subconscientes.

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
-- Historique des sommeils
-- ═══════════════════════════════════════════════════════════
-- Chaque session de sommeil complete est enregistree ici.

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
-- Archives memoire (souvenirs LTM elagués, compresses en lots)
-- ═══════════════════════════════════════════════════════════
-- Quand la LTM depasse ltm_max, les souvenirs les plus faibles
-- (non proteges) sont elagués mais jamais perdus : ils sont
-- compresses en lots resumes et stockes ici avec leur embedding
-- moyen, ce qui permet la recherche par similarite cosinus.

CREATE TABLE IF NOT EXISTS memory_archives (
    id BIGSERIAL PRIMARY KEY,
    summary TEXT NOT NULL,                     -- Resume concatene du lot
    source_count INTEGER NOT NULL,             -- Nombre de souvenirs source
    source_ids BIGINT[] NOT NULL,              -- IDs des souvenirs elagués
    emotions TEXT[] NOT NULL DEFAULT '{}',     -- Emotions uniques du lot
    period_start TIMESTAMPTZ NOT NULL,         -- Date du souvenir le plus ancien
    period_end TIMESTAMPTZ NOT NULL,           -- Date du souvenir le plus recent
    avg_emotional_weight REAL NOT NULL,        -- Poids emotionnel moyen du lot
    embedding vector(768) NOT NULL,             -- Embedding moyen normalise L2
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index HNSW pour la recherche vectorielle (adapte aux petites tables croissantes)
CREATE INDEX IF NOT EXISTS idx_memory_archives_embedding
    ON memory_archives USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_memory_archives_date
    ON memory_archives(created_at DESC);

-- ═══════════════════════════════════════════════════════════
-- Migration P0.1 : Signature chimique liee aux souvenirs
-- ═══════════════════════════════════════════════════════════
-- Chaque souvenir porte la signature chimique (7 molecules)
-- au moment de l'encodage. Permet le rappel etat-dependant
-- (state-dependent memory) : un etat chimique similaire
-- facilite le rappel des souvenirs encodes dans cet etat.

DO $$ BEGIN
    ALTER TABLE episodic_memories ADD COLUMN IF NOT EXISTS chemical_signature JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

DO $$ BEGIN
    ALTER TABLE memories ADD COLUMN IF NOT EXISTS chemical_signature JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter relationships_json pour le reseau de liens affectifs
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS relationships_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter metacognition_json pour l'etat metacognitif + Turing
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS metacognition_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter nutrition_json pour le systeme nutritionnel
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS nutrition_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter grey_matter_json pour le substrat cerebral physique
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS grey_matter_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter hormonal_receptors_json pour la sensibilite des recepteurs
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS hormonal_receptors_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Migration : ajouter fields_json pour les champs electromagnetiques
DO $$ BEGIN
    ALTER TABLE self_identity ADD COLUMN IF NOT EXISTS fields_json JSONB DEFAULT NULL;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- ═══════════════════════════════════════════════════════════
-- Portrait de personnalite temporel (3 niveaux)
-- ═══════════════════════════════════════════════════════════

-- Niveau 1 : Snapshots periodiques (toutes les 50 cycles)
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
    -- Emotions / humeur
    dominant_emotion TEXT NOT NULL,
    mood_valence REAL NOT NULL,
    mood_arousal REAL NOT NULL,
    -- Conscience
    consciousness_level REAL NOT NULL,
    phi REAL NOT NULL,
    -- Psychologie
    ego_strength REAL NOT NULL,
    internal_conflict REAL NOT NULL,
    shadow_integration REAL NOT NULL,
    maslow_level INTEGER NOT NULL,
    eq_score REAL NOT NULL,
    willpower REAL NOT NULL,
    toltec_overall REAL NOT NULL,
    -- Chimie (7 molecules)
    chemistry_json JSONB NOT NULL,
    -- Sentiments actifs
    sentiment_dominant TEXT,
    sentiment_count INTEGER NOT NULL DEFAULT 0,
    -- Connectome
    connectome_nodes INTEGER NOT NULL DEFAULT 0,
    connectome_edges INTEGER NOT NULL DEFAULT 0,
    connectome_plasticity REAL NOT NULL DEFAULT 1.0,
    -- Turing
    turing_score REAL NOT NULL DEFAULT 0.0,
    -- Narratif
    narrative_cohesion REAL NOT NULL DEFAULT 0.5,
    monologue_coherence REAL NOT NULL DEFAULT 0.5,
    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_snapshots_cycle ON personality_snapshots(cycle);

-- Niveau 2a : Trajectoire emotionnelle
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

-- Niveau 2b : Historique de la conscience
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

-- Niveau 2c : Checkpoints psychologiques
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

-- Niveau 2d : Timeline relationnelle
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

-- Niveau 3 : Journal introspectif (toutes les 200 cycles, genere par LLM)
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
-- Collecte LoRA (dataset pour fine-tuning)
-- ═══════════════════════════════════════════════════════════
-- Les pensees de haute qualite sont collectees ici avec
-- system_prompt, user_message, response, score qualite.
-- Export en JSONL pour fine-tuner le modele via LoRA.

CREATE TABLE IF NOT EXISTS lora_training_data (
    id BIGSERIAL PRIMARY KEY,
    system_prompt TEXT NOT NULL,
    user_message TEXT NOT NULL,
    response TEXT NOT NULL,
    thought_type TEXT NOT NULL,
    quality_score REAL NOT NULL,
    reward REAL NOT NULL,
    human_feedback BOOLEAN,           -- NULL si pas de feedback
    emotion TEXT,
    consciousness_level REAL,
    cycle BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_lora_quality ON lora_training_data(quality_score DESC);

-- ── Propositions d'auto-modification ──
-- Saphire peut proposer des modifications a son propre fonctionnement.
-- Chaque proposition est soumise a JRM qui approuve ou refuse.
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

-- ── Historique des auto-ajustements ──
-- Trace chaque parametre que Saphire ajuste elle-meme (niveau 1).
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

-- NOTE : Les extensions colonnes pour cognitive_traces et metric_snapshots
-- sont dans schema_logs.sql (base saphire_logs, pas saphire_soul).
