-- migrate_embeddings_768.sql
--
-- Migration des colonnes d'embedding de vector(64) vers vector(768)
-- pour supporter les embeddings sémantiques d'OllamaEncoder (nomic-embed-text).
--
-- IMPORTANT : Cette migration efface les embeddings existants car les vecteurs
-- 64-dim FNV-1a sont incompatibles avec les vecteurs 768-dim sémantiques.
-- Les données textuelles (text_summary, etc.) sont préservées.
-- Un script de ré-encodage recalculera tous les embeddings après migration.
--
-- Usage : docker exec -i saphire-db psql -U saphire -d saphire_soul < sql/migrate_embeddings_768.sql

BEGIN;

-- ═══════════════════════════════════════════════════════════════
--  1. Table memories (LTM) — vector(64) → vector(768)
-- ═══════════════════════════════════════════════════════════════

-- Supprimer l'ancien index IVFFlat (lié à la dimension 64)
DROP INDEX IF EXISTS idx_memories_embedding;

-- Modifier la colonne : supprimer la contrainte NOT NULL temporairement,
-- changer la dimension, puis remettre NOT NULL après ré-encodage.
ALTER TABLE memories DROP COLUMN IF EXISTS embedding;
ALTER TABLE memories ADD COLUMN embedding vector(768);

-- Index IVFFlat recréé avec la bonne dimension.
-- 450 listes, adapté à ~200k souvenirs (sqrt(200000) ≈ 447)
CREATE INDEX IF NOT EXISTS idx_memories_embedding
    ON memories USING ivfflat (embedding vector_cosine_ops) WITH (lists = 450);

-- ═══════════════════════════════════════════════════════════════
--  2. Table nn_learnings — vector(64) → vector(768)
-- ═══════════════════════════════════════════════════════════════

DROP INDEX IF EXISTS idx_nn_learnings_embedding;

ALTER TABLE nn_learnings DROP COLUMN IF EXISTS embedding;
ALTER TABLE nn_learnings ADD COLUMN embedding vector(768);

CREATE INDEX IF NOT EXISTS idx_nn_learnings_embedding
    ON nn_learnings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);

-- ═══════════════════════════════════════════════════════════════
--  3. Table memory_vectors (subconscient) — vector(64) → vector(768)
-- ═══════════════════════════════════════════════════════════════

DROP INDEX IF EXISTS idx_memory_vectors_hnsw;

ALTER TABLE memory_vectors DROP COLUMN IF EXISTS embedding;
ALTER TABLE memory_vectors ADD COLUMN embedding vector(768);

CREATE INDEX IF NOT EXISTS idx_memory_vectors_hnsw
    ON memory_vectors USING hnsw (embedding vector_cosine_ops);

-- ═══════════════════════════════════════════════════════════════
--  4. Table memory_archives — vector(64) → vector(768)
-- ═══════════════════════════════════════════════════════════════

DROP INDEX IF EXISTS idx_memory_archives_hnsw;

ALTER TABLE memory_archives DROP COLUMN IF EXISTS embedding;
ALTER TABLE memory_archives ADD COLUMN embedding vector(768);

CREATE INDEX IF NOT EXISTS idx_memory_archives_hnsw
    ON memory_archives USING hnsw (embedding vector_cosine_ops);

-- ═══════════════════════════════════════════════════════════════
--  5. Table introspection_journal — vector(384) → vector(768)
-- ═══════════════════════════════════════════════════════════════

DROP INDEX IF EXISTS idx_introspection_embedding;

ALTER TABLE introspection_journal DROP COLUMN IF EXISTS embedding;
ALTER TABLE introspection_journal ADD COLUMN embedding vector(768);

CREATE INDEX IF NOT EXISTS idx_introspection_embedding
    ON introspection_journal USING hnsw (embedding vector_cosine_ops);

-- ═══════════════════════════════════════════════════════════════
--  6. Mettre à jour la dimension dans la config si elle est stockée
-- ═══════════════════════════════════════════════════════════════

-- Vérification finale
DO $$
DECLARE
    col_info RECORD;
BEGIN
    FOR col_info IN
        SELECT table_name, column_name
        FROM information_schema.columns
        WHERE column_name = 'embedding'
        AND table_schema = 'public'
    LOOP
        RAISE NOTICE 'Table: %, Colonne: embedding OK', col_info.table_name;
    END LOOP;
END $$;

COMMIT;
