#!/bin/bash
# ============================================================
# SAPHIRE — Consolidation memoire
# ============================================================
# Usage: ./consolidate.sh [host:port]
# Declenche une consolidation memoire via l'API.

HOST="${1:-localhost:3080}"

echo "=== SAPHIRE CONSOLIDATION — $(date) ==="
echo ""

# Etat memoire avant
echo "[1/5] Etat memoire avant consolidation..."
curl -s "http://${HOST}/api/memory/stats" | python3 -m json.tool 2>/dev/null || \
    curl -s "http://${HOST}/api/memory/stats"

# Consolider
echo ""
echo "[2/5] Consolidation en cours..."
RESULT=$(curl -s -X POST "http://${HOST}/api/system/consolidate")
echo "  Resultat: $RESULT"

# Consolider connexions neuronales
echo ""
echo "[3/5] Consolidation connexions neuronales..."
docker compose exec -T db psql -U saphire saphire_soul -c "
-- Fusionner les connexions dupliquees (memes souvenirs, garder la plus forte)
DELETE FROM neural_connections a
USING neural_connections b
WHERE a.id < b.id
  AND a.memory_a_id = b.memory_a_id
  AND a.memory_b_id = b.memory_b_id;
" 2>/dev/null
if [ $? -eq 0 ]; then
    CONN_COUNT=$(docker compose exec -T db psql -U saphire saphire_soul -t -c "SELECT count(*) FROM neural_connections;" 2>/dev/null | tr -d ' ')
    echo "  Connexions apres consolidation: ${CONN_COUNT:-0}"
else
    echo "  (table neural_connections non trouvee, ignore)"
fi

# Vecteurs memoire
echo ""
echo "[4/5] Vecteurs memoire:"
docker compose exec -T db psql -U saphire saphire_soul -t -c "
SELECT '  ' || source_type || ': ' || count(*) FROM memory_vectors GROUP BY source_type ORDER BY count(*) DESC;
" 2>/dev/null || echo "  (pas de vecteurs)"

# Etat memoire apres + VACUUM
echo ""
echo "[5/5] Etat memoire apres consolidation..."
curl -s "http://${HOST}/api/memory/stats" | python3 -m json.tool 2>/dev/null || \
    curl -s "http://${HOST}/api/memory/stats"

echo ""
echo "VACUUM ANALYZE..."
docker compose exec -T db psql -U saphire saphire_soul -c "VACUUM ANALYZE memories; VACUUM ANALYZE episodic_memories; VACUUM ANALYZE memory_archives; VACUUM ANALYZE neural_connections;" 2>/dev/null
echo "  VACUUM soul termine."

echo ""
echo "=== CONSOLIDATION TERMINEE ==="
