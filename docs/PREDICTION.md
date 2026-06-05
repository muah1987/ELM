# ELM Prediction Engine

> World model, Delta calculation, contradiction resolution, and cold-start bootstrapping

---

## Overview

The Prediction Engine is the heartbeat of ELM. Without it, Delta is undefined — and Delta is the engine of everything else: compression decisions, significance weighting, world model updates, and the learning curve itself.

The core question it answers before every action:

```
Given what I know right now — my current state and what I am about to do —
what do I expect to happen?
```

The answer comes entirely from the ELM's own compressed past experience. The Prediction Engine does not have independent knowledge. It queries the World Model (Tier 1 and Tier 2 pockets) and assembles a prediction from what the ELM has already learned. This is what prevents hallucination at the architectural level — the reasoning layer can only work with territory it has actually experienced.

---

## The Prediction Pipeline

Every action cycle runs this sequence:

```
1.  Encode current State → Current_State_Vector (via UEE)
2.  Encode intended Action → Action_Vector
3.  Query World Model with Retrieval context weights
      w_fs=0.50, w_as=0.20, w_a=0.20, w_o=0.10
    → Returns top-K most similar Tier 1 / Tier 2 pockets
4.  Run Divergence Check on retrieved outcomes
      → D_conflict = variance of outcome vectors
5a. IF D_conflict <= CONFLICT_THRESHOLD:
      Assemble Predicted_Outcome_Distribution (weighted by similarity + confidence)
5b. IF D_conflict > CONFLICT_THRESHOLD:
      Trigger Bifurcation Engine (see below)
6.  Action executes
7.  Capture Actual_Outcome via UEE
8.  Delta = cosine_distance(Predicted_Outcome, Actual_Outcome)
9.  Update World Model per Delta magnitude
10. Store new Pocket with Delta as primary field
```

---

## Prediction Output Structure

```rust
pub struct Prediction {
    /// The expected outcome distribution
    pub predicted_outcome: Vec<f32>,

    /// How confident the prediction is [0.0 - 1.0]
    /// 0.0 = no basis for prediction (cold start or contradiction)
    pub confidence: f32,

    /// Which Tier 1/2 pockets contributed to this prediction
    pub source_pockets: Vec<PocketId>,

    /// Which modality had the lowest coverage
    /// Used by the reasoning layer to know where the blind spot is
    pub weakest_link: Option<ModalityType>,

    /// Whether a Phase 0/1 fallback was used
    pub fallback_used: bool,

    /// Whether the Bifurcation Engine was triggered
    pub contradiction_flag: bool,
}
```

The `weakest_link` field is one of the most valuable outputs. Before the action even executes, the ELM knows which part of its world model is least reliable for this situation. The reasoning layer can use this to prepare maximum-fidelity recording on that specific modality.

---

## Cold-Start Bootstrapping — Three Phases

At birth, ELM has no Tier 1 pockets. It cannot retrieve anything. Prediction is impossible. This is handled gracefully through three phases:

### Phase 0 — Null Prediction

```
Condition:  No pockets exist
Prediction: NULL
Confidence: 0.0
Delta:      MAX_VALUE for everything
Fallback:   true
```

This is correct behavior. Every experience is maximally surprising. Every pocket is stored at Quantization Level 0 — full float32 precision. The ELM is building its foundational memory at maximum fidelity, which is exactly what a newborn should do.

### Phase 1 — Sensor-Anchored Prediction

```
Condition:  Sensor-based Tier 1 pockets exist (no vision/language yet)
Prediction: Extrapolated from physical cause-effect rules only
Confidence: Low to medium
Delta:      Deviation from physical expectation
Fallback:   Partial
```

The ELM can now predict: "Moving North usually moves my position by (0, -1)." It cannot yet predict temperature gradients reliably or handle wall collisions without experience. Each new collision is still high-Delta.

### Phase 2 — Full World Model

```
Condition:  Multi-modal Tier 1 pockets exist
Prediction: Weighted ensemble across all modalities
Confidence: Grows with source_count and consistency
Delta:      True surprise relative to full learned model
Fallback:   false
```

Cross-modal predictions become possible. The ELM can predict that moving East near a known wall cluster will produce `touch=1`, even if it has not been to this exact cell, because the wall pattern is generalized in a Tier 1 rule.

---

## World Model Update Rules

After every experience, the World Model updates based on how surprised ELM was:

```
Delta < THRESHOLD_LOW  (expected outcome, prediction was good):
  No structural update
  Source Tier 1 pocket Confidence += CONFIDENCE_INCREMENT
  Effect: reliable rules become more trusted over time

Delta in [THRESHOLD_LOW, THRESHOLD_HIGH)  (mild surprise):
  Soft update
  Adjust Outcome_Distribution of nearest Tier 1 pocket
  Nudge Condition_Map boundaries slightly toward new data
  Effect: rules stay accurate as the world slowly changes

Delta >= THRESHOLD_HIGH  (genuine surprise):
  Hard update
  New Tier 0 pocket created at Quantization Level 0
  Nearest Tier 1 pocket Confidence -= CONFIDENCE_PENALTY
  State_Ambient preserved at full resolution for investigation
  Investigation mode triggered: Ambient Sweep runs
  Effect: world model flags that something has changed
```

