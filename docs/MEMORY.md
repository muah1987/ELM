# ELM Memory System

> Pocket structure, compression tiers, similarity engine, and clustering

---

## The Pocket

A Pocket is the atomic unit of memory in ELM. Every experience — a sensor reading, a collision, a successful action, a failed prediction — becomes a Pocket the moment it occurs.

Pockets are not logs. They are not embeddings. They are structured records of **what happened, what was expected, and how surprising the difference was.**

### Full Pocket Structure

```rust
struct Pocket {
    id:                 [u8; 32],           // SHA-256 hash
    timestamp:          u64,                // ARM cycle counter (CNTPCT_EL0)
    cluster_id:         Option<[u8; 32]>,   // null if novel
    quantization_level: u8,                 // 0=raw, 1=f16, 2=int8, 3=binary

    payload: ExperiencePayload,

    significance:       f32,                // importance score
    compression_tier:   u8,                 // 0, 1, or 2
    edges:              Vec<[u8; 32]>,      // similar Pocket IDs
    exceptions:         Vec<[u8; 32]>,      // Tier0 rule violations
}

struct ExperiencePayload {
    state_focal:   Vec<f32>,    // 256 dims — high-res attention data
    state_ambient: Vec<f16>,    // 256 dims — background context
    state_self:    StateSelf,   // hardware body snapshot
    action:        Action,      // what was done
    outcome:       Vec<f32>,    // 256 dims — what resulted
    delta:         f32,         // surprise magnitude [0.0 - 1.0]
    modality_map:  ModalityMap, // which dims came from which sense
}
```

### The Dual State

State is not a flat snapshot. It is split by attention level:

**State_Focal** — high resolution, the thing being directly attended to.
- Feeds similarity weight w_fs = 0.40-0.50 depending on context
- Always preserved through compression cycles
- What the action was aimed at

**State_Ambient** — lower resolution, background context.
- Feeds similarity weight w_as = 0.05-0.60 depending on context
- Dropped during compression unless repeatedly significant
- Where hidden variables hide

This split means: during a post-surprise investigation, the ELM can comb through State_Ambient looking for what it wasn't paying attention to that turned out to matter.

---

## Quantization Levels

Compression is variable and driven by Delta. Novel experiences are preserved at full fidelity. Familiar experiences are compressed.

### Level 0 — Raw Experience (float32)

```
Format:    float32
Dims:      full resolution
When:      Delta > threshold_high OR Cluster_ID = null
Cost:      high
Lifetime:  until significance decays OR cluster forms
```

Everything the ELM doesn't recognize yet lives here. Full precision. Nothing thrown away.

### Level 1 — Reduced Precision (float16)

```
Format:    float16
Dims:      full resolution
When:      Delta moderate, soft cluster exists
Cost:      medium (50% of Level 0)
Lifetime:  until Tier 1 compression triggers
```

### Level 2 — Product Quantization (int8)

```
Format:    int8 via product quantization
Dims:      compressed (256 → 64 sub-dimensions per segment)
When:      Delta low, strong cluster exists
Cost:      low (~12% of Level 0)
Lifetime:  merged into Tier 1 rule pocket
```

### Level 3 — Binary Hash

```
Format:    128-bit binary fingerprint
Dims:      N/A — semantic hash only
When:      Tier 2 abstraction
Cost:      negligible
Lifetime:  permanent, never deleted
```

### Quantization Decision Function

```
fn quantize(delta: f32, cluster_size: u32, significance: f32) -> u8 {
    if delta > THRESHOLD_HIGH {
        return 0;
    }
    if delta > THRESHOLD_MID && cluster_size < CLUSTER_MIN {
        return 1;
    }
    if delta <= THRESHOLD_MID && cluster_size >= CLUSTER_MIN {
        return 2;
    }
    // Tier 2 abstraction handled separately
    return 1; // safe default
}
```

---

## Significance

Significance determines how long a Pocket survives before being eligible for compression or deletion.

