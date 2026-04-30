# Value Layer Contracts

> Executable contracts for learnable weight adjustment in the value layer.

---

## Scenario: Value Layer Learnable Weights

### 1. Scope / Trigger

- Trigger: The theory requires value weights to be learnable from action outcomes, but the fixed-rule substrate (hard bounds, normalization, metacog veto) must not drift.
- Why this needs code-spec depth: weight adjustment crosses rumination (candidate generation), repository (persistence), value (scoring), and orchestration (runtime application) layers.

### 2. Signatures

- `src/cognition/value.rs`
  - `ValueAdjustment { goal_progress, information_gain, risk_avoidance, resource_efficiency, agent_robustness: f32 }`
  - `ValueConfig::WEIGHT_FLOOR: f32 = 0.05`
  - `ValueConfig::WEIGHT_CEILING: f32 = 0.60`
  - `ValueConfig::apply_adjustment(&self, adjustment: &ValueAdjustment, learning_rate: f32) -> ValueConfig`
  - `ValueConfig::from_persisted_adjustments(base: &ValueConfig, adjustments: &[ValueAdjustment], learning_rate: f32) -> ValueConfig`
- `src/cognition/rumination.rs`
  - `derive_value_adjustment(report) -> Option<ValueAdjustment>`
  - Wired into `derive_long_cycle_candidates()` to produce `ValueAdjustmentCandidate` entries
- `src/agent/orchestration.rs`
  - `WorkingMemoryScoringPort::from_persisted_adjustments(config, adjustments, learning_rate) -> Self`

### 3. Contracts

#### Adjustment contract

- Each dimension's weight is updated: `new_weight = old_weight + (delta * learning_rate)`
- Learning rate is typically 0.01 (very small to ensure slow drift).
- After applying delta, each weight is clamped to `[WEIGHT_FLOOR, WEIGHT_CEILING]`.
- After clamping, all weights are renormalized to sum to exactly 1.0.

#### Safe bounds

- `WEIGHT_FLOOR = 0.05`: no dimension can disappear entirely.
- `WEIGHT_CEILING = 0.60`: no dimension can dominate.
- These bounds are hard constraints that are never overridden by adjustments.

#### No-adjustment identity

- `ValueAdjustment::zero()` applied with any learning rate produces the original config.
- `from_persisted_adjustments(base, &[], 0.01)` returns `base` unchanged.

#### Rumination derivation

- Success outcomes (gate = Warning) produce positive deltas proportional to the selected branch's value vector.
- Failure outcomes (gate = SoftVeto/HardVeto/Escalate) produce negative deltas.
- The adjustment is serialized as JSON into the rumination candidate's `adjustment` field.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Zero adjustment applied | Weights unchanged |
| Positive delta on goal_progress | goal_progress weight increases (after renormalization) |
| Negative delta on risk_avoidance | risk_avoidance weight decreases |
| Delta would push weight below 0.05 | Clamped to 0.05 |
| Delta would push weight above 0.60 | Clamped to 0.60 |
| Multiple adjustments applied sequentially | Each adjustment applied in order with clamping and renormalization |
| Empty adjustments slice | Base config returned unchanged |
| Learning rate = 0.0 | No change regardless of delta magnitude |

### 5. Good / Base / Bad Cases

- Good:
  - Slow drift from action outcomes preserves preference evolution.
  - Hard bounds prevent runaway or vanishing dimensions.
  - No-adjustment path is identity.
- Base:
  - Default weights stay unchanged until rumination generates adjustments.
- Bad:
  - Skipping renormalization so weights no longer sum to 1.0.
  - Applying learning rate = 1.0 so adjustments are too aggressive.
  - Letting floor/ceiling bounds be overridden by caller.

### 6. Tests Required

- `tests/value_adjustment.rs`
  - Assert zero adjustment is identity
  - Assert positive/negative deltas move weights in correct direction
  - Assert normalization preserves sum = 1.0
  - Assert floor/ceiling clamping
  - Assert multi-adjustment batch fold
  - Assert learning rate controls magnitude
  - Assert single-dimension boost respects ceiling

### 7. Wrong vs Correct

#### Wrong

- Apply delta without renormalization, letting weights drift to arbitrary sums.
- Let learning rate be configurable without bounds, allowing 1.0 or negative values.
- Generate value adjustments from every LPQ cycle regardless of outcome quality.

#### Correct

- Apply delta, clamp, renormalize — in that order, every time.
- Default learning rate 0.01; caller can choose smaller.
- Generate adjustments only when action outcomes provide a clear signal (success or failure).

---

## Design Decision: Slow Baseline Drift with Hard Bounds

**Context**: The theory requires `W_baseline_new = W_baseline_old + mu * Delta_outcome`, but unrestricted drift would make the system unpredictable.

**Decision**: Apply adjustments with a very small default learning rate (0.01), hard floor/ceiling bounds per dimension, and mandatory renormalization after each adjustment.

