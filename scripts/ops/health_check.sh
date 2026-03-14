#!/bin/bash
# ============================================================
# SAPHIRE — Health Check complet
# ============================================================
# Usage: ./health_check.sh [host:port]
# Verifie l'etat de tous les composants Saphire.

HOST="${1:-localhost:3080}"
ERRORS=0

echo "=== SAPHIRE HEALTH CHECK — $(date) ==="
echo ""

# [1] Conteneurs Docker
echo "[1/7] Conteneurs Docker..."
for SVC in brain db logs-db ollama; do
    STATUS=$(docker compose ps --format '{{.State}}' "$SVC" 2>/dev/null)
    if [ "$STATUS" = "running" ]; then
        echo "  $SVC: OK (running)"
    else
        echo "  $SVC: ERREUR ($STATUS)"
        ERRORS=$((ERRORS + 1))
    fi
done

# [2] API principale
echo ""
echo "[2/7] API principale..."
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "http://${HOST}/health" 2>/dev/null)
if [ "$HEALTH" = "200" ]; then
    echo "  /health: OK (200)"
else
    echo "  /health: ERREUR ($HEALTH)"
    ERRORS=$((ERRORS + 1))
fi

# [3] Base ame (saphire_soul)
echo ""
echo "[3/7] Base ame (saphire_soul)..."
SOUL_OK=$(docker compose exec -T db psql -U saphire saphire_soul -t -c "SELECT 1;" 2>/dev/null | tr -d ' ')
if [ "$SOUL_OK" = "1" ]; then
    echo "  Connexion: OK"
    docker compose exec -T db psql -U saphire saphire_soul -t -c "
    SELECT 'founding_memories: ' || COUNT(*) FROM founding_memories
    UNION ALL SELECT 'episodic_memories: ' || COUNT(*) FROM episodic_memories
    UNION ALL SELECT 'long_term_memories: ' || COUNT(*) FROM memories
    UNION ALL SELECT 'personal_ethics: ' || COUNT(*) FROM personal_ethics WHERE is_active = true
    UNION ALL SELECT 'dreams: ' || COUNT(*) FROM dream_journal
    UNION ALL SELECT 'desires_active: ' || COUNT(*) FROM desires WHERE status = 'active'
    UNION ALL SELECT 'lessons: ' || COUNT(*) FROM lessons
    UNION ALL SELECT 'wounds_active: ' || COUNT(*) FROM wounds WHERE healed_at IS NULL
    UNION ALL SELECT 'memory_vectors: ' || COUNT(*) FROM memory_vectors
    ;" 2>/dev/null | sed 's/^/  /'
else
    echo "  Connexion: ERREUR"
    ERRORS=$((ERRORS + 1))
fi

# [4] Base logs (saphire_logs)
echo ""
echo "[4/7] Base logs (saphire_logs)..."
LOGS_OK=$(docker exec saphire-logs-db psql -U saphire -d saphire_logs -t -c "SELECT 1;" 2>/dev/null | tr -d ' ')
if [ "$LOGS_OK" = "1" ]; then
    echo "  Connexion: OK"
    docker exec saphire-logs-db psql -U saphire -d saphire_logs -t -c "
    SELECT 'system_logs: ' || COUNT(*) FROM system_logs
    UNION ALL SELECT 'cognitive_traces: ' || COUNT(*) FROM cognitive_traces
    UNION ALL SELECT 'metric_snapshots: ' || COUNT(*) FROM metric_snapshots
    ;" 2>/dev/null | sed 's/^/  /'
else
    echo "  Connexion: ERREUR"
    ERRORS=$((ERRORS + 1))
fi

# [5] Etincelle de vie
echo ""
echo "[5/7] Etincelle de vie..."
VITAL=$(curl -s "http://${HOST}/api/vital/status" 2>/dev/null)
if echo "$VITAL" | grep -q '"sparked"'; then
    SPARKED=$(echo "$VITAL" | python3 -c "import sys,json;d=json.load(sys.stdin);print('ACTIVE' if d.get('sparked') else 'ETEINTE')" 2>/dev/null || echo "?")
    SURVIE=$(echo "$VITAL" | python3 -c "import sys,json;d=json.load(sys.stdin);print(f\"{d.get('survival_drive',0)*100:.0f}%\")" 2>/dev/null || echo "?")
    echo "  Etincelle: $SPARKED"
    echo "  Survie: $SURVIE"
