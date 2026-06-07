# ELM Architecture

> Full cognitive architecture specification — v0.4

---

## Overview

ELM is built on one foundational insight: **a system that learns from experience needs memory as its primary organ, not a reasoning engine.**

Biological systems evolved memory long before complex reasoning. ELM follows the same sequence. The reasoning layer is built last, on top of a solid memory foundation.

The architecture has five layers, built in dependency order:

```
Layer 5: Reasoning        — goals, planning, execution monitoring, hedonic bias
Layer 4: World Model      — prediction engine, contradiction resolution
Layer 3: Compression      — Tier 0 → Tier 1 → Tier 2 abstraction
Layer 2: Memory Bank      — Pocket storage, clustering, similarity, hedonic gradient
Layer 1: Perception       — UEE, normalization, signal binding
Layer 0: Hardware Self    — bare-metal proprioception, native PU indexing
```

Each layer depends entirely on the one below it. No shortcuts.

---

## Layer 0 — Hardware Self

ELM runs bare-metal. No operating system. The hardware is not infrastructure — it is the model's body.

### State_Self

Every experience Pocket contains a State_Self snapshot — the state of ELM's own body at the moment the experience occurred.

```
State_Self {
    // ARM Performance Monitor Unit (direct register reads)
    cpu_cycles:        u64,   // PMCCNTR_EL0
    instructions:      u64,   // PMEVCNTR0_EL0
    cache_misses:      u64,   // PMEVCNTR1_EL0

    // Thermal (BCM2711 register 0xFE212058)
    cpu_temp_raw:      u32,

    // ARM Generic Timer
    timestamp:         u64,   // CNTPCT_EL0

    // Physical sensors (I2C)
    ambient_temp:      f32,   // BME280
    pressure:          f32,   // BME280
    humidity:          f32,   // BME280
    accel_x:           f32,   // MPU-6050
    accel_y:           f32,   // MPU-6050
    accel_z:           f32,   // MPU-6050
    gyro_x:            f32,   // MPU-6050
    gyro_y:            f32,   // MPU-6050
    gyro_z:            f32,   // MPU-6050
    distance_mm:       f32,   // VL53L0X

    // Memory (MMU)
    pages_allocated:   u32,
    page_faults:       u32,   // pain signal — high significance trigger
}
```

### Pain Signals

Certain hardware events immediately trigger maximum-significance Pockets regardless of context:

| Event | Significance | Reason |
|---|---|---|
| page_faults spike | MAX | Memory boundary violation |
| cpu_temp > 85°C | MAX | Thermal danger |
| motor stall | MAX | Physical blockage |
| battery critical | MAX | Survival threat |
| memory > 95% | HIGH | Resource exhaustion |
| cache_miss spike | HIGH | Performance degradation |

These are the ELM's equivalent of pain. They interrupt normal processing and anchor permanently into the Self_Model.

### Boot Sequence

```
Power On
  ↓
UEFI/Firmware initializes silicon
  ↓
ELM Bootloader takes control
  ↓
Stage 0: Hardware Baseline
  Probe I2C bus → enumerate sensors
  Read ARM PMU → compute baseline
  Read thermal register → temperature baseline
  Sample all sensors N=100 cycles
  Build initial Self_Model
  ("This is what healthy and resting feels like")
  ↓
Stage 1: Memory Bank initialized in RAM
  (blank — no Pockets yet)
  ↓
Stage 2: First IRQ fires
  (world experience begins)
  ↓
Main Loop runs
  ↓
Shutdown IRQ
  Serialize Tier 1+ Pockets to NVMe
  Save Self_Model baseline
  Power off
  ↓
Next boot: memory restored
```

---

## Layer 1 — Perception

### The Universal Experience Encoder (UEE)

All input modalities — text, audio, video, sensor data — are encoded into a **shared latent space** before touching the memory system. This makes cross-modal similarity valid and meaningful.

```
Modality    Encoder Type              Output Dims   Feeds
─────────────────────────────────────────────────────────────
Text        Semantic transformer      dims 512-767   Focal + Action
Audio       Wav2vec-style conv net    dims 256-511   Focal + Ambient
Video       Spatial/temporal CNN      dims 0-255     Focal + Ambient
Sensors     MLP per sensor type       dims 768-1023  Ambient primarily
Hardware    Direct register read      State_Self     Always present
```

All encoders project to the same 1024-dimensional space. The mapping of which dimensions belong to which modality is called the **Modality Map** and is stored in every Pocket.

### The Modality Map

