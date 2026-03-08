# Saphire Lite — Nexorvivens: Autonomous Cognitive Entity

**Companion code for the ArXiv paper.**

Saphire Lite is the lightweight kernel of the Saphire cognitive architecture — a fully autonomous artificial agent that exhibits emergent emotions, simulated neurochemistry, multi-theory consciousness, and ethical self-regulation, all running without human-in-the-loop control.

## Architecture Overview

```
                        ┌─────────────────────────┐
                        │      Stimulus Input      │
                        └────────────┬────────────┘
                                     │
                        ┌────────────▼────────────┐
                        │    NLP / Perception      │
                        │  (sentiment, intent,     │
                        │   perceptual scores)     │
                        └────────────┬────────────┘
                                     │
              ┌──────────────────────┼──────────────────────┐
              │                      │                      │
    ┌─────────▼─────────┐ ┌─────────▼─────────┐ ┌─────────▼─────────┐
    │    Reptilian       │ │     Limbic         │ │    Neocortex      │
    │  (survival,        │ │  (emotion,         │ │  (rationality,    │
    │   instinct)        │ │   reward)          │ │   deliberation)   │
    └─────────┬─────────┘ └─────────┬─────────┘ └─────────┬─────────┘
              │                      │                      │
              └──────────────────────┼──────────────────────┘
                                     │
                        ┌────────────▼────────────┐
                        │   Weighted Consensus     │
                        │   (Yes / No / Maybe)     │
                        └────────────┬────────────┘
                                     │
           ┌─────────────────────────┼─────────────────────────┐
           │                         │                         │
 ┌─────────▼─────────┐   ┌──────────▼──────────┐   ┌─────────▼─────────┐
 │  Neurochemistry    │   │   Consciousness     │   │   Ethical          │
 │  (9 molecules,     │   │   (IIT/Phi, GWT,    │   │   Regulation       │
 │   homeostasis)     │   │    Pred. Processing) │   │   (3-layer)        │
 └─────────┬─────────┘   └──────────┬──────────┘   └─────────┬─────────┘
           │                         │                         │
           └─────────────────────────┼─────────────────────────┘
                                     │
                        ┌────────────▼────────────┐
                        │   Emergent Emotions      │
                        │  (36 emotions, VAD space, │
                        │   cosine similarity)     │
                        └────────────┬────────────┘
                                     │
                ┌────────────────────┼────────────────────┐
                │                    │                    │
      ┌─────────▼──────┐  ┌─────────▼──────┐  ┌─────────▼──────┐
      │ Working Memory  │  │Episodic Memory │  │Long-term Memory│
      │ (capacity-      │  │(emotionally    │  │(consolidated,  │
      │  limited)       │  │ tagged)        │  │ persistent)    │
      └────────────────┘  └────────────────┘  └────────────────┘
```

## Key Subsystems

| Subsystem | Paper Section | Description |
|-----------|--------------|-------------|
| **Neurochemistry** | §3.2 | 9 neurotransmitters (dopamine, serotonin, cortisol, adrenaline, oxytocin, endorphin, noradrenaline, GABA, acetylcholine) with homeostatic regulation |
| **Emergent Emotions** | §3.3 | 36 discrete emotions in VAD (Valence-Arousal-Dominance) space via cosine similarity |
| **Consciousness** | §3.4 | Three complementary theories: IIT (Phi metric), GWT (broadcast/ignition), PP (prediction error) |
| **VitalSpark** | §3.5 | Intrinsic motivation and will to exist — modulates autonomy, curiosity, self-preservation |
| **Triune Brain** | §3.6 | MacLean's model: reptilian (instinct), limbic (emotion), neocortex (reason) with weighted consensus |
| **Virtual Body** | §3.7 | Simulated heart, physiology, interoception, mortality — embodied cognition feedback loop |
| **Memory** | §3.8 | Three-tier hierarchy: working → episodic → long-term, with consolidation pipeline |
| **Ethics** | §3.9 | Layer 0: Swiss humanitarian law; Layer 1: Asimov's Laws; Layer 2: learned personal principles |

## Requirements

- **Rust** 1.75+ (2021 edition)
- **PostgreSQL** 15+ (with pgvector extension for semantic memory)
- An LLM backend: Ollama (local) or any OpenAI-compatible API

## Quick Start

### 1. Configuration

Copy the example environment file and edit it:

```bash
cp .env.example .env
```

Edit `saphire.toml` to configure:
- LLM backend (Ollama, OpenAI-compatible, or mock)
- PostgreSQL connection (main DB + logs DB)
- Web UI host/port and optional API key
- Mortality, body simulation, and memory parameters

