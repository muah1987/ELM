# Plan: Native PU Indexing and Hardware Sensing

## Task Description
Transform the current static Hardware Abstraction Layer (HAL) into a dynamic, native Processing Unit (PU) Indexing system. The goal is to allow the ELM kernel to discover, index, and interact with all available hardware components (PUs) on the target architecture (aarch64) natively, enabling full hardware sensing and utilization.

## Objective
Implement a `PuRegistry` that dynamically indexes all hardware units and provides a native interface for the agent to sense and control any indexed PU.

## Problem Statement
The current HAL is a static trait implementation for the RPi4. It cannot handle multiple similar units, does not support dynamic hardware discovery, and requires the agent to have prior knowledge of the hardware layout. This limits the "body-as-self" principle, as the agent cannot "sense" what hardware it actually possesses at runtime.

## Solution Approach
We will implement a "Resource-Based Hardware Model":
1. **Define the `Pu` Trait**: Create a universal interface for any hardware unit (Processing Unit). This includes metadata (ID, type, version) and raw register access (`read_reg`, `write_reg`).
2. **Implement the `PuRegistry`**: A central index that stores all discovered PUs.
3. **Native Probing**: Implement a boot-time discovery sequence that probes MMIO regions to identify hardware and populate the registry.
4. **Integration**: Refactor `StateSelf` and `ELMAgent` to utilize the registry for proprioception and action execution.

## Relevant Files

### Existing Files
- `src/lib.rs`: Update module declarations.
- `src/hal/mod.rs`: Refactor from static traits to the PU Registry model.
- `src/hal/rpi4.rs`: Convert the RPi4 implementation into a set of specific `Pu` implementations (e.g., `Rpi4PmuPu`, `Rpi4NvmePu`).
- `src/sensors/state_self.rs`: Update to query the `PuRegistry` for metrics.
- `src/agent.rs`: Integrate native PU interaction.

### New Files
- `src/hal/pu.rs`: Defines the `Pu` trait, `PuId`, and the `PuRegistry`.
- `src/hal/discovery.rs`: Implements the hardware probing logic to index PUs.

## Implementation Phases

### Phase 1: Foundation (The PU Model)
Define the core abstractions for a "Processing Unit". This replaces the high-level HAL traits with a more granular, register-level approach.

### Phase 2: Native Implementation & Indexing
Refactor the RPi4 hardware logic into concrete `Pu` implementations and build the `PuRegistry` and discovery mechanism.

### Phase 3: Cognitive Integration
Connect the `PuRegistry` to the agent's proprioception and planning loops, allowing the agent to sense and use all indexed hardware.

## Team Orchestration

- I operate as the team lead and orchestrate the team to execute the plan.
- I use `Task` and `Task*` tools to deploy team members.

### Team Members
- **HAL-Architect**: 
  - Role: Expert in bare-metal Rust and MMIO. Responsible for `Pu` traits and `PuRegistry`.
  - Agent Type: `builder`
- **Hardware-Probe-Engineer**: 
  - Role: Specializes in hardware discovery and BCM2711 register maps.
  - Agent Type: `builder`
- **Cognitive-Integrator**: 
  - Role: Expert in the ELM cognitive loop. Integrates the registry into `StateSelf` and `ELMAgent`.
  - Agent Type: `builder`

## Step by Step Tasks

### 1. Define PU Trait and Registry
- **Task ID**: `def-pu-trait`
- **Depends On**: `none`
- **Assigned To**: `HAL-Architect`
- **Agent Type**: `builder`
- **Parallel**: `false`
- Create `src/hal/pu.rs`.
- Define `PuId` (unique identifier) and `PuInfo` (metadata).
- Define the `Pu` trait with methods for `identify()`, `read_reg()`, and `write_reg()`.
- Implement `PuRegistry` using `lazy_static` and a `BTreeMap<PuId, Box<dyn Pu>>`.

### 2. Refactor RPi4 into Concrete PUs
- **Task ID**: `refactor-rpi4-pu`
- **Depends On**: `def-pu-trait`
- **Assigned To**: `HAL-Architect`
- **Agent Type**: `builder`
- **Parallel**: `false`
- Refactor `src/hal/rpi4.rs`.
- Implement `Rpi4PmuPu`, `Rpi4NvmePu`, and `Rpi4I2cPu` by implementing the `Pu` trait.
- Remove old HAL traits in favor of the `Pu` model.

### 3. Implement Hardware Discovery Logic
- **Task ID**: `impl-hw-discovery`
- **Depends On**: `refactor-rpi4-pu`
- **Assigned To**: `Hardware-Probe-Engineer`
- **Agent Type**: `builder`
- **Parallel**: `false`
- Create `src/hal/discovery.rs`.
- Implement `probe_hardware()` which scans MMIO regions and registers identified PUs into the `PuRegistry`.
- Update `src/hal/mod.rs` to export the discovery mechanism.

### 4. Update Proprioception (StateSelf)
- **Task ID**: `update-state-self`
- **Depends On**: `impl-hw-discovery`
- **Assigned To**: `Cognitive-Integrator`
- **Agent Type**: `builder`
- **Parallel**: `false`
- Update `src/sensors/state_self.rs`.
- Modify `StateSelf` to query the `PuRegistry` for the `Rpi4PmuPu` to get real core temperature and CPU cycles.
- Implement a way to iterate over all indexed PUs to gather global hardware health.

### 5. Integrate Native Hardware Use in Agent
- **Task ID**: `agent-native-hw`
- **Depends On**: `update-state-self`
- **Assigned To**: `Cognitive-Integrator`
- **Agent Type**: `builder`
- **Parallel**: `false`
- Update `src/agent.rs`.
- Implement a "Native Command" interface where the agent can dispatch raw register writes to any indexed `PuId` as a special action.
- Update the planning engine to recognize the existence of multiple PUs.

### 6. Validation & Bare-Metal Build
- **Task ID**: `val-native-pu`
- **Depends On**: `agent-native-hw`
- **Assigned To**: `HAL-Architect`
- **Agent Type**: `builder`
- **Parallel**: `false`
- Run `cargo build --target aarch64-unknown-none`.
- Verify that the `PuRegistry` is correctly populated during a simulated boot.

## Testing Strategy
- **Unit Tests (Host)**: Use `cfg(test)` to create `MockPu` implementations and verify the `PuRegistry` indexing and retrieval logic on the host machine.
- **Integration Tests**: Verify that `StateSelf` correctly retrieves data from a registered `Pu`.
- **Hardware Verification**: On physical RPi4, log the list of indexed PUs at boot to verify that the probe discovered all components.

## Acceptance Criteria
- [ ] `PuRegistry` is implemented and accessible globally via `lazy_static`.
- [ la ] Hardware discovery `probe_hardware()` correctly indexes all RPi4 PUs at boot.
- [ ] `StateSelf` metrics are derived from the `PuRegistry` instead of static HAL calls.
- [ ] `ELMAgent` can interact with any PU in the registry via its `PuId`.
- [ ] The project compiles for `aarch64-unknown-none` without errors.

## Validation Commands
- `cargo build --target aarch64-unknown-none`
- `cargo test --lib --target x86_64-unknown-linux-gnu` (Verify Registry logic)

## Notes
- Ensure `PuRegistry` uses a thread-safe lock (e.g., `spin::Mutex`) as it's used in a `#![no_std]` environment.
- Use `volatile` for all register access within `Pu` implementations.
