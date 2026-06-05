# ELM Minimum Viable Prototype

> The Thermal Maze — proving the core memory loop works

---

## Why an MVP First

Paper is cheap. Compute is ruthless.

If we write a complete formal specification before building anything, we will over-engineer edge cases that don't exist while missing the physical bottlenecks that will crash the system on day one.

The MVP is the **physics engine** for the specification. We build a tiny, bounded version of ELM to prove the core loop works. Only then do we extend, grounded in empirical truth rather than theory.

The MVP must answer one question: **Can an untrained agent, starting from zero experience, build a memory of a world it has never seen — purely from surprise?**

If yes, ELM is real.

---

## MVP Scope

### What the MVP Includes

- Tier 0 Pocket creation and storage
- Tier 1 compression (from Tier 0 clusters)
- Delta calculation (predicted vs actual)
- Similarity engine (context-weighted)
- World Model (Tier 1 retrieval for prediction)
- Single-step reactive planning
- State_Self (basic hardware sensing)
- Sensor normalization
- UART debug output

### What the MVP Excludes

- Tier 2 abstraction
- Bifurcation / Contradiction engine
- Multi-step hierarchical planning
- Multi-modal encoding (UEE)
- Language or vision inputs
- NVMe persistence (in-memory only for MVP)
- Bare-metal boot (MVP runs in hosted Rust first, bare-metal second)

---

## The Environment — Thermal Maze

A 10×10 grid world. The agent has no map. It cannot see walls before hitting them.

### Grid Layout

```
Col:  0 1 2 3 4 5 6 7 8 9
Row 0: . . . . . . . . . .
Row 1: . . . . W W . . . .
Row 2: . . . . W . . . . .
Row 3: . . . . W . W W W .
Row 4: . S . . . . W . . .
Row 5: . . . . . . W . G .
Row 6: . . . W W . W . . .
Row 7: . . . W . . . . . .
Row 8: . . . W . . . H . .
Row 9: . . . . . . . . . .

S = Start position (1, 4)
G = Goal          (8, 5)
H = Heat source   (7, 8)
W = Wall (impassable)
. = Open space
```

### Sensor Inputs

The agent has three sensors only — no vision, no language:

```rust
struct SensorReading {
    position:    (u8, u8),   // (x, y) — current grid position
    temperature: f32,        // 0.0 - 100.0
    touch:       bool,       // true if last move hit a wall
}
```

### Temperature Field

Temperature radiates from H at (7, 8) using inverse distance:

```
temperature(x, y) = 100.0 / (1.0 + euclidean_distance((x,y), (7,8)))
```

Every cell has a different temperature. Every move produces a continuous signal change. The agent cannot memorize discrete states — it must build a predictive model.

### Actions

Four actions only:

```rust
enum Action {
    North,  // y - 1
    South,  // y + 1
    East,   // x + 1
    West,   // x - 1
}
```

---

## Experience Payload for MVP

The MVP uses a simplified payload — no UEE, no multi-modal encoding:

```rust
struct MvpPayload {
    state_focal:  [f32; 4],  // [x_norm, y_norm, temp_norm, touch_f32]
    state_self:   MvpStateSelf,
    action:       Action,
    outcome:      [f32; 4],  // [x_new, y_new, temp_new, touch_new]
    delta:        f32,
}

struct MvpStateSelf {
    cpu_temp:         f32,
    memory_used_mb:   f32,
    inference_micros: u64,  // time taken for last prediction
}
```

All values normalized to [0, 1] before storage.

---

## The Four Critical Test Moments

The MVP passes if and only if all four of these are observed:

### PASS_1 — First Wall Collision Triggers High Delta

```
Agent at (3, 4) attempts East
Expected: position = (4, 4), touch = false
Actual:   position = (3, 4), touch = true

Expected Delta: HIGH (> threshold_high)
Expected result: Tier 0 Pocket created at Level 0 (float32)
                 Pocket significance spikes
```

This proves surprise detection works.

### PASS_2 — Wall Pockets Cluster But Do NOT Compress

```
Agent hits cells (4,1), (4,2), (4,3), (4,4) successively
All produce high Delta (walls are consistently surprising)

Expected: Cluster forms (Cluster_ID assigned)
Expected: cluster_avg_delta remains HIGH
Expected: Compression does NOT trigger
          (high avg_delta prevents it)

Wall cluster acts as a persistent danger map.
```

This proves the compression gate works correctly.

### PASS_3 — Open Space Pockets DO Compress to Tier 1

```
Agent moves freely in open space repeatedly:
  (1,4)→(1,3), (1,3)→(1,2), (2,4)→(2,3), etc.

All produce low Delta (movement is predictable)
Cluster accumulates. cluster_avg_delta stays LOW.
cluster_size hits threshold_N.

Expected: Compression triggers
Expected: Tier 1 Rule created:
  "Moving in cardinal direction through open space
   advances position by 1 in that direction"
  Confidence: growing
  Condition_Map: [IF touch=true THEN rule fails]
  Outcome_Distribution: {success: ~0.94, blocked: ~0.06}
```

This proves the full compression cycle works.

### PASS_4 — Delta Drops on Familiar Paths

```
After Tier 1 rule exists, agent revisits familiar open cells.

Expected: Prediction Engine retrieves Tier 1 rule
Expected: Predicted_Outcome matches Actual_Outcome closely
Expected: Delta measurably lower than on first visit
Expected: Agent behavior changes — less cautious on known paths

Plot: Average Delta over time on open-space movement
  Should show:
    High and noisy (experiences 0-100)
    Declining (experiences 100-300)
    Plateauing near zero for familiar territory
    Spiking when entering new areas
```

**This is the learning curve. This is proof that ELM works.**

---

## Failure Mode Diagnostics