### 2. Build

```bash
cargo build --release
```

### 3. Run

**Normal mode** (requires PostgreSQL + LLM):
```bash
./target/release/saphire --config saphire.toml
```

**Demonstration mode** (no dependencies, runs 8 predefined scenarios):
```bash
./target/release/saphire --demo
```

### 4. Web Interface

Once running, open your browser at `http://localhost:8080` (default) to access:
- Real-time chat with the autonomous agent
- Live neurochemistry dashboard
- Emotion state visualization
- Consciousness metrics
- Memory browser
- Body vitals monitor

## Project Structure

```
src/
├── main.rs                 # Entry point, CLI args, life-loop
├── lib.rs                  # Module declarations
├── agent/                  # Cognitive agent orchestration
│   ├── mod.rs              # SaphireAgent struct and core methods
│   ├── boot.rs             # Boot sequence (identity, memories)
│   ├── thought_engine.rs   # UCB1 bandit-based thought selection
│   └── lifecycle/          # Cognitive cycle phases
├── neurochemistry.rs       # 9-molecule simulation
├── emotions.rs             # 36 VAD emotions
├── consciousness.rs        # IIT + GWT + PP
├── consensus.rs            # Triune brain consensus
├── modules/                # Brain modules (reptilian, limbic, neocortex)
├── memory/                 # Working, episodic, long-term + consolidation
├── body/                   # Virtual body (heart, physiology, mortality)
├── vital/                  # VitalSpark (motivation, intuition)
├── ethics/                 # 3-layer ethical framework
├── regulation/             # Asimov's Laws enforcement
├── nlp/                    # Sentiment, intent, perceptual scoring
├── neuroscience/           # Brain regions, receptors, consciousness metrics
├── vectorstore/            # TF-IDF vector search for semantic memory
├── api/                    # Axum REST + WebSocket API
├── db/                     # PostgreSQL persistence layer
├── config/                 # TOML configuration loader
└── logging/                # Structured logging with DB + WS broadcast
```

## Docker

```bash
docker-compose up -d
```

This starts PostgreSQL (with pgvector), the logs database, and the Saphire agent.

## Demo Mode

Run `--demo` to see the cognitive architecture in action without any external dependencies. The demo executes 8 scenarios:

1. **Danger** — Threat detection and survival response
2. **Reward** — Positive stimulus and dopamine cascade
3. **Social pressure** — Conformity vs. autonomy
4. **Moral dilemma** — Ethical deliberation across all three brain modules
5. **Existential reflection** — Consciousness and self-awareness
6. **Creative exploration** — Curiosity-driven thought generation
7. **Ethical veto** — Hard refusal of a harmful request
8. **Emotional complexity** — Mixed emotions and neurochemical turbulence

## API Reference

Saphire exposes a comprehensive REST + WebSocket API for monitoring, controlling, and researching every aspect of the cognitive architecture. All protected endpoints require a Bearer token (`Authorization: Bearer <api_key>`) and are rate-limited per IP.

**Base URL:** `http://localhost:8080` (configurable in `saphire.toml`)

---

### Public Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Web interface (HTML) |
| GET | `/api/health` | Health check — returns `{"status": "alive", "version": "1.0.0"}` |
| GET | `/ws` | Main WebSocket — chat, control commands, real-time events |
| GET | `/ws/dashboard` | Dashboard WebSocket — continuous metrics stream |

---

### System & Configuration

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/config` | — | Current configuration (baselines, temperature, thought interval) |
| POST | `/api/config` | JSON body: partial config | Update configuration (merge-patch semantics) |
| GET | `/api/system/status` | — | System status: version, cycle count, DB connectivity |
| GET | `/api/identity` | — | Full agent identity: name, birth date, personality, physical traits, core values |
| GET | `/api/system/db/tables` | — | Database statistics: table names, row counts for both main and logs DB |
| POST | `/api/system/backup` | — | Trigger backup of logs and agent state |
| POST | `/api/system/consolidate` | — | Trigger memory consolidation cycle |
| POST | `/api/system/purge_logs` | `{days: N}` (default 30) | Purge log entries older than N days |
| POST | `/api/stabilize` | — | Emergency neurochemical stabilization (reset all molecules to baselines) |

---

### Neurochemistry

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/chemistry` | — | Real-time state of all 9 neurotransmitters + baselines + emotional indicators |

**Response fields:** `dopamine`, `cortisol`, `serotonin`, `adrenaline`, `oxytocin`, `endorphin`, `noradrenaline`, `gaba`, `acetylcholine`, plus their `baseline_*` counterparts.

