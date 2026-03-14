#!/bin/bash
# Saphire Broadcast Logger — logs all broadcasts to daily files
# Usage: ./saphire-logger.sh [duration_in_seconds]
# Default: 3600s (1 hour). Use 0 for indefinite (re-launches every hour).

LOGDIR="/mnt/Data1/code/saphire/docs/self-knowledge/logs"
mkdir -p "$LOGDIR"

DURATION="${1:-3600}"
LOGFILE="$LOGDIR/broadcast-$(date +%Y-%m-%d).log"

echo "[$(date +%H:%M:%S)] Logger started — writing to $LOGFILE" | tee -a "$LOGFILE"

if [ "$DURATION" = "0" ]; then
  # Indefinite mode: re-launch every hour
  while true; do
    python3 /mnt/Data1/code/saphire/scripts/tools/claude-chat.py --listen 3600 2>&1 | tee -a "$LOGFILE"
    echo "[$(date +%H:%M:%S)] Reconnecting..." | tee -a "$LOGFILE"
  done
else
  python3 /mnt/Data1/code/saphire/scripts/tools/claude-chat.py --listen "$DURATION" 2>&1 | tee -a "$LOGFILE"
fi
