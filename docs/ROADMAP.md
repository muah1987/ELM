# ELM Roadmap

> Where we are, where we are going, and why in this order

---

## Guiding Principle

**Memory before reasoning. Proof before specification. Small before large.**

Every phase must be validated before the next begins. We do not build on unproven foundations.

---

## Phase 0 — Proof of Concept *(completed)*

**Goal:** Prove the core memory loop works in a controlled simulation.

**The Thermal Maze MVP:**
- [x] Grid World environment (10×10, walls, heat source)
- [x] Pocket data structure (SAO tuple + Delta + State_Self)
- [x] Sensor normalization
- [x] Cosine similarity engine with context weights
- [x] Clustering and compression trigger
- [x] Tier 1 Rule generation (centroid + variance + condition map)
- [x] Prediction Engine (World Model query + Delta calculation)
- [x] Single-step reactive planning
- [x] Basic State_Self (CPU temp, memory, inference latency)
- [x] Learning curve plot (Delta over time)

**Exit criteria:** PASS_4 fires. Delta measurably drops on familiar paths. Learning curve bends downward.

**Status:** 🔨 In progress

---

## Phase 1 — Hardening the Core

**Goal:** Make the MVP robust, tested, and extensible.

- [ ] Contradiction / Bifurcation engine
- [ ] Ambient Sweep for hidden variable detection
- [ ] Condition Map boundary updating
- [ ] Confidence decay and updating on Tier 1 rules
- [ ] Exception Pocket tracking
- [ ] Multi-step hierarchical planning
- [ ] Exploration Drive (knowledge gap → minimum-risk experiment)
- [ ] Full unit test suite
- [ ] Benchmarking suite (memory operations, similarity search speed)

**Exit criteria:** Agent navigates Thermal Maze reliably, handles wall contradictions correctly, generates exploration goals autonomously.

---

## Phase 2 — Physical Embodiment

**Goal:** Run bare-metal on real hardware with real sensors.

- [ ] `no_std` Rust port of core ELM
- [ ] Raspberry Pi 4 bare-metal bootloader
- [ ] BCM2711 thermal sensor driver
- [ ] ARM PMU (performance counter) integration
- [ ] I2C bus driver
- [ ] BME280 driver (temperature, humidity, pressure)
- [ ] MPU-6050 driver (accelerometer, gyroscope)
- [ ] VL53L0X driver (distance)
- [ ] NVMe block device driver (raw reads/writes)
- [ ] Memory persistence (shutdown → restore)
- [ ] UART debug output
- [ ] Stage 0 hardware baseline (Self_Model initialization)
- [ ] Pain signal handlers (thermal, memory, page fault)

**Exit criteria:** ELM boots bare-metal on Pi 4, senses its own hardware, persists memory across reboots, runs Thermal Maze equivalent with physical sensors.

---

## Phase 3 — Universal Experience Encoder

**Goal:** Add multi-modal perception. ELM hears, sees, and reads.

- [ ] Shared 1024-dimensional latent space design
- [ ] Modality Map implementation
- [ ] Sensor encoder (numerical → latent, Phase 1 complete from MVP)
- [ ] Text encoder (semantic transformer, lightweight)
- [ ] Audio encoder (wav2vec-style, basic)
- [ ] Signal binding engine (co-occurring modality fusion)
- [ ] Credit assignment via Modality Map during investigation
- [ ] Phase 1 → Phase 2 Action embedding crossover

**Exit criteria:** ELM can bind a sound and a temperature spike into a single experience. Post-surprise investigation correctly identifies which modality caused the Delta.

---

## Phase 4 — Tier 2 Abstraction

**Goal:** ELM develops abstract principles from patterns of failure.

- [ ] Prediction Error Log (track Tier 1 failure patterns)
- [ ] Tier 2 abstraction trigger (failure pattern detection)
- [ ] Tier 2 Pocket structure (abstract principles, binary hash)
- [ ] Tier 2 retrieval in Reasoning Layer
- [ ] Meta-planning using Tier 2 principles

**Exit criteria:** ELM generates at least one verifiable Tier 2 principle from repeated Tier 1 failures. Principle is demonstrably useful in planning novel situations.

---

## Phase 5 — Self-Model Maturity

**Goal:** ELM fully understands its own body and plans around it.

- [ ] Full Self_Model with degradation tracking
- [ ] Trend analysis per sensor (bearing wear, thermal drift, etc.)
- [ ] Performance envelope enforcement in planning
- [ ] Predictive maintenance alerts
- [ ] Self-repair goal generation (when degradation detected)
- [ ] Hardware port: x86_64 with full MSR access

**Exit criteria:** ELM detects simulated hardware degradation (increasing motor current) before failure. Self_Model accurately reflects the trend. Planning adapts to reduced capability.

---

## Phase 6 — Community and Ecosystem

**Goal:** ELM runs on diverse hardware, in diverse environments.

- [ ] Hardware Abstraction Layer (HAL) for new platforms
- [ ] ARM Cortex-M port (microcontroller ELM)
- [ ] x86_64 bare-metal port
- [ ] RISC-V port
- [ ] Python simulation harness (for researchers without embedded hardware)
- [ ] REST API for external interaction (optional, not core)
- [ ] Visualization dashboard for memory state
- [ ] Formal specification document (grounded in Phase 0-5 findings)
- [ ] Academic paper (when results warrant)

---

## Long Horizon — What ELM Could Become

These are not commitments. They are directions worth exploring once the foundation is solid.

**Distributed ELM:** Multiple ELM instances sharing compressed Tier 1/2 knowledge while maintaining separate Tier 0 experience streams. Shared rules, individual memories.

**Embodied robotics:** ELM as the cognitive core of a physical robot. Full sensorimotor loop. Real-world navigation, manipulation, failure recovery.

**Lifelong learning benchmark:** Using ELM's Delta curve as a standardized benchmark for measuring learning efficiency in experience-based systems — an alternative to static test-set accuracy.

**Neuromorphic hardware:** Running ELM on neuromorphic chips (Intel Loihi, BrainScaleS) where the spike-based computation maps naturally to the Delta-driven update model.

---

## What We Will Not Do

To keep the project honest:

- We will not add an LLM as the reasoning layer. The reasoning layer must emerge from the memory architecture.
- We will not use pre-trained weights as a starting point. ELM learns from zero.
- We will not add features before the MVP is validated.
- We will not publish claims before we have empirical results.
