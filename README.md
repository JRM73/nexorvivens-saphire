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