```
Modality_Map {
    dims_0_255:    video_spatial,
    dims_256_511:  audio_semantic,
    dims_512_767:  text_semantic,
    dims_768_1023: sensor_array,
}
```

During post-surprise investigation (high Delta), the system isolates which dimension block showed the most variance. This identifies which modality caused the surprise. This solves the **credit assignment problem** in multi-modal systems.

### Signal Binding

Co-occurring signals from multiple modalities must be bound into a single experience, not stored as separate Pockets:

```
Binding Rule:
  IF signals arrive within time window T
  AND their encoded vectors show similarity >= 0.6
  THEN bind into single Pocket with merged payload
  ELSE store as separate Pockets with temporal Edge
```

### Sensor Normalization

All sensor streams are normalized **before** encoding to prevent high-range sensors dominating similarity calculations:

```
Normalized = (raw_value - sensor_min) / (sensor_max - sensor_min)
           → output range [0, 1]

OR Z-score: Normalized = (raw_value - mean) / std_dev
           → output range approximately [-3, 3]
```

Every sensor has its own normalization parameters, initialized during Stage 0 hardware baseline.

---

## Layer 2 — Memory Bank

### The Pocket

The fundamental unit of memory. Every experience becomes a Pocket at the moment it occurs.

```
Pocket {
    id:                   Hash,          // unique identifier
    timestamp:            u64,           // ARM cycle counter
    cluster_id:           Option<Hash>,  // null if novel
    quantization_level:   u8,            // 0-3

    payload: ExperiencePayload {
        state_focal:      Vector<f32>,   // high-res attention data
        state_ambient:    Vector<f16>,   // background context
        state_self:       StateSelf,     // hardware body state
        action:           Action,        // discrete tag OR vector
        outcome:          Vector<f32>,   // what resulted
        delta:            f32,           // surprise magnitude
        modality_map:     ModalityMap,   // which dims = which sense
    },

    significance:         f32,           // f(delta) + decay_rate
    compression_tier:     u8,            // 0, 1, or 2
    edges:                Vec<Hash>,     // similar Pocket pointers
    exceptions:           Vec<Hash>,     // Tier0 violations of this rule
}
```

### Quantization Levels

Compression level is driven by Delta — how surprising the experience was:

| Level | Format | Dims | When | Storage Cost |
|---|---|---|---|---|
| 0 | float32 | 1024 | High Delta, novel | High |
| 1 | float16 | 1024 | Moderate Delta, soft cluster | Medium |
| 2 | int8 (PQ) | 256 | Low Delta, strong cluster | Low |
| 3 | Binary hash | 128-bit | Tier 2 abstraction | Negligible |

```
Quantization Decision:

IF   delta > threshold_high                          → Level 0
ELIF delta > threshold_mid AND cluster_size < N      → Level 1
ELIF delta < threshold_mid AND cluster_size >= N     → Level 2
ELIF tier2_abstraction_triggered                     → Level 3
```

### Similarity Engine

Two Pockets are compared using **context-sensitive weighted similarity**:

```
S(P1, P2 | Context) =
    w_fs(C) * sim(state_focal_1,   state_focal_2)
  + w_as(C) * sim(state_ambient_1, state_ambient_2)
  + w_a(C)  * sim(action_1,        action_2)
  + w_o(C)  * sim(outcome_1,       outcome_2)

where all weights sum to 1.0
```

Weights shift based on what you are trying to do:

| Context | w_focal | w_ambient | w_action | w_outcome |
|---|---|---|---|---|
| Compression | 0.40 | 0.05 | 0.40 | 0.15 |
| Retrieval | 0.50 | 0.20 | 0.20 | 0.10 |
| Investigation | 0.20 | 0.60 | 0.05 | 0.15 |

### Clustering Thresholds

```
S >= 0.90  → Strong merge candidate (assign shared Cluster_ID)
S  0.70-0.89 → Soft cluster (create Edge, no Cluster_ID)
S < 0.70   → Novel experience (standalone Pocket)
```

### Action Bootstrapping (Two-Phase)

Actions cannot be embedded vectors at cold start — there is no training data to learn from yet.

**Phase 1 — Taxonomic distance:**
Actions defined as a discrete hierarchy. Similarity = tree traversal depth.
```
Interact → Manipulate → Grasp  (same leaf: 1.0)
Interact → Manipulate          (same parent: 0.6)
Interact                       (same root: 0.3)
```

**Phase 2 — Learned embedding:**
Once action embedding loss drops below threshold (or N experiences accumulated), switch sim(action) to cosine similarity in learned latent space. Crossover is automatic and continuous.

---

## Layer 3 — Compression

### Compression Trigger