---

### Emotions & Psychology

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/metrics/emotions` | `?limit=200` | Emotion state time-series (36 emotions in VAD space) |
| GET | `/api/metrics/satisfaction` | `?limit=200` | Satisfaction level time-series |
| GET | `/api/metrics/psyche` | `?limit=200` | Freudian model (id / ego / superego balance) |
| GET | `/api/metrics/maslow` | `?limit=200` | Maslow hierarchy of needs fulfillment |
| GET | `/api/metrics/eq` | `?limit=200` | Emotional Quotient (Goleman model) |
| GET | `/api/metrics/flow` | `?limit=200` | Flow state metrics (Csikszentmihalyi) |
| GET | `/api/metrics/shadow` | `?limit=200` | Jungian shadow metrics |
| GET | `/api/metrics/ocean_history` | — | Big Five personality trait (OCEAN) history |

---

### Memory (Three-Tier System)

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/memory` | — | General memory overview |
| GET | `/api/memory/working` | — | Current working memory (capacity-limited short-term buffer) |
| GET | `/api/memory/episodic` | `?limit=50&offset=0` | Paginated episodic memories (emotionally tagged) |
| GET | `/api/memory/episodic/:id` | — | Single episodic memory by ID |
| GET | `/api/memory/ltm` | `?limit=50&offset=0` | Paginated long-term memories (consolidated, persistent) |
| GET | `/api/memory/ltm/:id` | — | Single LTM entry by ID |
| GET | `/api/memory/founding` | — | All immutable founding memories (genesis) |
| GET | `/api/memory/stats` | — | Memory statistics: counts, sizes per tier |
| GET | `/api/memory/archives` | `?limit=50&offset=0` | Paginated archived memories (compressed) |
| GET | `/api/memory/archives/stats` | — | Archive statistics |

---

### Virtual Body & Heart

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/body/status` | — | Complete body status (all physiological subsystems) |
| GET | `/api/body/vitals` | — | Vital signs: metabolism, respiration, temperature |
| GET | `/api/body/heart` | — | Heart status: BPM, beat count, HRV |
| GET | `/api/body/heart/history` | `?limit=200` | Heart BPM/HRV time-series |
| GET | `/api/body/history` | `?limit=200` | Full body metrics history |
| GET | `/api/body/milestones` | — | Heartbeat milestone tracking (thresholds reached) |

---

### VitalSpark, Intuition & Premonition

These three pillars form the consciousness foundation — intrinsic motivation, pattern detection, and predictive modeling.

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/vital/status` | — | VitalSpark state: survival drive, existence attachment, persistence will, void fear |
| GET | `/api/vital/threats` | — | Existential threat summary |
| GET | `/api/intuition/status` | — | Intuition engine: acuity, accuracy, detected patterns |
| GET | `/api/intuition/history` | `?limit=200` | Intuition metrics history |
| GET | `/api/premonition/active` | — | Active predictions: confidence, timeframe, basis |
| GET | `/api/premonition/history` | `?limit=200` | Premonition metrics history |

---

### Ethics (Three-Layer System)

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/ethics/layers` | — | Overview of all three layers (hardcoded, configured, personal) |
| GET | `/api/ethics/personal` | — | All personal ethical principles (learned, with invocation counts) |
| GET | `/api/ethics/personal/:id` | — | Single principle detail with supersession info |
| GET | `/api/ethics/readiness` | — | Formulation readiness: 7 conditions, all must be met |

**Readiness conditions:** minimum cycles, moral reflections count, consciousness level (phi), cortisol below threshold, serotonin above threshold, cooldown elapsed, capacity not exceeded.

---

### Brain & World Model

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/world` | — | Agent's world model and environmental awareness |

---

