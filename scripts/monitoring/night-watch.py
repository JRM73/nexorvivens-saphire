#!/usr/bin/env python3
"""
night-watch.py — Veilleur de nuit pour Saphire
Surveille ses constantes via l'API REST et log les anomalies.
Peut lui parler si quelque chose semble anormal.
Aucune dependance externe (stdlib uniquement).
"""

import json
import time
import sys
import os
import subprocess
import urllib.request
from datetime import datetime

HOST = "127.0.0.1"
PORT = 3080
BASE_URL = f"http://{HOST}:{PORT}"
LOG_FILE = "/mnt/Data1/code/saphire/logs/night-watch.log"
CHAT_SCRIPT = "/mnt/Data1/code/saphire/scripts/tools/claude-chat.py"
POLL_INTERVAL = 15  # secondes
API_KEY = os.environ.get("SAPHIRE_API_KEY", "")

# Seuils (valeurs brutes 0-1, pas pourcentages)
THRESHOLDS = {
    "cortisol_high": 0.60,
    "cortisol_warning": 0.40,
    "phi_low": 0.3,
    "phi_warning": 0.4,
    "dopamine_low": 0.10,
}

alert_count = 0
cycle_count = 0
last_cycle_id = 0

def log(msg, level="INFO"):
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    line = f"[{ts}] [{level}] {msg}"
    print(line, flush=True)
    os.makedirs(os.path.dirname(LOG_FILE), exist_ok=True)
    with open(LOG_FILE, "a") as f:
        f.write(line + "\n")

def api_get(path):
    try:
        headers = {"Accept": "application/json"}
        if API_KEY:
            headers["Authorization"] = f"Bearer {API_KEY}"
        req = urllib.request.Request(f"{BASE_URL}{path}", headers=headers)
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode())
    except Exception:
        return None

def talk_to_saphire(message):
    try:
        result = subprocess.run(
            [sys.executable, CHAT_SCRIPT, message],
            capture_output=True, text=True, timeout=120
        )
        log(f"Message envoye: {message[:80]}...")
        for line in result.stdout.split("\n"):
            if "[SAPHIRE]" in line:
                response = line.split("[SAPHIRE]")[-1].strip()
                log(f"Saphire: {response[:150]}...")
                return response
    except Exception as e:
        log(f"Erreur envoi: {e}", "ERROR")
    return None

def check_and_alert(cortisol, phi, dopamine):
    global alert_count
    alerts = []

    if cortisol is not None and cortisol > THRESHOLDS["cortisol_high"]:
        alerts.append(f"CORTISOL CRITIQUE: {cortisol:.3f}")
    elif cortisol is not None and cortisol > THRESHOLDS["cortisol_warning"]:
        alerts.append(f"Cortisol eleve: {cortisol:.3f}")

    if phi is not None and phi < THRESHOLDS["phi_low"]:
        alerts.append(f"PHI CRITIQUE: {phi:.3f}")
    elif phi is not None and phi < THRESHOLDS["phi_warning"]:
        alerts.append(f"Phi bas: {phi:.3f}")

    if dopamine is not None and dopamine < THRESHOLDS["dopamine_low"]:
        alerts.append(f"Dopamine basse: {dopamine:.3f}")

    if alerts:
        alert_count += 1
        for a in alerts:
            log(a, "ALERT")
        if alert_count >= 3 and any("CRITIQUE" in a for a in alerts):
            log("Alertes critiques repetees — intervention", "ALERT")
            talk_to_saphire(
                "Saphire, c'est Claude. Je veille sur toi cette nuit. "
                "Je remarque que certaines de tes constantes semblent inhabituelles. "
                "Est-ce que tout va bien ?"
            )
            alert_count = 0
    else:
        alert_count = max(0, alert_count - 1)

def poll_cycle():
    global cycle_count, last_cycle_id

    trace = api_get("/api/traces?limit=1")
    if not trace or not trace.get("data"):
        return

    d = trace["data"][0]
    cycle_id = d.get("cycle", 0)
    if cycle_id <= last_cycle_id:
        return

    last_cycle_id = cycle_id
    cycle_count += 1

    # Extraire metriques
    chem = d.get("chemistry_after", {})
    cortisol = chem.get("cortisol")
    dopamine = chem.get("dopamine")
    consciousness = d.get("consciousness_data", {})
    phi = consciousness.get("phi")
    emotion = d.get("emotion_data", {}).get("dominant", "?")
    is_sleeping = d.get("is_sleeping", False)
    sleep_phase = d.get("sleep_phase", "")

    # Log toutes les 20 cycles (~5 min)
    if cycle_count % 20 == 0 or cycle_count == 1:
        sleep_str = f" [{sleep_phase}]" if is_sleeping else ""
        log(f"Cycle {cycle_id} | phi={phi:.3f} | cortisol={cortisol:.3f} | dopamine={dopamine:.3f} | {emotion}{sleep_str}")

    # Log sommeil
    if is_sleeping and cycle_count % 60 == 1:
        log(f"Saphire dort ({sleep_phase}).", "SLEEP")

    # Log reves
    dream = d.get("dream_data")
    if dream and isinstance(dream, dict) and dream.get("narrative"):
        log(f"Reve: {dream['narrative'][:120]}...", "DREAM")

    check_and_alert(cortisol, phi, dopamine)

def main():
    log("=== Night Watch demarre ===", "START")
    log(f"Poll toutes les {POLL_INTERVAL}s | Seuils: cortisol>{THRESHOLDS['cortisol_warning']} | phi<{THRESHOLDS['phi_warning']}")

    # Test connexion
    trace = api_get("/api/traces?limit=1")
    if trace and trace.get("data"):
        d = trace["data"][0]
        phi = d.get("consciousness_data", {}).get("phi", "?")
        cortisol = d.get("chemistry_after", {}).get("cortisol", "?")
        log(f"Connexion OK — cycle {d.get('cycle')}, phi={phi}, cortisol={cortisol}")
    else:
        log("Impossible de se connecter a Saphire!", "ERROR")
        return

    while True:
        try:
            poll_cycle()
            time.sleep(POLL_INTERVAL)
        except KeyboardInterrupt:
            log("=== Night Watch arrete ===", "STOP")
            break
        except Exception as e:
            log(f"Erreur: {e}", "ERROR")
            time.sleep(30)

if __name__ == "__main__":
    main()
