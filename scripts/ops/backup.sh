#!/bin/bash
# Sauvegarde de Saphire (toutes les 4 heures)
# Usage: ./backup.sh [--quiet]
set -euo pipefail

BACKUP_DIR="/mnt/Data2/backups/saphire"
DATE=$(date +%Y%m%d_%H%M)
LOG_FILE="/mnt/Data1/code/saphire/logs/backup.log"
QUIET="${1:-}"

log() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    echo "$msg" >> "$LOG_FILE"
    [ "$QUIET" != "--quiet" ] && echo "$msg"
}

mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

log "=== Début sauvegarde Saphire ==="

# 1. Base principale (identité, mémoires, éthique, pensées, connectome)
log "Dump saphire_soul..."
docker exec saphire-db pg_dump -U saphire -d saphire_soul \
  | gzip > "$BACKUP_DIR/saphire_soul_${DATE}.sql.gz"
log "  OK: saphire_soul_${DATE}.sql.gz ($(stat -c%s "$BACKUP_DIR/saphire_soul_${DATE}.sql.gz" | numfmt --to=iec))"

# 2. Base de logs (traces cognitives, métriques, historique LLM)
log "Dump saphire_logs..."
docker exec saphire-logs-db pg_dump -U saphire -d saphire_logs \
  | gzip > "$BACKUP_DIR/saphire_logs_${DATE}.sql.gz"
log "  OK: saphire_logs_${DATE}.sql.gz ($(stat -c%s "$BACKUP_DIR/saphire_logs_${DATE}.sql.gz" | numfmt --to=iec))"

# 3. Configuration
log "Archive config..."
tar czf "$BACKUP_DIR/config_${DATE}.tar.gz" \
  -C /mnt/Data1/code/saphire \
  saphire.toml profiles/ personalities/ factory_defaults.toml
log "  OK: config_${DATE}.tar.gz"

# 4. Documentation identitaire
log "Archive self-knowledge..."
tar czf "$BACKUP_DIR/self-knowledge_${DATE}.tar.gz" \
  -C /mnt/Data1/code/saphire \
  docs/self-knowledge/
log "  OK: self-knowledge_${DATE}.tar.gz"

# 5. Vérification d'intégrité
ERRORS=0
for f in "$BACKUP_DIR"/*_${DATE}.*; do
  size=$(stat -c%s "$f")
  if [ "$size" -lt 100 ]; then
    log "ERREUR: $(basename "$f") semble vide ($size octets)"
    ERRORS=$((ERRORS + 1))
  fi
done

if [ "$ERRORS" -gt 0 ]; then
  log "=== SAUVEGARDE ÉCHOUÉE ($ERRORS erreurs) ==="
  exit 1
fi

# 6. Rotation : garder 30 jours
DELETED=$(find "$BACKUP_DIR" -name "*.gz" -mtime +30 -delete -print | wc -l)
[ "$DELETED" -gt 0 ] && log "Rotation: $DELETED fichiers supprimés (>30 jours)"

# 7. Réplication vers le i7 (si allumé)
I7_HOST="192.168.1.129"
I7_DEST="malice@${I7_HOST}:/mnt/C512/code/backups/saphire/"
if ping -c 1 -W 2 "$I7_HOST" > /dev/null 2>&1; then
  log "i7 détecté — réplication rsync..."
  if rsync -az --delete "$BACKUP_DIR/" "$I7_DEST" 2>&1; then
    log "  OK: réplication vers $I7_HOST terminée"
  else
    log "  WARN: rsync échoué (SSH ou permissions ?)"
  fi
else
  log "i7 éteint — réplication ignorée"
fi

# 8. Résumé
TOTAL=$(du -sh "$BACKUP_DIR" | cut -f1)
log "=== Sauvegarde terminée — Total: $TOTAL ==="