The key insight: the World Model only pays compute cost when it is actually wrong. Confirmed predictions cost almost nothing to process. Surprises trigger the full investigation pipeline.

---

## Bifurcation Engine — Contradiction Resolution

When two retrieved pockets are highly similar in State and Action but have wildly different Outcomes, the ELM has encountered a **POMDP trap** — a situation where the current observable state is missing a hidden variable that determines the real outcome.

Averaging the conflicting outcomes would produce a prediction that never actually occurred. This is how standard statistical models hallucinate. ELM never averages contradictions.

### Detection

```
D_conflict = mean pairwise cosine distance between retrieved outcome vectors

IF D_conflict > CONFLICT_THRESHOLD:
    contradiction detected
```

### Response

```
1. Prediction.contradiction_flag = TRUE
   Prediction.confidence = 0.0
   Prediction.predicted_outcome = BIMODAL (do not collapse to mean)

2. Ambient Sweep:
   Pull State_Ambient from each conflicting pocket
   Subtract across Modality_Map dimensions
   Find the dimension block with highest variance

3a. IF divergence found in ambient dims:
      The hidden variable was present but unattended
      Promote that feature → State_Focal for this cluster
      Create new Condition_Map boundary on nearest Tier 1 rule
      Re-query with updated weights
      Confidence restored

3b. IF ambient dims are identical (hidden variable completely unobserved):
      Execute action with confidence = 0.0
      Prepare for maximum-Delta recording
      Flag resulting Tier 0 pocket for Active Exploration
      ("I need to design an experiment to understand this")
```

### Example

```
Pocket A: State=(3,4), Action=East, Outcome=moved_to(4,4) ← success
Pocket B: State=(3,4), Action=East, Outcome=blocked         ← failure

D_conflict: HIGH → Bifurcation triggered

Ambient Sweep:
  Pocket A ambient: temperature=18.2, touch_history=0
  Pocket B ambient: temperature=18.2, touch_history=0
  → Ambients are identical
  → Hidden variable: unknown

Result:
  ELM steps East with confidence=0.0
  Wall is hit → Delta=MAX
  Full Tier 0 pocket stored
  Active Exploration flag set
  Future experiment: probe (4,4) from different approach angles
```

Over time, Active Exploration flags accumulate into a queue that the Reasoning Layer uses to design targeted experiments — minimum-risk actions that maximize information gain about the unknown.

---

## Prediction Error Gradient — Path to Tier 2

The Prediction Engine tracks not just individual Delta values but **patterns of failure** across Tier 1 rules. This is the mechanism that eventually seeds Tier 2 abstraction.

```
Prediction_Error_Log {
    which Tier 1 rules fail most often
    which modality was weakest_link most often
    which Condition_Map boundaries are violated most often
    structural properties shared by failures
}
```

When a pattern emerges — multiple different Tier 1 rules failing for the same structural reason — the ELM generates a Tier 2 principle:

```
Tier 1 failures:
  "Force limit changes with temperature"    (confidence dropped 0.9 → 0.3)
  "Force limit changes with material type"  (confidence dropped 0.8 → 0.2)
  "Force limit changes with surface state"  (confidence dropped 0.7 → 0.4)

Shared structure: "force limits are not fixed — they vary with context"

Tier 2 principle emerges:
  "Physical interaction limits are context-dependent.
   Any rule about force, pressure, or resistance must include
   environmental state as a conditioning variable."
```

This principle then informs how the Prediction Engine weights future Tier 1 rule retrieval — rules that include environmental conditioning get higher confidence than rules that don't, in physical interaction contexts.

---

## Prediction Confidence Over Time

The learning curve — the proof that ELM works — is directly readable from the Prediction Engine's output:

```
Early life (Phase 0-1):
  Average confidence: 0.0 - 0.2
  Average Delta: 0.7 - 1.0
  Every experience is new territory

Mid life (Phase 1-2 transition):
  Average confidence: 0.3 - 0.6
  Average Delta: 0.3 - 0.6
  Familiar paths predicted reliably
  New territory still causes surprise spikes

Mature (Phase 2):
  Average confidence: 0.7 - 0.9 on known paths
  Average Delta: 0.05 - 0.2 on known paths
  Spikes only when genuinely new situations encountered
  Spike magnitude itself becomes informative:
    Small spike = slight variation of known rule
    Large spike = genuinely new territory
```

The shape of the Delta curve over time is the ELM's learning curve. It is not a loss function from backpropagation. It is a direct measure of how surprised the agent is by a world it is actively learning to predict.

---

## Configuration Parameters

| Parameter | Default | Description |
|---|---|---|
| `RETRIEVAL_K` | 5 | Number of top-K pockets retrieved per prediction |
| `CONFLICT_THRESHOLD` | 0.40 | D_conflict above this triggers Bifurcation |
| `THRESHOLD_LOW` | 0.10 | Delta below this = no world model update |
| `THRESHOLD_HIGH` | 0.70 | Delta above this = hard update + investigation |
| `CONFIDENCE_INCREMENT` | 0.02 | Confidence gain per confirmed prediction |
| `CONFIDENCE_PENALTY` | 0.15 | Confidence loss per failed prediction |

All parameters require empirical tuning. The defaults are starting points for the MVP. The MVP's failure mode diagnostics (see `docs/MVP.md`) will reveal which parameters need adjustment first.
