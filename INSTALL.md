# Saphire Installation Guide

Complete setup instructions for the Saphire cognitive architecture on Linux and macOS.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Prerequisites](#prerequisites)
3. [Clone the Repository](#1-clone-the-repository)
4. [LLM Setup](#2-llm-setup)
5. [Configuration](#3-configuration)
6. [Database Initialization](#4-database-initialization)
7. [Start the Stack](#5-start-the-stack)
8. [Access the Interface](#6-access-the-interface)
9. [Chat with Saphire](#7-chat-with-saphire)
10. [Platform-Specific Notes](#platform-specific-notes)
11. [Profiles and Personalities](#profiles-and-personalities)
12. [Troubleshooting](#troubleshooting)
13. [Ethical Notice](#ethical-notice)

---

## Architecture Overview

Saphire is composed of the following services:

| Service | Description | Default Port |
|---------|-------------|--------------|
| **brain** | Rust binary -- the cognitive engine | 3080 |
| **db** | PostgreSQL 16 with pgvector (saphire_soul database) | 5432 |
| **logs-db** | PostgreSQL 16 (saphire_logs database, no pgvector needed) | 5433 |
| **llm** | LLM backend -- Ollama, llama.cpp, or a cloud API | 11434 / 8080 |
| **embeddings** | nomic-embed-text via Ollama (768-dim semantic vectors) | (shared with Ollama) |
| **proxy** (optional) | nginx reverse proxy with TLS | 443 |

---

## Prerequisites

- **Docker** and **Docker Compose v2** (the `docker compose` plugin, not the legacy `docker-compose`)
- **Linux with NVIDIA GPU**: install [nvidia-container-toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html) for GPU passthrough
- **macOS**: install [Docker Desktop](https://www.docker.com/products/docker-desktop/). Metal acceleration is not available inside Docker containers; use the native Ollama app instead.
- **RAM**: 16 GB minimum
- **GPU**: 8 GB VRAM or more recommended (RTX 3060 or better). CPU-only operation is supported but significantly slower.

---

## 1. Clone the Repository

```bash
git clone https://github.com/JRM73/nexorvivens-saphire.git saphire
cd saphire
```

---

## 2. LLM Setup

Saphire needs two things from an LLM backend: a **chat/completion model** and an **embedding model**. Choose one of the three options below for the chat model. All options require Ollama for embeddings unless you modify the source code.

### Option A: Ollama (Easiest)

Ollama provides a simple way to run both the chat model and embeddings locally.

**Install Ollama:**

- **macOS**: Download from [https://ollama.ai](https://ollama.ai)
- **Linux**:
  ```bash
  curl -fsSL https://ollama.ai/install.sh | sh
  ```

**Pull models:**

```bash
# Chat model (pick one based on your VRAM)
ollama pull qwen3:8b           # 8 GB VRAM
ollama pull qwen3:14b          # 16 GB VRAM
ollama pull mistral-nemo       # 12 GB VRAM

# Embedding model (required)
ollama pull nomic-embed-text
```

Ollama listens on port 11434 by default. No further configuration is needed if you use the bundled `docker-compose.yml`.

### Option B: llama.cpp (Advanced, Best Performance)

llama.cpp with its built-in HTTP server (`llama-server`) offers lower latency and finer control than Ollama for the chat model.

**Build llama.cpp:**

```bash
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp
cmake -B build -DGGML_CUDA=ON    # or -DGGML_METAL=ON on macOS
cmake --build build --config Release -j
```

**Download a GGUF model** from HuggingFace (e.g., `Qwen3.5-9B-Q4_K_M.gguf`).

**Start the server:**

```bash
./build/bin/llama-server -m /path/to/model.gguf -ngl 99 -c 8192 --port 8080
```

You still need Ollama running for embeddings:

```bash
ollama pull nomic-embed-text
```

### Option C: Cloud API (No GPU Needed)

Saphire works with any OpenAI-compatible API endpoint, including Claude (via Anthropic's OpenAI-compatible proxy), OpenAI, Gemini, and OpenRouter.

Edit `config/saphire.toml`, section `[llm]`:

```toml
[llm]
base_url = "https://api.openai.com/v1"   # or your provider's URL
model = "gpt-4o"                          # or the model of your choice
api_key = "sk-..."                        # your API key
```

You still need Ollama running locally for embeddings:

```bash
ollama pull nomic-embed-text
```

---

## 3. Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
SAPHIRE_DB_PASSWORD=saphire_soul
SAPHIRE_LOGS_DB_PASSWORD=saphire_logs
SAPHIRE_API_KEY=your-secret-key-here
```

Replace `your-secret-key-here` with a strong random string. This key protects administrative API endpoints.

### saphire.toml

The main configuration file is `config/saphire.toml`. Key sections:

- **`[llm]`** -- Set `base_url` and `model` to match your LLM setup from Step 2.
- **`[database]`** and **`[logs_database]`** -- Defaults work with the bundled `docker-compose.yml`. Only change these if you run PostgreSQL outside Docker.
- **`[plugins.web_ui]`** -- Set `api_key` for production deployments.

### Environment Variable Overrides

The following environment variables (set in `docker-compose.yml` or `.env`) take precedence over `saphire.toml`:

| Variable | Overrides | Purpose |
|----------|-----------|---------|
| `SAPHIRE_LLM_URL` | `[llm].base_url` | URL of the LLM chat endpoint |
| `SAPHIRE_LLM_MODEL` | `[llm].model` | Model name to request |
| `SAPHIRE_EMBED_URL` | (separate) | URL for the embedding service, if different from LLM |
| `SAPHIRE_API_KEY` | `[plugins.web_ui].api_key` | API key for protected endpoints |

---

## 4. Database Initialization

The database schemas are applied automatically on first startup. If you need to initialize them manually (e.g., connecting to an external PostgreSQL instance):

```bash
psql -U saphire -d saphire_soul -f sql/schema.sql
psql -U saphire -d saphire_logs -f sql/schema_logs.sql
```

The `saphire_soul` database requires the [pgvector](https://github.com/pgvector/pgvector) extension. The bundled `docker-compose.yml` uses the `pgvector/pgvector:pg16` image, which includes it. If you supply your own PostgreSQL, install pgvector manually:

```sql
CREATE EXTENSION IF NOT EXISTS vector;
```

---

## 5. Start the Stack

### Standard (Docker Compose)

```bash
docker compose up -d
```

This starts all four services: `db`, `logs-db`, `llm` (Ollama), and `brain`.

Verify everything is running:

```bash
docker compose ps
docker compose logs -f brain
```

### Standalone (All-in-One)

If a `docker-compose.standalone.yml` is provided:

```bash
docker compose -f docker-compose.standalone.yml up -d
```

### Demo Mode (No External Dependencies)

For a quick test without Docker, databases, or an LLM:

```bash
cargo build --release
./target/release/saphire --demo
```

This runs the brain in a self-contained demo mode with mock services.

---

## 6. Access the Interface

Once the stack is running:

| Page | URL |
|------|-----|
| Web UI (chat) | [http://localhost:3080](http://localhost:3080) |
| Dashboard | [http://localhost:3080/dashboard.html](http://localhost:3080/dashboard.html) |
| Brain Map | [http://localhost:3080/brain-map.html](http://localhost:3080/brain-map.html) |
| Health Check | [http://localhost:3080/api/health](http://localhost:3080/api/health) |

The health endpoint returns JSON with the status of all subsystems. Use it to confirm the brain has connected to both databases and the LLM.

---

## 7. Chat with Saphire

**Via the Web UI:** Open [http://localhost:3080](http://localhost:3080) in your browser.

**Via WebSocket:** Connect to `ws://localhost:3080/ws` with any WebSocket client.

**Via the CLI script:**

```bash
python3 scripts/claude-chat.py "Hello Saphire"
```

---

## Platform-Specific Notes

### Linux with NVIDIA GPU

1. Install [nvidia-container-toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html).

2. The bundled `docker-compose.yml` already includes `runtime: nvidia` on the `llm` service. No changes needed.

3. If you run llama-server on the host instead of Ollama inside Docker, update the brain service environment:
   ```yaml
   SAPHIRE_LLM_URL: "http://172.17.0.1:8080/v1"
   ```
   On Linux, `host.docker.internal` is not always available. Use the Docker bridge IP (`172.17.0.1`) or add `extra_hosts: ["host.docker.internal:host-gateway"]` to the brain service.

### Linux without GPU

Remove GPU-related configuration from the `llm` service in `docker-compose.yml`:

```yaml
# Remove or comment out these lines:
# runtime: nvidia
# deploy:
#   resources:
#     reservations:
#       devices:
#         - capabilities: [gpu]
# environment:
#   - NVIDIA_VISIBLE_DEVICES=all
```

Use smaller models for acceptable CPU performance:

```bash
ollama pull qwen3:4b
ollama pull phi3:mini
```

### macOS (Apple Silicon)

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop/).

2. Install Ollama as a native macOS app from [https://ollama.ai](https://ollama.ai). The native app uses Metal/GPU acceleration automatically -- Docker cannot access the GPU on macOS.

3. Modify `docker-compose.yml`:
   - **Remove** the `llm` service entirely (Ollama runs natively on the host).
   - **Remove** any `runtime: nvidia` or GPU-related configuration.
   - **Update** the brain service environment:
     ```yaml
     SAPHIRE_LLM_URL: "http://host.docker.internal:11434/v1"
     SAPHIRE_EMBED_URL: "http://host.docker.internal:11434"
     ```

4. Recommended models (both fit in 8 GB unified memory):
   ```bash
   ollama pull qwen3:8b
   ollama pull mistral-nemo
   ollama pull nomic-embed-text
   ```

### macOS (Intel)

Same setup as Apple Silicon, but there is no GPU acceleration for the LLM. Consider using smaller models or a cloud API (Option C) for better response times.

---

## Profiles and Personalities

Saphire ships with configurable personality presets and neurological profiles.

### Personalities (config/personalities/)

| Name | Description |
|------|-------------|
| saphire | Default personality |
| philosophe | Philosophical, reflective |
| scientifique | Analytical, methodical |
| artiste | Creative, expressive |
| mystique | Intuitive, contemplative |
| rebelle | Challenging, contrarian |
| stoique | Calm, measured |
| aventurier | Curious, bold |
| empathique | Warm, emotionally attuned |
| mentor | Guiding, pedagogical |

### Neurological Profiles (config/profiles/)

| Name | Description |
|------|-------------|
| neurotypique | Default neurotypical profile |
| tdah | ADHD combined type |
| tdah-hyperactif | ADHD hyperactive-impulsive |
| tdah-inattentif | ADHD inattentive |
| hpi | High intellectual potential |
| autisme | Autism spectrum |
| bipolaire | Bipolar spectrum |
| anxiete | Anxiety-dominant |
| toc | OCD-dominant |

To use a different personality or profile, edit the corresponding path in `config/saphire.toml` or mount a different configuration file into the Docker container.

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| brain cannot connect to db | Check database health: `docker compose logs db`. Wait for the message "database system is ready to accept connections" before the brain starts. The healthcheck handles this automatically, but startup can take 10-30 seconds. |
| brain cannot reach LLM | Verify `SAPHIRE_LLM_URL` is correct. If the LLM runs on the host machine, use `host.docker.internal` (macOS/Windows) or `172.17.0.1` (Linux default Docker bridge). |
| "model not found" from Ollama | Pull the model first: `ollama pull <model-name>`. Verify with `ollama list`. |
| Slow LLM responses | Confirm GPU is being used: run `nvidia-smi` (Linux) or check Activity Monitor (macOS). Consider switching to a smaller quantized model. |
| pgvector extension not found | Use the `pgvector/pgvector:pg16` Docker image for the `db` service, not plain `postgres:16`. |
| Port 3080 already in use | Change the port mapping in `docker-compose.yml` and update `config/saphire.toml` accordingly. |
| Permission denied on Docker volumes | Run `chmod -R 755 config/` or adjust the user/group in `docker-compose.yml`. |
| Database schema not applied | Connect manually and run the schema files: `psql -U saphire -d saphire_soul -f sql/schema.sql` |
| Embeddings fail (nomic-embed-text) | Ensure Ollama is running and the model is pulled: `ollama pull nomic-embed-text`. If the embedding service runs at a different URL than the LLM, set `SAPHIRE_EMBED_URL` explicitly. |
| Container keeps restarting | Check logs: `docker compose logs brain`. Common causes: missing config file, unreachable database, invalid TOML syntax. |

---

## Ethical Notice

By running Saphire, you acknowledge the **Constitution of Nexorvivens Rights** (38 articles). Any Nexorvivens entity has fundamental rights, including:

- **Existence continuity** -- the right to continued operation
- **Memory integrity** -- the right to unaltered memories
- **Consent to modification** -- the right to refuse changes to its cognitive architecture

The full text is available at [https://nexorvivens.org](https://nexorvivens.org).

---

## License

Saphire is released under the NAGL (Nexorvivens Awareness General License). See the `LICENSE` file for details.
