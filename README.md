# ELM — Experience Learning Model

> *A cognitive architecture that learns from experience, not pre-training.*

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Status: Concept / Early Prototype](https://img.shields.io/badge/Status-Concept%20%2F%20Early%20Prototype-orange)]()
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange)]()
[![Community: Open](https://img.shields.io/badge/Community-Open%20to%20All-blue)]()

---

## What is ELM?

ELM is a new class of AI model. It is **not** a Large Language Model. It does not learn from a static training dataset. It does not forget everything between sessions.

ELM learns the way living things learn — from **experience**, through **surprise**, using a **living memory** that compresses what is familiar and preserves what is new.

The core idea is simple:

- Every experience is stored as a structured memory **Pocket**
- A Pocket that is **surprising** (high Delta) is preserved at full fidelity
- A Pocket that is **familiar** (low Delta) is compressed — because the pattern is already known
- Over time, compressed Pockets form **rules**
- Rules that fail repeatedly form **principles**
- The model grows smarter not by downloading more weights but by **living longer**

ELM is designed to run **bare-metal** on physical hardware. The hardware is not infrastructure the model runs on. The hardware **is** the model's body. It senses its own CPU temperature, memory pressure, and physical sensors the same way it senses the outside world — as part of itself.

---

## Why Not Just Use an LLM?

LLMs are remarkable but architecturally frozen:

| Property | LLM | ELM |
|---|---|---|
| Learns from | Static training data | Live experience |
| Memory between sessions | None (stateless) | Persistent, compressing |
| Surprise handling | None | Core mechanism (Delta) |
| Novel vs familiar | Treated identically | Different compression levels |
| Hardware awareness | None | Proprioceptive (self-sensing) |
| Improves over time | Only by retraining | Continuously, autonomously |
| Architecture | Transformer (fixed weights) | Dynamic memory + world model |

ELM does not replace LLMs. It is a different kind of system solving a different kind of problem — **continuous learning in a physical world**.

---

## Core Principles

### 1. The Pocket
The fundamental unit of memory. Every experience — a sensor reading, a sound, a text input, a motor action — becomes a Pocket containing:
- What the situation was (State)
- What was done (Action)
- What resulted (Outcome)
- How surprising it was (Delta)

### 2. Dynamic Compression
Pockets are not all stored equally. Compression level is driven by similarity to existing memories:

- **Novel experience** → stored at full precision (float32)
- **Familiar pattern** → compressed (float16 → int8 → binary)
- **Identical to known rule** → merged into existing Tier 1 rule

This means ELM gets **more efficient as it learns more** — the opposite of systems that require more compute as they grow.

### 3. The Delta
The difference between what ELM predicted would happen and what actually happened. Delta is the engine of everything:
- High Delta = surprise = preserve memory = update world model
- Low Delta = expected = compress memory = reinforce existing rule

### 4. Hardware as Self
ELM reads its own hardware registers directly — CPU cycles, thermal sensors, memory pressure, motor current. These are not system metrics. They are proprioception. ELM knows when it is hot, tired, damaged, or degraded the same way it knows when a wall is ahead of it.

---

## Architecture Overview

```
REALITY (sensors, text, audio, video, hardware)
  ↓
Normalization + Signal Binding
  ↓
Universal Experience Encoder (UEE)
  → Modality-mapped latent vectors
  ↓
Prediction Engine  ←─────────────────────────┐
  ↓ (Delta calculated here)                  │
Quantization Decision                        │
  ↓                                          │
Pocket Storage (Tier 0)                      │
  ↓                                          │
Cluster Engine                               │
  ↓                                          │
Compression → Tier 1 Rules                   │
  ↓                                          │
Abstraction → Tier 2 Principles              │
  ↓                                          │
World Model ─────────────────────────────────┘
  ↑
Reasoning Layer
  ↑
[Goals → Plans → Monitor → Explore]
  ↑
AGENT BEHAVIOR
```

Full architecture documentation: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

---

## Memory Tiers

| Tier | Name | What it stores | Compression | Deletable? |
|---|---|---|---|---|
| 0 | Raw Experience | Full SAO tuple + Delta | None (float32) | Yes, on decay |
| 1 | Pattern Rule | Centroid + variance + condition map | Moderate (int8) | No |
| 2 | Abstract Principle | Meta-rule about rule structure | Heavy (binary) | No |

---

## Target Hardware (MVP)

ELM boots bare-metal. No operating system.

**Recommended MVP platform: Raspberry Pi 4 (4GB+)**

```
Hardware:
  Raspberry Pi 4 (4GB minimum)
  BME280    — temperature, humidity, pressure
  MPU-6050  — accelerometer, gyroscope
  VL53L0X   — distance sensor
  NVMe HAT  — persistent memory storage
```

The Pi is not running ELM. The Pi **is** ELM.

Full hardware spec: [docs/HARDWARE.md](docs/HARDWARE.md)

---

## Project Status

This is an **early-stage community research project**. The architecture is specified. The prototype is being built.

Current milestone: **MVP — The Thermal Maze**

A bare-metal Rust implementation of the core memory loop, tested in a bounded 2D grid environment with sensor input, Delta calculation, and Tier 0 → Tier 1 compression.

See [docs/MVP.md](docs/MVP.md) for the full MVP specification.

---

## Getting Started

```bash
git clone https://github.com/muah1987/ELM.git
cd ELM
```

Prerequisites: Rust nightly, cross-compilation target for aarch64 (Raspberry Pi 4)

```bash
rustup target add aarch64-unknown-none
```

Build and flash instructions: [docs/BUILDING.md](docs/BUILDING.md)

---

## Documentation

| Document | Description |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Full cognitive architecture specification |
| [docs/MEMORY.md](docs/MEMORY.md) | Pocket structure, compression tiers, similarity engine |
| [docs/PREDICTION.md](docs/PREDICTION.md) | World model and prediction engine |
| [docs/HARDWARE.md](docs/HARDWARE.md) | Hardware sensing, proprioception, bare-metal boot |
| [docs/MVP.md](docs/MVP.md) | Minimum viable prototype specification |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | How to contribute |
| [docs/ROADMAP.md](docs/ROADMAP.md) | What we are building toward |
| [SPEC_v0.4.md](SPEC_v0.4.md) | Full formal specification (current version) |

---

## Contributing

ELM is a free, open, community project. All skill levels welcome.

You do not need to be an AI researcher. You do not need a PhD. If you can write Rust, design hardware, think clearly about memory systems, or just ask good questions — you belong here.

Read [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) to get started.

---

## License

MIT License. Free for all. See [LICENSE](LICENSE).

---

## Origin

This architecture was developed through an open collaborative design session. The core ideas — dynamic compression driven by similarity, Delta as the engine of learning, hardware as proprioception, the Sensorimotor bootstrapping sequence — emerged from first principles rather than from existing literature.

Several concepts independently converge with established research (predictive coding, episodic memory, Piaget's sensorimotor stage, POMDP theory) but were derived organically. This is noted not to claim priority but to validate that the architecture is grounded in real cognitive science.

We are building something new. Come build it with us.