| Failure | Symptom | Diagnosis |
|---|---|---|
| FAIL_1 | Everything compresses, walls included | Compression threshold D too high |
| FAIL_2 | Nothing compresses, open space stays Tier 0 | Threshold N too large, or similarity metric broken |
| FAIL_3 | Delta never drops on familiar paths | World Model retrieval broken, Tier 1 not being used for prediction |
| FAIL_4 | Agent loops forever, never reaches G | Single-step planning insufficient, or temperature gradient too weak |

Each failure points to exactly which component needs fixing.

---

## MVP Success Metric — The Learning Curve

After the prototype runs 500 experiences, generate this plot:

```
Y axis: Average Delta (open-space movement only)
X axis: Experience number

Expected shape:
  ┌──────────────────────────────────────────┐
  │ 1.0 ╭─────╮                             │
  │     │     │                             │
  │ 0.5 │     ╰──────╮                      │
  │     │             ╰─────╮               │
  │ 0.0 │                    ╰──────────────│
  │     └──────────────────────────────────  │
  │     0      100     200     300     500  │
  └──────────────────────────────────────────┘

High and noisy early (everything is new)
Declining as Tier 1 rules form
Plateauing near zero for known territory
```

This is not a loss function from backpropagation. This is not accuracy on a held-out test set. This is a direct measure of how surprised the agent is by a world it is actively learning to predict. The curve going down means ELM is working.

---

## Goal: Increase Temperature

The agent's hardcoded MVP goal:

```rust
let goal = Goal {
    target_state: encode([_, _, 0.9, _]),  // temperature normalized near 1.0
    success_threshold: 0.05,
    origin: GoalOrigin::External,
};
```

Single-step planning: at each position, query World Model for adjacent cell with highest predicted temperature. Move there.

The goal is *near* the heat source but not identical to it. The agent must learn the temperature gradient from experience — it cannot know the layout in advance.

---

## Implementation Plan

### Phase 1 — Grid World Environment

```rust
// Implement first, before any ELM code
struct GridWorld {
    walls: [[bool; 10]; 10],
    heat_source: (u8, u8),
    agent_pos: (u8, u8),
}

impl GridWorld {
    fn step(&mut self, action: Action) -> SensorReading
    fn temperature_at(&self, x: u8, y: u8) -> f32
    fn is_wall(&self, x: u8, y: u8) -> bool
}
```

Test the environment in isolation first. Print the grid. Verify temperature field. Verify wall collisions.

### Phase 2 — Pocket Data Structure

Implement and unit test the Pocket struct. Verify:
- Serialization / deserialization
- Quantization level assignment from Delta
- Significance decay function

### Phase 3 — Similarity Engine

Implement and unit test:
- Cosine similarity function
- Context-weighted similarity
- Clustering threshold assignments

Test with synthetic Pocket pairs. Verify weights sum to 1.0. Verify clustering assignments are correct.

### Phase 4 — Memory Bank + Compression Engine

Implement:
- Pocket storage and retrieval
- Cluster tracking
- Compression trigger
- Tier 1 Rule generation (centroid + variance, not average)

Test by manually inserting known similar Pockets and verifying compression fires correctly.

### Phase 5 — Prediction Engine

Implement:
- World Model query (retrieval context weights)
- Predicted_Outcome assembly
- Delta calculation
- World Model update rules

Test by running 50 experiences, then verifying that prediction confidence improves on repeated situations.

### Phase 6 — Full Loop Integration

Connect all components. Run the Thermal Maze scenario. Collect Delta-over-time data. Plot the learning curve.

### Phase 7 — Hardware Sensing

Add State_Self sampling:
- CPU temperature (platform-appropriate)
- Memory usage
- Inference latency (measure time of prediction step)

Verify State_Self appears in every Pocket. Run artificial CPU stress during a run and observe whether the ELM notices degraded inference latency.

### Phase 8 — Bare Metal Port

Port the validated Rust code to `no_std`. Add BCM2711 drivers. Flash to Raspberry Pi 4. Connect physical sensors. Run Thermal Maze with real sensor inputs substituting the simulated readings.

---

## Running the MVP

```bash
# Clone
git clone https://github.com/YOUR_ORG/elm
cd elm

# Run simulation (hosted Rust, no hardware needed)
cargo run --example thermal_maze

# Run with debug output
RUST_LOG=debug cargo run --example thermal_maze

# Generate learning curve plot
cargo run --example thermal_maze -- --plot
```

Output format:
```
[0000] Boot. No memory.
[0001] Pos:(1,4) Temp:18.2 → East → Pos:(2,4) Temp:19.1 | Delta:0.03 | Q:1
[0002] Pos:(2,4) Temp:19.1 → East → Pos:(3,4) Temp:20.8 | Delta:0.04 | Q:1
[0003] Pos:(3,4) Temp:20.8 → East → BLOCKED touch=true   | Delta:0.91 | Q:0 !!
[0003] SURPRISE — wall detected. Pocket stored at full precision.
[0004] Cluster check: wall_cluster size=1 avg_delta=0.91 (no compress)
...
[0089] Tier1 Rule created: open_space_movement (source_count=47, confidence=0.71)
[0090] Pos:(1,4) → East — using Tier1 rule — Delta:0.02 (was 0.03 at experience 1)
```

---

## What Comes After MVP

If PASS_4 fires — the learning curve bends down — we have empirical proof.

Next steps in order:

1. Write the formal specification grounded in MVP findings
2. Add Bifurcation / Contradiction engine
3. Add multi-step planning
4. Add Universal Experience Encoder (text + audio + video)
5. Bare-metal boot on Raspberry Pi 4
6. Physical sensor integration
7. Full Self_Model and degradation tracking
8. Tier 2 abstraction

The MVP is not the destination. It is the proof that the destination is real.
