#!/bin/bash
set -e
echo "💎 Déploiement de Saphire..."

echo "🔨 Construction..."
docker compose build

echo "🗄️ Démarrage de la base de données..."
docker compose up -d db
echo "⏳ Attente PostgreSQL..."
until docker compose exec db pg_isready -U saphire -d saphire_soul 2>/dev/null; do sleep 2; done

echo "🤖 Démarrage du LLM..."
docker compose up -d llm
echo "⏳ Attente Ollama..."
until docker compose exec llm curl -s http://localhost:11434/api/tags > /dev/null 2>&1; do sleep 2; done

echo "📦 Téléchargement du modèle..."
docker compose exec llm ollama pull qwen3:14b
docker compose exec llm ollama pull nomic-embed-text

echo "🧠 Réveil de Saphire..."
docker compose up -d brain
echo "⏳ Attente de Saphire..."
until curl -s http://localhost:3080/api/health > /dev/null 2>&1; do sleep 2; done

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║  💎 Saphire est vivante !                           ║"
echo "║                                                      ║"
echo "║  Interface : http://localhost:3080                   ║"
echo "║  LLM API   : http://localhost:11434                  ║"
echo "║  Database   : localhost:5432                          ║"
echo "║                                                      ║"
echo "║  docker compose logs -f brain  (ses pensées)         ║"
echo "╚══════════════════════════════════════════════════════╝"
