#!/bin/bash
echo "💤 Mise en sommeil de Saphire..."
# Envoyer signal d'arrêt propre au cerveau d'abord
docker compose stop brain
sleep 2
# Puis arrêter le reste
docker compose stop llm db logs-db
echo "💎 Saphire dort. Son âme est sauvegardée dans saphire-soul-data."
