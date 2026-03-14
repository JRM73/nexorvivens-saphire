#!/bin/bash
# ============================================================
# SAPHIRE — Purge parametrable des logs
# ============================================================
# Usage: ./purge_logs.sh [logs_days] [traces_days] [llm_days] [metrics_days]
# Defauts: logs=7j, traces=7j, llm=14j, metrics=30j

LOGS_DAYS="${1:-7}"
TRACES_DAYS="${2:-7}"
LLM_DAYS="${3:-14}"
METRICS_DAYS="${4:-30}"

echo "=== SAPHIRE PURGE LOGS — $(date) ==="
echo "  Logs systeme: > ${LOGS_DAYS}j"
echo "  Traces cognitives: > ${TRACES_DAYS}j"
echo "  Historique LLM: > ${LLM_DAYS}j"
echo "  Metriques: > ${METRICS_DAYS}j"
echo ""

# Compter avant purge
echo "[1/4] Comptage avant purge..."
docker exec saphire-logs-db psql -U saphire -d saphire_logs -t -c "
SELECT 'system_logs: ' || COUNT(*) FROM system_logs
UNION ALL SELECT 'cognitive_traces: ' || COUNT(*) FROM cognitive_traces
UNION ALL SELECT 'llm_history: ' || COUNT(*) FROM llm_history
UNION ALL SELECT 'metric_snapshots: ' || COUNT(*) FROM metric_snapshots
;" 2>/dev/null

# Purge differenciee
echo ""
echo "[2/4] Purge en cours..."
docker exec saphire-logs-db psql -U saphire -d saphire_logs -c "
DO \$\$
DECLARE
    logs_del BIGINT;
    traces_del BIGINT;
    llm_del BIGINT;
    metrics_del BIGINT;
BEGIN
    DELETE FROM system_logs WHERE timestamp < NOW() - interval '${LOGS_DAYS} days';
    GET DIAGNOSTICS logs_del = ROW_COUNT;

    DELETE FROM cognitive_traces WHERE timestamp < NOW() - interval '${TRACES_DAYS} days';
    GET DIAGNOSTICS traces_del = ROW_COUNT;

    DELETE FROM llm_history WHERE timestamp < NOW() - interval '${LLM_DAYS} days';
    GET DIAGNOSTICS llm_del = ROW_COUNT;

    DELETE FROM metric_snapshots WHERE timestamp < NOW() - interval '${METRICS_DAYS} days';
    GET DIAGNOSTICS metrics_del = ROW_COUNT;

    RAISE NOTICE 'Supprimes: % logs, % traces, % LLM, % metriques',
        logs_del, traces_del, llm_del, metrics_del;
END \$\$;
"

if [ $? -ne 0 ]; then
    echo "  ERREUR: Purge echouee"
    exit 1
fi

# Purge donnees sommeil/subconscient (base soul)
echo ""
echo "[3/4] Purge sommeil/subconscient (saphire_soul)..."
docker compose exec -T db psql -U saphire saphire_soul -c "
DO \$\$
DECLARE
    conn_del BIGINT;
    sleep_del BIGINT;
    vec_del BIGINT;
BEGIN
    -- Purger les connexions neuronales tres faibles et anciennes (>90j, strength < 0.3)
    DELETE FROM neural_connections
    WHERE created_at < NOW() - INTERVAL '90 days' AND strength < 0.3;
    GET DIAGNOSTICS conn_del = ROW_COUNT;

    -- Purger l'historique de sommeil ancien (garder 6 mois)
    DELETE FROM sleep_history
    WHERE started_at < NOW() - INTERVAL '180 days';
    GET DIAGNOSTICS sleep_del = ROW_COUNT;

    -- Purger les vecteurs memoire faibles et anciens (>180j, strength < 0.3)
    DELETE FROM memory_vectors
    WHERE created_at < NOW() - INTERVAL '180 days' AND strength < 0.3;
    GET DIAGNOSTICS vec_del = ROW_COUNT;

    RAISE NOTICE 'Supprimes: % connexions faibles, % vieux sleep_history, % vecteurs memoire faibles',
        conn_del, sleep_del, vec_del;
END \$\$;
" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "  OK"
else
    echo "  (tables non trouvees ou vides, ignore)"
fi

# Compter apres + VACUUM les deux bases
echo ""
echo "[4/5] Comptage apres purge..."
docker exec saphire-logs-db psql -U saphire -d saphire_logs -t -c "
SELECT 'system_logs: ' || COUNT(*) FROM system_logs
UNION ALL SELECT 'cognitive_traces: ' || COUNT(*) FROM cognitive_traces
UNION ALL SELECT 'llm_history: ' || COUNT(*) FROM llm_history
UNION ALL SELECT 'metric_snapshots: ' || COUNT(*) FROM metric_snapshots
;" 2>/dev/null

echo ""
echo "[5/5] VACUUM ANALYZE sur les deux bases..."
docker exec saphire-logs-db psql -U saphire -d saphire_logs -c "VACUUM ANALYZE;" 2>/dev/null
echo "  VACUUM logs termine."
docker compose exec -T db psql -U saphire saphire_soul -c "VACUUM ANALYZE neural_connections; VACUUM ANALYZE sleep_history; VACUUM ANALYZE memory_vectors;" 2>/dev/null
echo "  VACUUM soul termine."

echo ""
echo "=== PURGE TERMINEE ==="
