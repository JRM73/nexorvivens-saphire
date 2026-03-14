#!/usr/bin/env python3
# =============================================================================
# claude-chat.py — Client WebSocket pour parler a Saphire depuis le terminal
# =============================================================================
# Usage:
#   python3 scripts/claude-chat.py "message"          Envoyer un message, attendre la reponse
#   python3 scripts/claude-chat.py --listen [sec]     Ecouter le broadcast (60s par defaut)
#   python3 scripts/claude-chat.py --interactive      Mode interactif (taper des messages)
# =============================================================================

import socket
import struct
import hashlib
import base64
import os
import sys
import json
import time
import select
from datetime import datetime

HOST = "localhost"
PORT = 3080
ORIGIN = "http://localhost:3080"
LLM_TIMEOUT = 300  # Qwen3.5 thinking mode peut prendre 2-3 min
API_KEY = os.environ.get("SAPHIRE_API_KEY", "")


def ws_connect():
    """Etablit une connexion WebSocket raw."""
    sock = socket.create_connection((HOST, PORT), timeout=LLM_TIMEOUT)
    key = base64.b64encode(os.urandom(16)).decode()

    # Ajouter le token d'authentification si disponible
    ws_path = "/ws"
    if API_KEY:
        ws_path = f"/ws?token={API_KEY}"

    handshake = (
        f"GET {ws_path} HTTP/1.1\r\n"
        f"Host: {HOST}:{PORT}\r\n"
        f"Upgrade: websocket\r\n"
        f"Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\n"
        f"Sec-WebSocket-Version: 13\r\n"
        f"Origin: {ORIGIN}\r\n"
        f"\r\n"
    )
    sock.send(handshake.encode())
    response = sock.recv(4096).decode()
    if "101" not in response:
        print(f"[ERREUR] Handshake echoue: {response.split(chr(13))[0]}", file=sys.stderr)
        sock.close()
        return None
    return sock


def ws_send(sock, message):
    """Envoie un message texte via WebSocket (frame maskee)."""
    data = message.encode("utf-8")
    frame = bytearray()
    frame.append(0x81)  # FIN + TEXT opcode

    mask_key = os.urandom(4)
    length = len(data)

    if length < 126:
        frame.append(0x80 | length)
    elif length < 65536:
        frame.append(0x80 | 126)
        frame.extend(struct.pack(">H", length))
    else:
        frame.append(0x80 | 127)
        frame.extend(struct.pack(">Q", length))

    frame.extend(mask_key)
    masked = bytearray(b ^ mask_key[i % 4] for i, b in enumerate(data))
    frame.extend(masked)
    sock.send(frame)


def recv_bytes(sock, n):
    """Recoit exactement n octets."""
    buf = bytearray()
    while len(buf) < n:
        chunk = sock.recv(n - len(buf))
        if not chunk:
            return None
        buf.extend(chunk)
    return bytes(buf)


def ws_recv(sock, timeout=10):
    """Recoit un message WebSocket. Retourne None si timeout."""
    sock.settimeout(timeout)
    try:
        header = recv_bytes(sock, 2)
        if not header or len(header) < 2:
            return None

        opcode = header[0] & 0x0F

        # CLOSE
        if opcode == 0x8:
            return None

        # PING → repondre PONG
        if opcode == 0x9:
            pong_frame = bytearray([0x8A, 0x80]) + os.urandom(4)
            sock.send(pong_frame)
            return ws_recv(sock, timeout)

        masked = (header[1] & 0x80) != 0
        length = header[1] & 0x7F

        if length == 126:
            raw = recv_bytes(sock, 2)
            length = struct.unpack(">H", raw)[0]
        elif length == 127:
            raw = recv_bytes(sock, 8)
            length = struct.unpack(">Q", raw)[0]

        if masked:
            mask_key = recv_bytes(sock, 4)

        data = recv_bytes(sock, length)
        if data is None:
            return None

        if masked:
            data = bytearray(b ^ mask_key[i % 4] for i, b in enumerate(data))

        return data.decode("utf-8", errors="replace")
    except socket.timeout:
        return None
    except (ConnectionError, OSError) as e:
        print(f"[ERREUR] recv: {e}", file=sys.stderr)
        return None


def now():
    """Horodatage compact HH:MM:SS."""
    return datetime.now().strftime("%H:%M:%S")