**Why**:
- Slow drift ensures stability across sessions.
- Hard bounds prevent any dimension from dominating or disappearing.
- Renormalization keeps the projection interpretable.

**Consequences**:
- Value preferences evolve from experience within safe bounds.
- Future phases can add per-task dynamic deltas (`Delta W_task/self/metacog`) as an overlay on top of the learned baseline.

---

## Scenario: Threshold Gates + Runtime Dynamic Weight Deltas

### 1. Scope / Trigger

- Trigger: Theory requires `Score(m) = ∏1[vᵢ ≥ θᵢ] · Σwᵢvᵢ` (non-compensatory thresholds) and `W = W_baseline + ΔW_task + ΔW_self + ΔW_metacog` (runtime context-driven weight deltas).
- Why: Without thresholds, no single dimension can block an action; without dynamic deltas, weights can't adapt to current context.

### 2. Signatures

- `src/cognition/value.rs`
  - `ValueThresholds { risk_avoidance, goal_progress, information_gain, resource_efficiency, agent_robustness: f32 }`
  - `ValueThresholds::default()` → `risk_avoidance: 0.15`, all others `0.0`
  - `DynamicWeightDelta { deltas: HashMap<String, f32> }`
  - `DynamicWeightDelta::apply_to(&self, weights: &[f32; 5]) -> [f32; 5]`
  - `derive_dynamic_delta(request: &WorkingMemoryRequest) -> DynamicWeightDelta`
  - `ValueScorer::project_with_delta(&self, values: &ValueVector, delta: Option<&DynamicWeightDelta>) -> ProjectedScore`
- `src/agent/orchestration.rs`
  - `score()` derives `DynamicWeightDelta` from request context before calling `project_with_delta()`

### 3. Contracts

#### Threshold gate contract

- Before computing weighted sum, each dimension is checked against its threshold floor.
- If any dimension `vᵢ < θᵢ` (where `θᵢ > 0.0`), the final score is `0.0` and `threshold_passed = false`.
- Dimensions with threshold `0.0` are unconstrained.
- Thresholds are configurable per-instance, not hardcoded.

#### Dynamic delta contract

- `derive_dynamic_delta()` reads from `WorkingMemoryRequest` fields:
  - `ΔW_task`: from `active_goal` presence (exploration goal → boost info_gain).
  - `ΔW_self`: from `readiness_flags`/`capability_flags` (low readiness → boost robustness).
  - `ΔW_metacog`: from `metacog_flags` (uncertainty → boost info_gain, risk).
- Deltas are ephemeral per-request, never persisted.
- After applying deltas, weights are clamped to `[0.05, 0.60]` and renormalized.

#### Combined scoring flow

1. Compute effective weights: `W_eff = W_baseline + learned_adjustments + dynamic_deltas`.
2. Clamp + renormalize `W_eff`.
3. Check thresholds: if any `vᵢ < θᵢ`, score = 0.0.
4. Otherwise: score = `Σ W_effᵢ · vᵢ`.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| No threshold set on any dimension | All actions scored normally |
| risk_avoidance < 0.15 (default threshold) | Score = 0.0, threshold_passed = false |
| risk_avoidance >= 0.15, other dims below threshold 0.0 | No threshold effect (floor = 0.0 is unconstrained) |
| No dynamic delta provided | Identical scoring to current behavior |
| Exploration goal active | info_gain weight boosted |
| Low readiness flags | robustness weight boosted, efficiency reduced |
| Metacog uncertainty flag | info_gain and risk weights boosted |
| Delta would push weight out of bounds | Clamped to [0.05, 0.60] then renormalized |

### 5. Good / Base / Bad Cases

- Good:
  - Extremely unsafe actions (risk_avoidance < 0.15) are hard-blocked regardless of other dimensions.
  - Weights adapt to context without persisting temporary shifts.
  - No-delta path is identity with current behavior.
- Base:
  - Default thresholds only block very low risk_avoidance.
  - No delta produces same scoring as before.
- Bad:
  - Setting all thresholds high would block nearly every action.
  - Persisting dynamic deltas would cause unwanted drift.

### 6. Tests Required

- `tests/value_thresholds_dynamic_weights.rs`
  - Assert default threshold blocks risk_avoidance < 0.15
  - Assert configurable thresholds on arbitrary dimensions
  - Assert no-threshold passes all
  - Assert derive_dynamic_delta from goal context
  - Assert derive_dynamic_delta from readiness/capability flags
  - Assert derive_dynamic_delta from metacog flags
  - Assert combined threshold + delta scoring
  - Assert no-delta identity

### 7. Wrong vs Correct

#### Wrong

- Check thresholds after computing weighted sum (wastes computation).
- Persist dynamic deltas to database.
- Set default thresholds that block most actions.
- Apply deltas without renormalization.

#### Correct

- Check thresholds first, short-circuit to 0.0 on failure.
- Deltas are computed fresh per request, never stored.
- Conservative defaults: only risk_avoidance has a non-zero floor.
- Clamp + renormalize after every weight modification.