```
significance = f(delta) + base_value - (time_elapsed * decay_rate)

Where:
  f(delta)    = delta ^ 2          (surprise increases significance quadratically)
  base_value  = 0.1                (everything starts with minimal significance)
  decay_rate  = configurable       (slower for high-significance pockets)
```

High-Delta pockets self-protect. A pocket with significance = 0.95 will decay very slowly. A pocket with significance = 0.05 will decay quickly and be eligible for deletion.

Pain signals (page faults, thermal events, motor stalls) are initialized at significance = 1.0 and decay extremely slowly. These memories do not fade easily.

---

## Similarity Engine

### The Core Formula

```
S(P1, P2 | C) =
    w_fs(C) * sim(state_focal_1,   state_focal_2)
  + w_as(C) * sim(state_ambient_1, state_ambient_2)
  + w_a(C)  * sim(action_1,        action_2)
  + w_o(C)  * sim(outcome_1,       outcome_2)
```

Where `sim()` is cosine similarity in the shared latent space.

### Context Weights

The same two Pockets have different similarity scores depending on why you are asking:

```
Context: Compression
  (Are these experiences similar enough to merge?)
  w_fs = 0.40, w_as = 0.05, w_a = 0.40, w_o = 0.15

Context: Retrieval
  (Is this past experience relevant to my current situation?)
  w_fs = 0.50, w_as = 0.20, w_a = 0.20, w_o = 0.10

Context: Investigation
  (Why was I surprised? What did I miss?)
  w_fs = 0.20, w_as = 0.60, w_a = 0.05, w_o = 0.15
```

Investigation mode dramatically upweights State_Ambient — because the hidden variable that caused the surprise is almost certainly lurking in the background context.

### Clustering Thresholds

```
S >= 0.90   Strong merge candidate
            → Assign shared Cluster_ID
            → Schedule for compression if cluster_size >= N

S 0.70-0.89 Soft cluster
            → Create Edge pointer between pockets
            → No Cluster_ID yet
            → Monitor for growth

S < 0.70    Novel experience
            → No cluster assignment
            → Standalone Pocket
```

### Action Similarity Bootstrapping

At cold start, there is no trained action embedding space. Similarity between actions falls back to **taxonomic distance**:

```
Action Taxonomy Example (robot):
  Physical
  ├── Locomotion
  │   ├── Forward
  │   ├── Backward
  │   ├── Left
  │   └── Right
  └── Manipulation
      ├── Grasp
      ├── Release
      └── Push

Similarity calculation:
  Same leaf node:   1.0   (Forward vs Forward)
  Same parent:      0.6   (Forward vs Backward)
  Same grandparent: 0.3   (Forward vs Grasp)
  No relation:      0.0
```

Crossover to learned embeddings happens automatically when the action embedding model's predictive loss drops below `EMBEDDING_CONFIDENCE_THRESHOLD`.

---

## Compression — Tier 0 to Tier 1

### Compression Trigger

```
IF cluster_size >= CLUSTER_MIN_N
AND cluster_avg_delta <= CLUSTER_MAX_DELTA
THEN trigger_compression(cluster_id)
```

Important: High-Delta clusters (wall collisions, pain events) accumulate Pockets and form clusters but never trigger compression because their avg_delta stays high. They form persistent high-significance maps — exactly correct.

### Tier 1 Rule Structure

A Tier 1 pocket is not a mean experience. Averaging loses variance, and variance contains the boundaries of where rules break.

```rust
struct Tier1Pocket {
    id:            [u8; 32],
    created:       u64,
    source_cluster: [u8; 32],
    source_count:  u32,

    rule: GeneralizedRule {
        state_focal:   CentroidWithVariance,
        state_ambient: Option<CentroidWithVariance>, // dropped if not significant
        action:        CentroidWithVariance,
        outcome:       OutcomeDistribution,
        condition_map: Vec<BoundaryCondition>,
    },

    confidence:       f32,
    compression_tier: 1,
    edges:            Vec<[u8; 32]>,
    exceptions:       Vec<[u8; 32]>,
}

struct CentroidWithVariance {
    centroid: Vec<f32>,
    variance: Vec<f32>,
}

struct OutcomeDistribution {
    outcomes: HashMap<OutcomeLabel, f32>,  // label → probability
}

struct BoundaryCondition {
    feature:        String,
    threshold:      f32,
    direction:      ThresholdDirection,  // Above | Below
    outcome_shifts: OutcomeLabel,
    confidence:     f32,
}
```