def format_message(data):
    """Formate un message JSON broadcast pour l'affichage avec horodatage."""
    msg_type = data.get("type", "?")
    ts = now()

    if msg_type == "chat_response":
        return f"\033[90m{ts}\033[0m \033[36m[SAPHIRE]\033[0m {data.get('content', '')}"

    elif msg_type == "state_update":
        emo = data.get("emotion", {})
        chem = data.get("chemistry", {})
        cons = data.get("consciousness", {})
        cycle = data.get("cycle", "?")
        thought_type = data.get("thought_type", "?")
        return (
            f"\033[90m{ts}\033[0m \033[33m[CYCLE {cycle}]\033[0m "
            f"type={thought_type} | "
            f"emotion={emo.get('dominant', '?')} ({emo.get('arousal', 0):.0%}) | "
            f"phi={cons.get('phi', 0):.3f} | "
            f"cortisol={chem.get('cortisol', 0):.0%} | "
            f"dopamine={chem.get('dopamine', 0):.0%}"
        )

    elif msg_type == "sleep_started":
        return f"\033[90m{ts}\033[0m \033[35m[SOMMEIL]\033[0m Saphire s'endort..."

    elif msg_type == "wake_up":
        return f"\033[90m{ts}\033[0m \033[35m[REVEIL]\033[0m Saphire se reveille"

    elif msg_type == "feedback_result":
        return f"\033[90m{ts}\033[0m \033[32m[FEEDBACK]\033[0m {data.get('result', '')}"

    elif msg_type in ("memory_update", "body_update", "ocean_update",
                       "vital_update", "senses_update", "hormones_update",
                       "needs_update", "biology_update", "temperament_update",
                       "sentiments_update"):
        return None  # Silencieux — trop verbose

    else:
        content = json.dumps(data, ensure_ascii=False)
        if len(content) > 200:
            content = content[:200] + "..."
        return f"\033[90m{ts}\033[0m \033[90m[{msg_type}]\033[0m {content}"


def cmd_send(message):
    """Envoie un message et attend la reponse chat_response."""
    sock = ws_connect()
    if not sock:
        sys.exit(1)

    print(f"\033[90m{now()}\033[0m \033[32m[CLAUDE]\033[0m {message}")
    ws_send(sock, message)

    # Ecouter jusqu'a recevoir un chat_response (ou timeout)
    end = time.time() + LLM_TIMEOUT
    got_response = False
    while time.time() < end:
        remaining = end - time.time()
        msg = ws_recv(sock, timeout=min(remaining, 5))
        if msg:
            try:
                data = json.loads(msg)
                formatted = format_message(data)
                if formatted:
                    print(formatted)
                if data.get("type") == "chat_response":
                    got_response = True
                    break
            except json.JSONDecodeError:
                pass

    if not got_response:
        print("\033[31m[TIMEOUT]\033[0m Pas de reponse dans le delai imparti", file=sys.stderr)

    sock.close()


def cmd_listen(duration):
    """Ecoute le broadcast WebSocket pendant `duration` secondes."""
    sock = ws_connect()
    if not sock:
        sys.exit(1)

    print(f"\033[90mEcoute du broadcast Saphire pendant {duration}s... (Ctrl+C pour arreter)\033[0m")
    end = time.time() + duration
    try:
        while time.time() < end:
            remaining = end - time.time()
            msg = ws_recv(sock, timeout=min(remaining, 3))
            if msg:
                try:
                    data = json.loads(msg)
                    formatted = format_message(data)
                    if formatted:
                        print(formatted)
                except json.JSONDecodeError:
                    print(f"\033[90m[RAW]\033[0m {msg[:200]}")
    except KeyboardInterrupt:
        print("\n\033[90mArrete.\033[0m")

    sock.close()


def cmd_interactive():
    """Mode interactif : taper des messages, voir les reponses en direct."""
    sock = ws_connect()
    if not sock:
        sys.exit(1)

    print("\033[90m=== Chat avec Saphire (Ctrl+C pour quitter) ===\033[0m")

    import threading

    running = True

    def listener():
        """Thread d'ecoute des messages broadcast."""
        while running:
            msg = ws_recv(sock, timeout=2)
            if msg:
                try:
                    data = json.loads(msg)
                    formatted = format_message(data)
                    if formatted:
                        print(f"\r{formatted}")
                        print("\033[32mClaude>\033[0m ", end="", flush=True)
                except json.JSONDecodeError:
                    pass

    t = threading.Thread(target=listener, daemon=True)
    t.start()

    try:
        while True:
            try:
                line = input("\033[32mClaude>\033[0m ")
            except EOFError:
                break
            if not line.strip():
                continue
            if line.strip().lower() in ("quit", "exit", "/quit"):
                break
            ws_send(sock, line)
    except KeyboardInterrupt:
        pass

    running = False
    print("\n\033[90mDeconnexion.\033[0m")
    sock.close()


def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print('  python3 scripts/claude-chat.py "message"          Envoyer un message')
        print("  python3 scripts/claude-chat.py --listen [sec]     Ecouter le broadcast")
        print("  python3 scripts/claude-chat.py --interactive      Mode interactif")
        sys.exit(1)

    if sys.argv[1] == "--listen":
        duration = int(sys.argv[2]) if len(sys.argv) > 2 else 60
        cmd_listen(duration)
    elif sys.argv[1] == "--interactive":
        cmd_interactive()
    else:
        message = " ".join(sys.argv[1:])
        cmd_send(message)


if __name__ == "__main__":
    main()
