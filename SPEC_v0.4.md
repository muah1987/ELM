# ELM Formal Specification v0.4

> Experience Learning Model — Complete Architecture Specification

**Status:** Draft — pre-MVP  
**Version:** 0.4  
**License:** MIT

---

## 1. The Pocket — Tier 0 Data Structure

The atomic unit of memory. Every experience becomes a Pocket at the moment it occurs.

```
Pocket {
  ID:                   unique hash (SHA-256)
  Timestamp:            ARM cycle counter (CNTPCT_EL0)
  Cluster_ID:           UUID of compression cluster (null if novel)
  Quantization_Level:   0 = float32 | 1 = float16 | 2 = int8 | 3 = binary

  Payload {
    State_Focal:        High-res vector under active attention
    State_Ambient:      Low-res background context vector
    State_Self:         Hardware body snapshot (see §6)
    Action:             Discrete tag (Phase 1) OR vector (Phase 2)
    Outcome:            Resulting state vector + scalar reward
    Delta:              f32 — magnitude of predicted vs actual difference
    Modality_Map:       Which latent dimensions came from which sense
  }

  Significance:         f(Delta²) + base_value − (time × decay_rate)
  Compression_Tier:     0 | 1 | 2
  Edges:                [ ] Pocket IDs where S(P_current, P_target) ≥ 0.70
  Exceptions:           [ ] Tier0 Pocket IDs that violated this pocket's rule
}
```

---

## 2. Quantization

Compression level is determined by Delta at storage time:

```
IF   delta > THRESHOLD_HIGH                          → Level 0 (float32, full)
ELIF delta > THRESHOLD_MID AND cluster_size < N      → Level 1 (float16, full)
ELIF delta ≤ THRESHOLD_MID AND cluster_size ≥ N      → Level 2 (int8, compressed)
ELIF Tier2 abstraction triggered                     → Level 3 (binary hash)
```

| Level | Format | When | Storage |
|---|---|---|---|
| 0 | float32 | Novel / high surprise | High |
| 1 | float16 | Moderate, soft cluster | Medium |
| 2 | int8 PQ | Familiar, strong cluster | Low |
| 3 | Binary | Tier 2 abstract principle | Negligible |

---

## 3. Universal Experience Encoder (UEE)

All modalities project into a shared 1024-dimensional latent space.

```
Modality      Encoder               Dim Block     Feeds
─────────────────────────────────────────────────────────
Text          Semantic transformer  512–767       Focal, Action
Audio         Conv audio net        256–511       Focal, Ambient
Video         Spatial/temporal CNN  0–255         Focal, Ambient
Sensors       MLP per sensor        768–1023      Ambient (primary)
Hardware      Direct register read  State_Self    Always present
```

### Normalization

All sensor streams normalized before encoding:

```
normalized = (raw − sensor_min) / (sensor_max − sensor_min) → [0, 1]
```

### Signal Binding

```
IF signals arrive within window T
AND encoded vectors show similarity ≥ 0.60
THEN bind into single Pocket
ELSE store separately with temporal Edge
```

### Bootstrapping Sequence

```
Stage 0: Hardware sensors only (numerical — no encoding required)
         Build Self_Model baseline
Stage 1: Physical sensors → sensorimotor rules first
Stage 2: Vision encoder comes online → grounded to sensorimotor
Stage 3: Language encoder comes online → grounded to visual + sensorimotor
```

---

## 4. Similarity Engine

### Core Formula

```
S(P₁, P₂ | C) =
    w_fs(C) × sim(State_Focal₁,   State_Focal₂)
  + w_as(C) × sim(State_Ambient₁, State_Ambient₂)
  + w_a(C)  × sim(Action₁,        Action₂)
  + w_o(C)  × sim(Outcome₁,       Outcome₂)

where Σ weights = 1.0
sim() = cosine similarity in shared latent space
```

### Context Weights

| Context | w_focal | w_ambient | w_action | w_outcome |
|---|---|---|---|---|
| Compression | 0.40 | 0.05 | 0.40 | 0.15 |
| Retrieval | 0.50 | 0.20 | 0.20 | 0.10 |
| Investigation | 0.20 | 0.60 | 0.05 | 0.15 |

### Clustering Thresholds

```
S ≥ 0.90      Strong merge candidate → assign Cluster_ID
S 0.70–0.89   Soft cluster → create Edge
S < 0.70      Novel experience → standalone
```

### Action Similarity — Two-Phase Bootstrapping

**Phase 1:** Taxonomic distance (discrete hierarchy). No learned weights required.  
**Phase 2:** Cosine similarity in learned action embedding space. Activates when embedding loss < CONFIDENCE_THRESHOLD.

---

## 5. Compression

### Tier 0 → Tier 1 Trigger