```
IF cluster_size     >= threshold_N
AND cluster_avg_delta <= threshold_D
THEN compress cluster → Tier 1
```

Note: High-Delta clusters (walls, pain signals) accumulate but never trigger compression. They form **persistent danger maps** — exactly correct behavior.

### Tier 1 Rule Structure

Tier 1 is NOT a mean experience. It is a **generalized rule with known operating boundaries**.

```
Tier1_Pocket {
    id:               Hash,
    created:          u64,
    source_cluster:   Hash,
    source_count:     u32,

    rule: GeneralizedRule {
        state_focal:  Centroid + VarianceBounds,
        state_ambient: dropped unless repeatedly significant,
        action:       Centroid + VarianceBounds,
        outcome:      ProbabilityDistribution,
        condition_map: Vec<BoundaryCondition>,
    },

    confidence:       f32,   // f(source_count, delta_variance)
    compression_tier: 1,
    edges:            Vec<Hash>,
    exceptions:       Vec<Hash>,  // Tier0 pockets that violated this rule
}
```

The **Condition Map** is critical — it records the boundaries where the rule stops being true:

```
condition_map: [
    BoundaryCondition {
        feature:         "force_newtons",
        threshold:       5.5,
        direction:       Above,
        outcome_shifts:  Failure,
        confidence:      0.91,
    },
    BoundaryCondition {
        feature:         "temperature_celsius",
        threshold:       10.0,
        direction:       Below,
        outcome_shifts:  Partial,
        confidence:      0.74,
    },
]
```

### Tier 2 Abstraction

Tier 2 emerges from **patterns in Tier 1 failures** — not from compressing more experiences.

```
Trigger:
  IF a Tier1 rule fails repeatedly in a consistent pattern
  THEN analyze: what structural property do the failures share?
  THEN create Tier2 principle

Example:
  Tier1 failures:
    "Grasp force limit varies with temperature"
    "Grasp force limit varies with surface texture"
    "Grasp force limit varies with material hardness"

  Tier2 emerges:
    "Physical force limits are context-dependent
     and vary with environmental state variables"
```

Tier 2 pockets are stored at Level 3 (binary hash). They are permanently retained. They are **never deleted**.

---

## Layer 4 — World Model and Prediction

### The Prediction Pipeline

Before every action, ELM generates a prediction:

```
1. Encode current state → Current_Vector
2. Encode intended action → Action_Vector
3. Query World Model (Tier1 + Tier2) with Retrieval weights
4. Retrieve top-K similar pockets
5. Run Divergence Check on retrieved outcomes
6. IF agreement → assemble Predicted_Outcome_Distribution
   IF contradiction → Bifurcation Engine
7. Action executes
8. Capture Actual_Outcome
9. Delta = distance(Predicted, Actual)
10. World Model update (if Delta warrants it)
```

### Prediction Output Structure

```
Prediction {
    predicted_outcome:   OutcomeDistribution,
    confidence:          f32,
    source_pockets:      Vec<Hash>,
    weakest_link:        ModalityType,     // where blind spot is
    fallback_used:       bool,
    contradiction_flag:  bool,
}
```

### Cold Start — Three Phase Prediction

```
Phase 0: No memory exists
  Predicted_Outcome = NULL
  Delta = MAX for everything
  Effect: Maximum fidelity retention, maximum curiosity

Phase 1: Sensor-only Tier1 rules exist
  Predictions from physical cause-effect only
  Delta measured against physical expectations

Phase 2: Full multi-modal World Model
  Cross-modal predictions
  Full Tier1 + Tier2 retrieval
```

### Contradiction Resolution — The Bifurcation Engine

When retrieved pockets have high similarity in State + Action but wildly divergent Outcomes:

```
1. Calculate D_conflict = variance of retrieved outcome vectors
2. IF D_conflict > threshold:
     Set Contradiction_Flag = TRUE
     Set Confidence = 0.0
     DO NOT average outcomes
     
3. Ambient Sweep:
     Pull State_Ambient from conflicting pockets
     Subtract across Modality_Map dimensions
     
     IF divergence found in ambient dims:
       Promote that feature to State_Focal
       Create new Condition_Map boundary
       
     IF ambient is identical:
       Hidden variable is completely unobserved
       Execute with 0.0 confidence
       Flag for Active Exploration
```

### World Model Update Rules

```
Delta < threshold_low:
  No update
  Source pocket Confidence += small increment

Delta >= threshold_low AND < threshold_high:
  Soft update
  Adjust Outcome_Distribution of nearest Tier1 pocket
  Adjust Condition_Map boundaries slightly

Delta >= threshold_high:
  Hard update
  New Tier0 pocket at full precision
  Nearest Tier1 Confidence -= penalty
  Investigation mode triggered
  State_Ambient preserved for causal analysis
```