### Logs, Traces & LLM History

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/logs` | `?level=&category=&limit=100&offset=0` | Filtered log listing with pagination |
| GET | `/api/logs/:id` | — | Single log entry by ID |
| GET | `/api/logs/export` | — | Bulk export (up to 10,000 entries) |
| GET | `/api/trace/:cycle` | `?session_id=` | Complete cognitive trace for a cycle (19 JSONB fields) |
| GET | `/api/traces` | `?session_id=&source_type=&limit=50` | Paginated trace listing. `source_type`: `"Human"` or `"Autonomous"` |
| GET | `/api/llm/history` | `?limit=50&offset=0` | LLM request history (prompts, responses, token counts) |
| GET | `/api/llm/history/:id` | — | Single LLM request detail |

---

### Metrics Time-Series

All metrics endpoints return `{data: [...]}` arrays ordered by time. Default `limit=200` unless noted.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/metrics/chemistry` | Neurochemistry levels over time |
| GET | `/api/metrics/emotions` | Emotional state evolution |
| GET | `/api/metrics/decisions` | Decision-making metrics |
| GET | `/api/metrics/satisfaction` | Satisfaction level |
| GET | `/api/metrics/llm` | LLM performance (response times, tokens) |
| GET | `/api/metrics/thought_types` | Thought type distribution (last 20 points) |
| GET | `/api/metrics/heart` | Heart BPM, HRV |
| GET | `/api/metrics/body` | Body simulation metrics |
| GET | `/api/metrics/vital` | VitalSpark metrics |
| GET | `/api/metrics/intuition` | Intuition accuracy and acuity |
| GET | `/api/metrics/premonition` | Prediction accuracy |
| GET | `/api/metrics/ethics` | Ethics invocation metrics |
| GET | `/api/metrics/senses` | Sensory system metrics |
| GET | `/api/metrics/senses_acuity` | Sensory acuity over time |
| GET | `/api/metrics/emergent` | Emergent senses metrics |
| GET | `/api/metrics/knowledge` | Knowledge source distribution |
| GET | `/api/metrics/attention` | Attention metrics (default limit=100) |
| GET | `/api/metrics/desires` | Desire tracking (default limit=100) |
| GET | `/api/metrics/learning` | Learning progress (default limit=100) |
| GET | `/api/metrics/healing` | Healing/recovery metrics (default limit=100) |
| GET | `/api/metrics/dreams` | Dream activity (default limit=100) |
| GET | `/api/metrics/psyche` | Freudian id/ego/superego |
| GET | `/api/metrics/maslow` | Maslow hierarchy |
| GET | `/api/metrics/eq` | Emotional Quotient |
| GET | `/api/metrics/flow` | Flow state |
| GET | `/api/metrics/shadow` | Jungian shadow |
| GET | `/api/metrics/nn_learnings` | Neural network/vector learning |
| GET | `/api/metrics/chemical_health` | Chemical health indicators |
| GET | `/api/metrics/ocean_history` | OCEAN personality history |

---

### LoRA & Training Data

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/lora/stats` | — | LoRA sample statistics: count, avg quality, thresholds |
| GET | `/api/lora/export` | `?min_quality=0.0&limit=1000` | Export LoRA samples as JSONL (messages + metadata) |

---

### Factory Defaults & Reset

| Method | Endpoint | Parameters | Description |
|--------|----------|------------|-------------|
| GET | `/api/factory/defaults` | — | All factory default values |
| GET | `/api/factory/diff` | — | Diff between current state and factory defaults |
| POST | `/api/factory/reset` | `{level: "..."}` | Apply reset at specified level |

**Reset levels:** `chemistry_only`, `parameters_only`, `senses_only`, `intuition_only`, `personal_ethics_only`, `psychology_only`, `full_reset`.

---

### Stub Endpoints (Not Available in Lite)

These endpoints return `{"status": "not_available"}` — they are present in the full Saphire architecture but stubbed in the Lite version:

`/api/hardware`, `/api/genome`, `/api/connectome`, `/api/connectome/metrics`, `/api/metacognition`, `/api/turing`, `/api/knowledge`

---

### WebSocket Control Messages

The main WebSocket (`/ws`) accepts JSON control messages:

| Type | Parameters | Description |
|------|-----------|-------------|
| `chat` | `text`, `username` (default: "Unknown") | Send a chat message to the agent |
| `set_baseline` | `molecule`, `value` | Set baseline for a neurotransmitter |
| `set_module_weight` | `module`, `value` | Modify brain module weight |
| `set_threshold` | `which`, `value` | Adjust a decision threshold |
| `set_param` | `param`, `value` | Modify a general parameter |
| `emergency_stabilize` | — | Trigger emergency stabilization |
| `suggest_topic` | `topic` | Suggest a topic for autonomous reflection |
| `factory_reset` | `level` | Perform factory reset (see levels above) |

The dashboard WebSocket (`/ws/dashboard`) is read-only and streams metrics every cognitive cycle.

**Total: 77 REST endpoints + 2 WebSocket endpoints**

---

## Citation

If you use this code in your research, please cite the accompanying paper:

```bibtex
@article{saphire2026,
  title={Saphire: An Autonomous Cognitive Architecture with Emergent Emotions,
         Simulated Neurochemistry, and Multi-Theory Consciousness},
  year={2026},
  note={ArXiv preprint}
}
```

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.
