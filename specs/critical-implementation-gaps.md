# Plan: Critical Implementation Gaps

## Task Description
This task addresses the systemic gap between the high-level architectural specifications in `docs/` and the current minimal implementation in `src/`. The goal is to transform the ELM prototype from a "simulated reactive loop" into a "bare-metal cognitive engine" by implementing the missing layers of the architecture: Hardware HAL, NVMe Persistence, advanced Compression, UEE Perception, and Goal-Driven Planning.

## Objective
Align the codebase with the v0.4 Architecture Specification, specifically implementing the "body-as-self" principle and the multi-tier memory compression pipeline on real (or high-fidelity simulated) hardware.

## Problem Statement
The current implementation is a "skeleton" that satisfies the basic "Thermal Maze" simulation but lacks the fundamental characteristics of ELM:
1. **Hardware Blindness**: `StateSelf` is mocked; the system cannot sense its own physical state.
2. **Amnesia**: No persistence mechanism; all learning is lost on reboot.
3. **Naive Compression**: Compression is a binary flag toggle rather than a mathematical abstraction (centroid-based).
4. **Reactive, Not Proactive**: The agent reacts to the environment but cannot plan toward a goal.
5. **Rigid Perception**: Data is handled as raw structs instead of normalized, encoded experience units (UEE).

## Solution Approach
We will implement the architecture in "bottom-up" order, following the dependency chain defined in `docs/ARCHITECTURE.md` (Layer 0 $\rightarrow$ Layer 5).

1. **Layer 0/1 (Hardware & Perception)**: Implement a Hardware Abstraction Layer (HAL) for Raspberry Pi 4. This enables real register reads for temperature, CPU cycles, and I2C sensor communication.
2. **Layer 2/3 (Memory & Compression)**: Upgrade the `MemoryBank` from simple clustering to centroid-based compression and implement raw LBA writes to NVMe for persistence.
3. **Layer 4/5 (World Model & Reasoning)**: Replace the reactive `step` function with a proper Planning Engine that queries the World Model to maximize temperature (the MVP goal).

## Relevant Files

### Existing Files
- `src/lib.rs`: Update to include new modules (`hal`, `planning`, `perception`).
- `src/sensors/state_self.rs`: Expand from mocks to real hardware readings via the HAL.
- `src/memory/bank.rs`: Implement centroid-based compression and persistence hooks.
- `src/memory/pocket.rs`: Update to support Tier 2 abstraction and UEE encoding.
- `src/agent.rs`: Refactor the `step` loop to use the new planning engine.
- `Cargo.toml`: Add dependencies for serialization (`postcard`) and fixed-point math (`micromath`).

### New Files
- `src/hal/mod.rs`: HAL trait definitions (I2C, UART, NVMe, PMU).
- `src/hal/rpi4.rs`: Concrete implementation for BCM2711.
- `src/perception/uee.rs`: Universal Experience Encoder for signal normalization.
- `src/planning/mod.rs`: Planning interface and goal definitions.
- `src/planning/engine.rs`: Goal-driven state transition logic.

## Implementation Phases

### Phase 1: Hardware Foundation (Layer 0)
Implement the HAL to move beyond mocks. This ensures that `StateSelf` reflects actual physical reality.
- Define `Hal` trait.
- Implement RPi4 specific register reads for CPU Temp and PMU.
- Implement minimal I2C driver for BME280/MPU6050.

### Phase 2: Persistence & Advanced Memory (Layer 2-3)
Move from volatile RAM to persistent storage and from simple clusters to semantic rules.
- Implement `NVMe` block driver in HAL.
- Implement `serialize/deserialize` for Pockets using `postcard`.
- Replace simple `compression_tier` flag with `Centroid` calculations in `MemoryBank`.
- Implement Tier 2 (Abstract Principles) by compressing clusters of Tier 1 rules.

### Phase 3: Cognitive Loop (Layer 4-5)
Implement the "Mind" that uses the memory.
- Implement the UEE to normalize all sensor inputs to `[0, 1]`.
- Build the `PlanningEngine` that iterates through possible actions and selects the one with the highest predicted `StateFocal` temperature.
- Integrate the full loop: `Sense` $\rightarrow$ `Plan` $\rightarrow$ `Act` $\rightarrow$ `Predict` $\rightarrow$ `Learn`.

## Step by Step Tasks

### 1. Establish Hardware Abstraction Layer (HAL)
- Create `src/hal/mod.rs` with traits for `Pmu`, `I2c`, and `Nvme`.
- Create `src/hal/rpi4.rs` implementing these traits using `volatile` register access.
- Update `src/sensors/state_self.rs` to use the `Hal` trait instead of hardcoded mocks.

### 2. Implement UEE Perception Layer
- Create `src/perception/uee.rs`.
- Implement normalization functions for temperature, distance, and coordinates.
- Update `ExperiencePayload` to store normalized `f32` values instead of raw types.

### 3. Build Raw NVMe Persistence
- Implement `nvme_write_block` and `nvme_read_block` in `src/hal/rpi4.rs`.
- Add serialization logic to `src/memory/bank.rs` to save/load the `BTreeMap` of pockets to specific LBA ranges.
- Implement a `graceful_shutdown` trigger that flushes the memory bank to disk.

### 4. Upgrade Compression to Centroid-Based Abstraction
- Refactor `MemoryBank::compress_to_tier_1` to calculate the centroid (mean) and variance of a cluster.
- Implement the `Tier 2` compression logic: when a group of Tier 1 rules exhibit similar variance, compress them into a "Principle".
- Update `src/prediction/engine.rs` to query Tier 2 principles first, then Tier 1 rules.

### 5. Implement Goal-Driven Planning Engine
- Create `src/planning/engine.rs`.
- Implement a "one-step lookahead" planner that queries the `WorldModel` for all 4 cardinal actions.
- Implement a `Goal` struct that defines the target state (e.g., `temp > 0.9`).
- Update `ELMAgent::step` to call the planner instead of taking a random/hardcoded action.

### 6. End-to-End Integration & Hardware Validation
- Update `src/bin/thermal_maze.rs` to use the real HAL in a `cfg(target_arch = "aarch64")` block.
- Run the agent on the physical Raspberry Pi 4.
- Verify "PASS_4" (Learning Curve) using real sensor data.

## Testing Strategy
- **HAL Mocking**: Use `cfg(test)` to provide a `MockHal` that simulates register changes, allowing CI to pass while testing hardware logic.
- **Persistence Unit Tests**: Verify that a `MemoryBank` saved to a virtual block device can be restored with 100% fidelity.
- **Compression Validation**: Insert 10 synthetic pockets with high similarity and verify a Tier 1 rule is generated with the correct centroid.
- **Planning Regression**: Run a "Known World" simulation and verify the agent reaches the goal in the minimum number of steps.

## Acceptance Criteria
- [ ] `StateSelf` reads from real BCM2711 registers on aarch64.
- [ ] Memory Bank survives a reboot (NVMe Persistence).
- [ ] Tier 1 and Tier 2 pockets are created via centroid calculation.
- [ ] Agent successfully navigates to the heat source using the Planning Engine.
- [ ] All CI builds pass for both `x86_64` (simulation) and `aarch64` (bare-metal).

## Validation Commands
- `cargo build --target aarch64-unknown-none` (Verify bare-metal compilation)
- `cargo test memory::compression` (Verify centroid logic)
- `cargo run --bin thermal_maze` (Verify planning loop)