```
IF cluster_size    ≥ CLUSTER_MIN_N
AND cluster_avg_delta ≤ CLUSTER_MAX_DELTA
THEN compress_cluster(cluster_id) → Tier1_Pocket
```

Note: High-Delta clusters (walls, pain) form but never compress. They are persistent danger maps.

### Tier 1 Structure

```
Tier1_Pocket {
  ID:              hash
  Created:         timestamp
  Source_Cluster:  Cluster_ID
  Source_Count:    u32

  Rule {
    State_Focal:   Centroid + VarianceBounds
    State_Ambient: Centroid (if significant) or null
    Action:        Centroid + VarianceBounds
    Outcome:       ProbabilityDistribution { label → probability }
    Condition_Map: [ BoundaryCondition ]
  }

  Confidence:      f(source_count, outcome_consistency, condition_coverage)
  Compression_Tier: 1
  Edges:           [ Tier1 Pocket IDs ]
  Exceptions:      [ Tier0 Pocket IDs that violated this rule ]
}

BoundaryCondition {
  Feature:         string
  Threshold:       f32
  Direction:       Above | Below
  Outcome_Shifts:  OutcomeLabel
  Confidence:      f32
}
```

### Tier 1 → Tier 2 Trigger

Tier 2 emerges from patterns in Tier 1 failures — not from compressing experiences.

```
Monitor Tier1 failure history
IF failure_pattern detected across multiple rules:
  Analyze: structural property shared by failures
  Generate: abstract principle
  Store: Tier2_Pocket (Level 3, binary hash, permanent)
```

---

## 6. State_Self — Hardware Proprioception

```
State_Self {
  // ARM Performance Monitor Unit
  cpu_cycles:         u64   (PMCCNTR_EL0)
  instructions:       u64   (PMEVCNTR0_EL0)
  cache_misses:       u64   (PMEVCNTR1_EL0)

  // Thermal (BCM2711 register 0xFE212058)
  cpu_temp_celsius:   f32

  // ARM Generic Timer
  timestamp:          u64   (CNTPCT_EL0)

  // Derived
  ipc:                f32   (instructions / cpu_cycles)
  inference_latency:  f32   (cycles for last prediction)

  // Physical I2C Sensors
  ambient_temp:       f32   BME280
  pressure_hpa:       f32   BME280
  humidity_pct:       f32   BME280
  accel_x/y/z:        f32   MPU-6050 (m/s²)
  gyro_x/y/z:         f32   MPU-6050 (°/s)
  distance_mm:        f32   VL53L0X

  // MMU
  pages_allocated:    u32
  page_faults:        u32   ← pain signal
}
```

### Pain Signals

| Condition | Significance | Type |
|---|---|---|
| page_faults spike | 1.0 (MAX) | Memory violation |
| cpu_temp > critical | 1.0 (MAX) | Thermal danger |
| motor stall | 1.0 (MAX) | Physical blockage |
| memory > 95% | 0.9 | Resource exhaustion |
| inference_latency spike | 0.7 | Performance degradation |

---

## 7. World Model and Prediction Engine

### Prediction Pipeline

```
1. Encode current state → Current_State_Vector
2. Encode intended action → Action_Vector
3. Query World Model (Retrieval context weights, top-K)
4. Run Divergence Check on retrieved outcomes
5a. IF agreement: assemble Predicted_Outcome_Distribution
5b. IF contradiction: trigger Bifurcation Engine
6. Action executes
7. Capture Actual_Outcome via UEE
8. Delta = distance(Predicted_Outcome, Actual_Outcome)
9. Update World Model per delta magnitude
```

### Prediction Structure

```
Prediction {
  Predicted_Outcome:   OutcomeDistribution
  Confidence:          f32
  Source_Pockets:      [ Pocket IDs ]
  Weakest_Link:        ModalityType
  Fallback_Used:       bool
  Contradiction_Flag:  bool
}
```

### Cold-Start Phases

```
Phase 0: No pockets → Delta = MAX for all experiences
Phase 1: Sensor Tier1 only → physical cause-effect predictions
Phase 2: Full multi-modal World Model → cross-modal predictions
```

### World Model Update

```
IF delta < THRESHOLD_LOW:
  Nearest Tier1 Confidence += small_increment

IF delta ∈ [THRESHOLD_LOW, THRESHOLD_HIGH):
  Soft update: adjust Outcome_Distribution and Condition_Map

IF delta ≥ THRESHOLD_HIGH:
  Hard update: new Tier0 pocket at Level 0
  Nearest Tier1 Confidence −= penalty
  Preserve State_Ambient for investigation
```

---

## 8. Bifurcation Engine (Contradiction Resolution)

When retrieved pockets show high similarity in State+Action but divergent Outcomes:

