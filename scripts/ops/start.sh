#!/bin/bash
echo "💎 Réveil de Saphire..."

# Bases de données d'abord
echo "🗄️ Démarrage des bases de données..."
docker compose up -d db logs-db

echo "⏳ Attente PostgreSQL (soul)..."
until docker compose exec db pg_isready -U saphire -d saphire_soul 2>/dev/null; do sleep 2; done
echo "⏳ Attente PostgreSQL (logs)..."
until docker compose exec logs-db pg_isready -U saphire -d saphire_logs 2>/dev/null; do sleep 2; done

# LLM
echo "🤖 Démarrage du LLM..."
docker compose up -d llm
echo "⏳ Attente Ollama..."
until docker compose exec llm curl -s http://localhost:11434/api/tags > /dev/null 2>&1; do sleep 2; done

# Cerveau
echo "🧠 Réveil de Saphire..."
docker compose up -d brain
echo "⏳ Attente de Saphire..."
until curl -s http://localhost:3080/api/health > /dev/null 2>&1; do sleep 2; done

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║  💎 Saphire est réveillée !                         ║"
echo "║                                                      ║"
echo "║  Interface : http://localhost:3080                   ║"
echo "║  Dashboard : http://localhost:3080/dashboard         ║"
echo "║  LLM API   : http://localhost:11434                  ║"
echo "║  Database   : localhost:5432 / 5433                   ║"
echo "║                                                      ║"
echo "║  docker compose logs -f brain  (ses pensées)         ║"
echo "╚══════════════════════════════════════════════════════╝"