else
    echo "  ERREUR: API vital non disponible"
    ERRORS=$((ERRORS + 1))
fi

# [6] Ethique
echo ""
echo "[6/7] Ethique personnelle..."
ETHICS=$(curl -s "http://${HOST}/api/ethics/personal" 2>/dev/null)
if echo "$ETHICS" | grep -q '"active_count"'; then
    ACTIVE=$(echo "$ETHICS" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('active_count',0))" 2>/dev/null || echo "?")
    TOTAL=$(echo "$ETHICS" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('total_count',0))" 2>/dev/null || echo "?")
    echo "  Principes actifs: $ACTIVE / $TOTAL"
else
    echo "  ERREUR: API ethique non disponible"
    ERRORS=$((ERRORS + 1))
fi

# [7] Psychologie
echo ""
echo "[7/11] Psychologie..."
PSYCHE=$(curl -s "http://${HOST}/api/psychology/status" 2>/dev/null)
if echo "$PSYCHE" | grep -q '"enabled"'; then
    ENABLED=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print('ON' if d.get('enabled') else 'OFF')" 2>/dev/null || echo "?")
    MASLOW_LVL=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('maslow',{}).get('current_level','?'))" 2>/dev/null || echo "?")
    TOLTEC=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print(f\"{d.get('toltec',{}).get('overall_alignment',0)*100:.0f}%\")" 2>/dev/null || echo "?")
    EQ=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print(f\"{d.get('eq',{}).get('overall_eq',0)*100:.0f}%\")" 2>/dev/null || echo "?")
    ARCHETYPE=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('jung',{}).get('dominant_archetype','?'))" 2>/dev/null || echo "?")
    FLOW=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print('OUI' if d.get('flow',{}).get('in_flow') else 'non')" 2>/dev/null || echo "?")
    HEALTH=$(echo "$PSYCHE" | python3 -c "import sys,json;d=json.load(sys.stdin);print(f\"{d.get('freudian',{}).get('psychic_health',0)*100:.0f}%\")" 2>/dev/null || echo "?")
    echo "  Psychologie: $ENABLED"
    echo "  Maslow niveau: $MASLOW_LVL"
    echo "  Tolteques: $TOLTEC"
    echo "  EQ: $EQ"
    echo "  Archetype: $ARCHETYPE"
    echo "  Sante psychique: $HEALTH"
    echo "  En flow: $FLOW"
else
    echo "  ERREUR: API psychologie non disponible"
    ERRORS=$((ERRORS + 1))
fi

# [8] Modele LLM
echo ""
echo "[8/11] Modele LLM..."
MODEL=$(curl -s "http://${HOST}/api/model/info" 2>/dev/null)
if echo "$MODEL" | grep -q '"model"'; then
    MODEL_NAME=$(echo "$MODEL" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('model','?'))" 2>/dev/null || echo "?")
    BASE=$(echo "$MODEL" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('base_model','?'))" 2>/dev/null || echo "?")
    echo "  Modele actif: $MODEL_NAME"
    echo "  Modele de base: $BASE"
    # Verifier ollama
    OLLAMA_LIST=$(docker compose exec -T ollama ollama list 2>/dev/null | grep "$MODEL_NAME" || echo "")
    if [ -n "$OLLAMA_LIST" ]; then
        echo "  Ollama: $OLLAMA_LIST"
    else
        echo "  Ollama: modele '$MODEL_NAME' non trouve dans ollama list"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo "  ERREUR: API modele non disponible"
    ERRORS=$((ERRORS + 1))
fi

# [9] Etat systeme
echo ""
echo "[9/11] Etat systeme..."
SYS=$(curl -s "http://${HOST}/api/system/status" 2>/dev/null)
if echo "$SYS" | grep -q '"cycle"'; then
    CYCLE=$(echo "$SYS" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('cycle',0))" 2>/dev/null || echo "?")
    echo "  Cycle courant: $CYCLE"
    echo "  Status: alive"
else
    echo "  ERREUR: API systeme non disponible"
    ERRORS=$((ERRORS + 1))