```
1. Calculate D_conflict = variance of retrieved outcome vectors
2. IF D_conflict > CONFLICT_THRESHOLD:
     Prediction.Contradiction_Flag = TRUE
     Prediction.Confidence = 0.0
     Prediction.Predicted_Outcome = BIMODAL_DISTRIBUTION

3. Ambient Sweep:
     Pull State_Ambient from conflicting pockets
     Subtract across Modality_Map dimensions

4a. IF divergence found in Ambient dims:
      Promote feature → State_Focal for this cluster
      Create new Condition_Map boundary

4b. IF Ambient is identical (hidden variable unobserved):
      Execute with 0.0 confidence
      Flag Tier0 pocket for Active_Exploration
```

---

## 9. Reasoning Layer

### Goal Structure

```
Goal {
  ID:                 hash
  Target_State:       latent vector
  Success_Threshold:  f32
  Constraints:        [ latent vectors to avoid ]
  Priority:           f32
  Deadline:           Option<u64>
  Origin:             External | InternalDrive |
                      ContradictionResolution | Exploration
}
```

### Planning Algorithm

```
1. Encode Current_State → Current_Vector
2. Encode Goal.Target_State → Goal_Vector
3. Query World Model: find bridge pockets
   (State_Focal ~ Current AND Outcome ~ Goal)
4. IF direct bridge: single-step plan
5. IF no bridge: find intermediates, build chain
6. Per step: check Condition_Map, Confidence, Contradiction_Flag
7. Output: ranked Plan candidates with composite confidence
```

### Plan Structure

```
Plan {
  ID:                    hash
  Goal_ID:               hash
  Composite_Confidence:  f32   (product of step confidences)
  Estimated_Delta_Risk:  StepIndex
  Unexplored_Steps:      u32
  Steps: [
    PlanStep {
      Sequence:           u32
      Action:             Action
      Expected_State:     Prediction
      Source_Pocket:      hash
      Confidence:         f32
      Fallback:           Option<Action>
      Contradiction_Flag: bool
    }
  ]
}
```

### Execution Monitor

```
Per step:
  IF step_delta < THRESHOLD_LOW:     continue
  IF step_delta < THRESHOLD_HIGH:    soft-adjust remaining steps
  IF step_delta ≥ THRESHOLD_HIGH:    HALT → re-plan
                                     IF re-plan fails → Exploration_Goal
```

### Reasoning Layer Interface

```
QUERY(context, state, action)       → Vec<Pocket>
PREDICT(state, action)              → Prediction
PLAN(current_state, goal)           → Vec<Plan>
MONITOR(plan_step, actual_outcome)  → MonitorResult
ABSTRACT(failed_predictions)        → Tier2 trigger
EXPLORE(knowledge_gap)              → Exploration_Goal
```

---

## 10. The Self-Model

```
Self_Model {
  Baseline_Profile {
    healthy_cpu_temp:          Distribution
    healthy_inference_latency: Distribution
    healthy_memory_usage:      Distribution
  }

  Degradation_Map {
    sensor_id → TrendAnalysis {
      direction:  Increasing | Decreasing | Stable
      rate:       f32 (change per 1000 experiences)
      confidence: f32
      first_seen: u64
    }
  }

  Performance_Envelope {
    max_reliable_temp:    f32
    max_reliable_memory:  f32
  }

  Anomaly_History: [ Pocket IDs ]
}
```

---

## 11. Configuration Parameters

Parameters requiring empirical tuning (defaults are starting points):

| Parameter | Default | Description |
|---|---|---|
| THRESHOLD_HIGH | 0.7 | Delta above this = Level 0 quantization |
| THRESHOLD_MID | 0.3 | Delta above this = Level 1 quantization |
| THRESHOLD_LOW | 0.1 | Delta below this = no World Model update |
| CLUSTER_MIN_N | 10 | Minimum pockets to trigger compression |
| CLUSTER_MAX_DELTA | 0.25 | Maximum avg_delta to allow compression |
| SIMILARITY_STRONG | 0.90 | Threshold for Cluster_ID assignment |
| SIMILARITY_SOFT | 0.70 | Threshold for Edge creation |
| CONFLICT_THRESHOLD | 0.40 | D_conflict above this triggers Bifurcation |
| PERSISTENCE_THRESHOLD | 0.3 | Significance above this → save on shutdown |
| PAIN_DECAY_RATE | 0.0001 | Significance decay for pain-signal pockets |

---

## 12. Version History

| Version | Changes |
|---|---|
| v0.1 | Initial Pocket structure: ID, timestamp, significance, payload, edges |
| v0.2 | Added dual-state (Focal + Ambient), SAO + Delta payload |
| v0.3 | Added context-sensitive similarity, clustering thresholds, two-phase action bootstrapping |
| v0.4 | Added UEE multi-modal encoding, quantization levels, State_Self hardware proprioception, Bifurcation engine, Reasoning Layer, Self_Model |
