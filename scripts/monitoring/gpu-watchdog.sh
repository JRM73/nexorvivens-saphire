#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────
# gpu-watchdog.sh — Surveillance GPU + container Ollama
#
# Enregistre les metriques GPU toutes les minutes dans un log rotatif.
# Detecte les anomalies (temperature, fallback CPU, port conflit)
# et redemarre le container si necessaire.
#
# Installation :
#   sudo cp scripts/gpu-watchdog.service scripts/gpu-watchdog.timer /etc/systemd/system/
#   sudo systemctl daemon-reload
#   sudo systemctl enable --now gpu-watchdog.timer
# ──────────────────────────────────────────────────────────────

set -euo pipefail

LOG_DIR="/var/log/saphire"
LOG_FILE="$LOG_DIR/gpu-watchdog.log"
MAX_LOG_SIZE=10485760  # 10 Mo
COMPOSE_DIR="/mnt/Data1/code/saphire"
CONTAINER="saphire-llm"
TEMP_CRIT=90
ALERT_FILE="/tmp/saphire-gpu-alert"

mkdir -p "$LOG_DIR"

# --- Rotation log ---
if [[ -f "$LOG_FILE" ]] && (( $(stat -c%s "$LOG_FILE" 2>/dev/null || echo 0) > MAX_LOG_SIZE )); then
    mv "$LOG_FILE" "$LOG_FILE.1"
fi

timestamp() { date '+%Y-%m-%d %H:%M:%S'; }

log() { echo "[$(timestamp)] $*" >> "$LOG_FILE"; }

alert() {
    log "ALERTE: $*"
    # Eviter le spam — 1 alerte par heure max
    if [[ ! -f "$ALERT_FILE" ]] || (( $(date +%s) - $(stat -c%Y "$ALERT_FILE" 2>/dev/null || echo 0) > 3600 )); then
        touch "$ALERT_FILE"
        echo "[$(timestamp)] GPU WATCHDOG ALERTE: $*" | wall 2>/dev/null || true
    fi
}

# --- 1. Verification nvidia-smi ---
if ! nvidia_out=$(nvidia-smi --query-gpu=temperature.gpu,utilization.gpu,utilization.memory,memory.used,memory.total,power.draw,persistence_mode --format=csv,noheader,nounits 2>&1); then
    alert "nvidia-smi echoue — driver GPU potentiellement crashe: $nvidia_out"
    exit 1
fi

IFS=',' read -r temp gpu_util mem_util mem_used mem_total power persist <<< "$nvidia_out"
temp=$(echo "$temp" | xargs)
gpu_util=$(echo "$gpu_util" | xargs)
mem_util=$(echo "$mem_util" | xargs)
mem_used=$(echo "$mem_used" | xargs)
mem_total=$(echo "$mem_total" | xargs)
power=$(echo "$power" | xargs)
persist=$(echo "$persist" | xargs)

log "GPU: ${temp}C | util=${gpu_util}% | VRAM=${mem_used}/${mem_total}MiB | power=${power}W | persist=${persist}"

# --- 2. Alertes GPU ---
if (( temp > TEMP_CRIT )); then
    alert "Temperature GPU critique: ${temp}C (seuil: ${TEMP_CRIT}C)"
fi

if [[ "$persist" != "Enabled" ]]; then
    log "WARN: Persistence mode desactive — tentative de reactivation"
    nvidia-smi -pm 1 2>/dev/null && log "Persistence mode reactive" || alert "Echec reactivation persistence mode"
fi

# --- 3. Verification container Ollama ---
container_state=$(docker inspect -f '{{.State.Status}}' "$CONTAINER" 2>/dev/null || echo "absent")
if [[ "$container_state" != "running" ]]; then
    alert "Container $CONTAINER non running (etat: $container_state)"
    log "Tentative redemarrage via docker compose..."
    cd "$COMPOSE_DIR" && docker compose up -d llm 2>> "$LOG_FILE"
    sleep 10
    new_state=$(docker inspect -f '{{.State.Status}}' "$CONTAINER" 2>/dev/null || echo "absent")
    if [[ "$new_state" == "running" ]]; then
        log "Container $CONTAINER redemarre avec succes"
    else
        alert "Echec redemarrage container $CONTAINER (etat: $new_state)"
    fi
    exit 0
fi

# --- 4. Detection fallback CPU (Ollama sans GPU = surcharge) ---
ollama_pid=$(docker inspect -f '{{.State.Pid}}' "$CONTAINER" 2>/dev/null || echo "0")
if [[ "$ollama_pid" != "0" ]]; then
    # Verifier si le processus ollama utilise le GPU
    gpu_procs=$(nvidia-smi --query-compute-apps=pid --format=csv,noheader 2>/dev/null || echo "")
    ollama_cpu=$(docker stats --no-stream --format '{{.CPUPerc}}' "$CONTAINER" 2>/dev/null | tr -d '%' || echo "0")
    ollama_cpu_int=${ollama_cpu%.*}

    if [[ -z "$gpu_procs" ]] && (( ollama_cpu_int > 150 )); then
        alert "Ollama en fallback CPU (CPU=${ollama_cpu}%, aucun process GPU) — surcharge detectee"
    fi

    log "Container $CONTAINER: running | CPU=${ollama_cpu}% | PID=$ollama_pid"
fi

# --- 5. Detection conflit port (ollama host vs docker) ---
host_ollama=$(pgrep -f '/usr/local/bin/ollama serve' 2>/dev/null || true)
if [[ -n "$host_ollama" ]]; then
    alert "Ollama host detecte (PID=$host_ollama) — conflit port 11434 possible!"
fi
