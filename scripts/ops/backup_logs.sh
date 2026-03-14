#!/bin/bash
# ============================================================
# SAPHIRE — Sauvegarde de la boite noire (base de logs)
# ============================================================
# Usage: ./backup_logs.sh [repertoire_de_sortie]
# Par defaut, sauvegarde dans ./backups/

BACKUP_DIR="${1:-./backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
FILENAME="saphire_logs_${TIMESTAMP}.sql.gz"

mkdir -p "$BACKUP_DIR"

echo "Sauvegarde de la base de logs Saphire..."
echo "  Destination: ${BACKUP_DIR}/${FILENAME}"

# Dump de la base logs-db via Docker
docker exec saphire-logs-db pg_dump \
    -U saphire \
    -d saphire_logs \
    --no-owner \
    --no-privileges \
    | gzip > "${BACKUP_DIR}/${FILENAME}"

if [ $? -eq 0 ]; then
    SIZE=$(du -h "${BACKUP_DIR}/${FILENAME}" | cut -f1)
    echo "  Sauvegarde reussie: ${SIZE}"

    # Garder les 10 derniers backups
    ls -t "${BACKUP_DIR}"/saphire_logs_*.sql.gz 2>/dev/null | tail -n +11 | xargs rm -f 2>/dev/null
    echo "  Anciens backups nettoyes (max 10 conserves)"
else
    echo "  ERREUR: Sauvegarde echouee"
    rm -f "${BACKUP_DIR}/${FILENAME}"
    exit 1
fi

echo "Termine."