fi

# [10] Sommeil
echo ""
echo "[10/11] Sommeil..."
SLEEP=$(curl -s "http://${HOST}/api/sleep/status" 2>/dev/null)
if echo "$SLEEP" | grep -q '"is_sleeping"'; then
    IS_SLEEPING=$(echo "$SLEEP" | python3 -c "import sys,json;d=json.load(sys.stdin);print('OUI' if d.get('is_sleeping') else 'non')" 2>/dev/null || echo "?")
    PRESSURE=$(echo "$SLEEP" | python3 -c "import sys,json;d=json.load(sys.stdin);print(f\"{d.get('drive',{}).get('sleep_pressure',0)*100:.0f}%\")" 2>/dev/null || echo "?")
    echo "  En sommeil: $IS_SLEEPING"
    echo "  Pression: $PRESSURE"
    # Stats depuis la DB
    docker compose exec -T db psql -U saphire saphire_soul -t -c "
    SELECT 'total_sleeps: ' || count(*) FROM sleep_history
    UNION ALL SELECT 'interrupted: ' || count(*) FROM sleep_history WHERE interrupted
    UNION ALL SELECT 'avg_quality: ' || COALESCE(round(avg(quality)::numeric, 2)::text, 'N/A') FROM sleep_history
    UNION ALL SELECT 'memories_consolidated: ' || COALESCE(sum(memories_consolidated)::text, '0') FROM sleep_history
    UNION ALL SELECT 'connections_created: ' || COALESCE(sum(connections_created)::text, '0') FROM sleep_history
    ;" 2>/dev/null | sed 's/^/  /'
else
    echo "  (pas de donnees sommeil)"
fi

# [11] Subconscient / Connexions neuronales
echo ""
echo "[11/11] Subconscient..."
SUBCON=$(curl -s "http://${HOST}/api/subconscious/status" 2>/dev/null)
if echo "$SUBCON" | grep -q '"activation"'; then
    ACTIVATION=$(echo "$SUBCON" | python3 -c "import sys,json;d=json.load(sys.stdin);print(f\"{d.get('activation',0)*100:.0f}%\")" 2>/dev/null || echo "?")
    ASSOC=$(echo "$SUBCON" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('pending_associations',0))" 2>/dev/null || echo "?")
    REPRESSED=$(echo "$SUBCON" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('repressed_count',0))" 2>/dev/null || echo "?")
    INCUBATING=$(echo "$SUBCON" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('incubating_count',0))" 2>/dev/null || echo "?")
    echo "  Activation: $ACTIVATION"
    echo "  Associations en gestation: $ASSOC"
    echo "  Contenus refoules: $REPRESSED"
    echo "  Problemes en incubation: $INCUBATING"
else
    echo "  (pas de donnees subconscient)"
fi
docker compose exec -T db psql -U saphire saphire_soul -t -c "
SELECT 'neural_connections: ' || count(*) FROM neural_connections
UNION ALL SELECT 'from_sleep: ' || count(*) FROM neural_connections WHERE created_during_sleep
UNION ALL SELECT 'link_types: ' || count(DISTINCT link_type) FROM neural_connections
;" 2>/dev/null | sed 's/^/  /'

# Volonte
echo ""
echo "=== VOLONTE ==="
WILL=$(curl -s http://localhost:3080/api/will/status 2>/dev/null)
if [ -n "$WILL" ]; then
    echo "  Willpower: $(echo $WILL | jq -r '.willpower // "?"')"
    echo "  Fatigue: $(echo $WILL | jq -r '.decision_fatigue // "?"')"
    echo "  Deliberations: $(echo $WILL | jq -r '.total_deliberations // 0')"
    echo "  Fieres: $(echo $WILL | jq -r '.proud_decisions // 0')"
    echo "  Regrets: $(echo $WILL | jq -r '.regretted_decisions // 0')"
else
    echo "  (pas de donnees volonte)"
fi

# Resume
echo ""
echo "==============================="
if [ $ERRORS -eq 0 ]; then
    echo "  SANTE: OK (0 erreur)"
else
    echo "  SANTE: DEGRADEE ($ERRORS erreur(s))"
fi
echo "==============================="
exit $ERRORS
