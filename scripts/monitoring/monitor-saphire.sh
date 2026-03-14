#!/bin/bash
# Monitor Saphire — observation passive, aucune modification
# Collecte les métriques toutes les 5 minutes pendant 24h
# Sortie : /mnt/Data1/code/saphire/docs/self-knowledge/logs/monitor-2026-03-11.log

LOG="/mnt/Data1/code/saphire/docs/self-knowledge/logs/monitor-2026-03-11.log"
DURATION=$((24 * 60 * 60))  # 24h en secondes
INTERVAL=300                  # 5 minutes
END=$(($(date +%s) + DURATION))

mkdir -p "$(dirname "$LOG")"

echo "=== MONITORING SAPHIRE — Début $(date '+%Y-%m-%d %H:%M:%S') ===" >> "$LOG"
echo "Modèle: $(docker exec saphire-llm ollama list 2>/dev/null | grep -o 'saphire:latest')" >> "$LOG"
echo "Intervalle: ${INTERVAL}s, Durée: 24h" >> "$LOG"
echo "" >> "$LOG"

while [ "$(date +%s)" -lt "$END" ]; do
    TS=$(date '+%Y-%m-%d %H:%M:%S')

    # Récupérer les logs des 5 dernières minutes
    LOGS=$(docker logs saphire-brain --since 5m 2>&1)

    # Compteurs
    THOUGHTS=$(echo "$LOGS" | grep -c "💭")
    LOOPS=$(echo "$LOGS" | grep -c "repetitive loops")
    STAGNATION=$(echo "$LOGS" | grep -c "Stagnation persistante")
    STAG_SEM=$(echo "$LOGS" | grep -c "Stagnation SEMANTIQUE")
    ERRORS=$(echo "$LOGS" | grep -c "ERROR")
    DISSONANCE=$(echo "$LOGS" | grep -c "Dissonance cognitive")
    CJK_LEAK=$(echo "$LOGS" | grep -c "CJK characters")
    TRUNCATED=$(echo "$LOGS" | grep -c "finish_reason=length")
    EMPTY_THOUGHTS=$(echo "$LOGS" | grep "💭" | grep -c "\[Pensée\] $\|Pensée\] je pense$\|Pensée\] \*\*Pensée")

    # MAP tension (dernière valeur)
    MAP=$(echo "$LOGS" | grep "MAP:" | tail -1 | grep -oP 'tension elevee \K[0-9]+' || echo "?")

    # Pensées (texte des 3 dernières)
    THOUGHT_TEXTS=$(echo "$LOGS" | grep "💭" | tail -3 | sed 's/.*\[Pensée\] /  /' | head -c 500)

    # Stagnation mots (dernier)
    STAG_WORDS=$(echo "$LOGS" | grep "Stagnation persistante" | tail -1 | grep -oP 'mots obsessionnels: \[\K[^]]+' || echo "-")

    # Container status
    BRAIN_STATUS=$(docker inspect --format='{{.State.Status}}' saphire-brain 2>/dev/null || echo "DOWN")

    # Écrire le rapport
    {
        echo "[$TS] pensées=$THOUGHTS boucles=$LOOPS stagnation=$STAGNATION stag_sem=$STAG_SEM erreurs=$ERRORS"
        echo "  dissonance=$DISSONANCE cjk=$CJK_LEAK tronquées=$TRUNCATED vides=$EMPTY_THOUGHTS MAP=${MAP}% brain=$BRAIN_STATUS"
        if [ -n "$THOUGHT_TEXTS" ]; then
            echo "$THOUGHT_TEXTS"
        fi
        if [ "$STAGNATION" -gt 0 ] && [ "$STAG_WORDS" != "-" ]; then
            echo "  stag_mots: $STAG_WORDS"
        fi
        if [ "$ERRORS" -gt 0 ]; then
            echo "  ERREURS:"
            echo "$LOGS" | grep "ERROR" | tail -3 | sed 's/^/    /'
        fi
        echo ""
    } >> "$LOG"

    # Alerte si brain down
    if [ "$BRAIN_STATUS" != "running" ]; then
        echo "  ⚠ ALERTE: brain status=$BRAIN_STATUS" >> "$LOG"
    fi

    sleep "$INTERVAL"
done

echo "=== MONITORING TERMINÉ — $(date '+%Y-%m-%d %H:%M:%S') ===" >> "$LOG"