The **Condition Map** is what separates a Tier 1 rule from a naive average. It records: *when does this rule stop being true?*

Exception Pockets — Tier 0 pockets that violated the rule — are never deleted. They are preserved at Tier 0 and pointed to from the Tier 1 exception list. If enough exceptions accumulate, they seed a new rule.

### Tier 1 Confidence

```
confidence = f(source_count, outcome_consistency, condition_map_coverage)

source_count:           more experiences → higher confidence
outcome_consistency:    low outcome variance → higher confidence
condition_map_coverage: well-defined boundaries → higher confidence
```

A rule built from 3 experiences has low confidence. Built from 300, high. The reasoning layer always knows how much to trust a rule it retrieves.

---

## Compression — Tier 1 to Tier 2

### Tier 2 Abstraction Trigger

Tier 2 emerges from analyzing patterns in **Tier 1 failures**, not from compressing more experiences.

```
Monitor Tier 1 pockets for:
  repeated failures of similar rules
  consistent structural property in failures

IF failure_pattern detected:
  Analyze: what do these failures have in common?
  Generate: abstract principle
  Store as: Tier 2 pocket (Level 3, binary hash)
```

### Tier 2 Pocket Structure

```rust
struct Tier2Pocket {
    id:              [u8; 32],
    created:         u64,
    source_rules:    Vec<[u8; 32]>,  // Tier1 rules that generated this
    
    principle:       AbstractPrinciple {
        description:     Vec<f32>,  // latent space encoding of principle
        scope:           PrincipleScope,
        applies_to:      Vec<RuleCategory>,
        failure_pattern: Vec<f32>,  // what failure looks like
    },

    confidence:       f32,
    compression_tier: 2,
}
```

Tier 2 pockets are never deleted. They are the ELM's deepest knowledge — abstract understanding of how its rules work and fail.

---

## Memory Bank Operations

### Write

```
fn store_pocket(payload: ExperiencePayload) -> PocketId {
    let delta = payload.delta;
    let q_level = quantize(delta, 0, 0.0);
    
    let pocket = Pocket::new(payload, q_level);
    let similar = find_similar(pocket, Context::Compression);
    
    assign_cluster_or_edges(pocket, similar);
    check_compression_trigger(pocket.cluster_id);
    
    memory_bank.insert(pocket)
}
```

### Read

```
fn retrieve(state: Vector, action: Action, k: usize) -> Vec<Pocket> {
    let query = encode_query(state, action);
    ann_search(query, k, Context::Retrieval)
}
```

### ANN Search

Approximate Nearest Neighbor search over the memory bank. For MVP scale (< 100K pockets) a simple HNSW index is sufficient. At larger scales, partitioned search by Cluster_ID first.

---

## Persistence

Memory must survive power cycles. Without persistence, every boot is amnesia.

### What is saved on shutdown

```
Tier 0: Pockets with significance > PERSISTENCE_THRESHOLD
Tier 1: All rules (always saved)
Tier 2: All principles (always saved)
Self_Model: Baseline profile and degradation map
Action embedding model: current weights
```

### Storage format

Raw byte serialization to NVMe block device. No filesystem required. ELM writes directly to block addresses it owns.

```
Block layout:
  0x0000 - 0x0FFF    Boot metadata
  0x1000 - 0x1FFF    Self_Model
  0x2000 - 0x2FFF    Memory Bank index
  0x3000 - end       Pocket storage (variable length records)
```

See [docs/HARDWARE.md](HARDWARE.md) for NVMe driver details.