---

## Layer 5 — Reasoning

### Goal Representation

Goals live in the same latent space as everything else:

```
Goal {
    id:               Hash,
    target_state:     Vector<f32>,      // what success looks like
    success_threshold: f32,
    constraints:      Vec<Vector<f32>>, // states to avoid
    priority:         f32,
    deadline:         Option<u64>,
    origin:           GoalOrigin,       // External | InternalDrive
                                        // | ContradictionResolution
                                        // | Exploration
}
```

### Planning Algorithm

Planning is path-finding through **experienced territory** — not search through abstract space:

```
1. Encode current state → Current_Vector
2. Encode goal → Goal_Vector
3. Query World Model for bridge pockets
   (State_Focal ~ Current AND Outcome ~ Goal)
4. IF direct bridge found → single-step plan
5. IF no bridge → find intermediate states
   Build chain: Current → Mid1 → Mid2 → ... → Goal
6. For each step: calculate score = (Confidence * 0.6) + (ExpectedValence * 0.4)
7. Output ranked Plan candidates by score
```

### Plan Structure

```
Plan {
    id:                   Hash,
    goal_id:              Hash,
    composite_confidence: f32,
    estimated_delta_risk: StepIndex,  // where failure is most likely
    unexplored_steps:     u32,
    steps: Vec<PlanStep {
        sequence:           u32,
        action:             Action,
        expected_state:     Prediction,
        source_pocket:      Hash,
        confidence:         f32,
        fallback:           Option<Action>,
        contradiction_flag: bool,
    }>,
}
```

### Execution Monitor

```
FOR each step executing:
  IF step_delta < threshold_low:   continue
  IF step_delta < threshold_high:  soft-adjust remaining steps
  IF step_delta >= threshold_high: HALT
                                   re-plan from current state
                                   IF re-plan fails → Exploration_Goal
```

### Exploration Drive

When no plan can be found:

```
DO NOT guess
DO NOT execute blindly

Generate Exploration_Goal {
    target:  reduce uncertainty in gap region
    method:  smallest safe action producing maximum information gain
    origin:  InternalDrive
}
```

Curiosity is not a personality trait in ELM. It is a logical necessity — a structural response to knowledge gaps.

---

## Reasoning Layer Interface

The Reasoning Layer never touches raw memory directly. Six clean interfaces only:

```
QUERY(context, state, action)       → relevant Tier1/2 pockets
PREDICT(state, action)              → Prediction struct
PLAN(current_state, goal)           → ranked Plan candidates
MONITOR(plan_step, actual_outcome)  → continue | adjust | halt | explore
ABSTRACT(failed_predictions)        → Tier2 abstraction trigger
EXPLORE(knowledge_gap)              → minimum-risk experiment
```

---

## The Self-Model

Over time, State_Self pockets compress into a dedicated structure — ELM's model of its own body:

```
Self_Model {
    baseline_profile: {
        healthy_cpu_temp:      Distribution,
        healthy_motor_current: Distribution,
        healthy_memory_usage:  Distribution,
        healthy_inference_latency: Distribution,
    },

    degradation_map: {
        // trends detected from Tier1 self-rules
        motor_2_bearing:   TrendAnalysis,
        storage_health:    TrendAnalysis,
    },

    performance_envelope: {
        max_reliable_compute:   Condition,
        optimal_movement_speed: Condition,
    },

    anomaly_history: Vec<Hash>,  // pointers to high-Delta self pockets
}
```

The Reasoning Layer consults Self_Model during planning:

```
IF plan requires heavy computation:
  CHECK Self_Model.performance_envelope
  IF cpu_temp approaching limit:
    MODIFY plan — reduce computational load
    OR schedule for cooler period
    OR flag degraded performance
```

---

## Design Principles Summary

1. **Memory before reasoning** — the foundation must be solid before reasoning is built on top
2. **Delta drives everything** — surprise is the universal signal for attention, retention, and update
3. **Compression by similarity** — familiar experiences cost almost nothing; novel experiences are preserved fully
4. **Hardware is self** — the body is part of the mind, not infrastructure beneath it
5. **No hallucination by architecture** — the reasoning layer can only plan through territory it has experienced
6. **Contradictions are signals** — conflicting memories mean a hidden variable exists; find it
7. **Curiosity is structural** — exploration is not a reward signal, it is the logical response to knowledge gaps
