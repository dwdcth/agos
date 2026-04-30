# Value Threshold Function + Dynamic Runtime Weights

## Goal

Complete the value layer's theory compliance by adding: (1) non-compensatory threshold gates so certain dimensions can block an action regardless of total score, and (2) runtime dynamic weight deltas (ΔW_task, ΔW_self, ΔW_metacog) that adjust weights based on current context before scoring.

## What I Already Know

- Theory formula: `Score(m) = ∏1[vᵢ ≥ θᵢ] · Σwᵢvᵢ` — some dimensions have hard thresholds
- Theory formula: `W = W_baseline + ΔW_task + ΔW_self + ΔW_metacog` — weights adjust dynamically
- `ValueScorer.project()` currently only does linear weighted sum: `Σ wᵢ · vᵢ`
- `ValueConfig` has baseline weights + learnable adjustments via `apply_adjustment()`
- Metacognition already provides veto at the gate level, but not inside the scoring formula
- Working memory request carries active_goal, active_risks, metacog_flags, readiness_flags, capability_flags

## Requirements

### 1. Threshold gates
- Add `ValueThresholds` struct with optional floor per dimension (e.g., risk_avoidance floor = 0.3)
- If any dimension's value is below its threshold, the projected score becomes 0.0 (hard block)
- Dimensions without a threshold are unconstrained (floor = 0.0)
- Default thresholds: `risk_avoidance: 0.15` (extremely unsafe actions are blocked), all others 0.0
- Thresholds are configurable, not hardcoded

### 2. Dynamic weight deltas
- `DynamicWeightDelta` struct with signed deltas per dimension
- Derived from current request context:
  - ΔW_task: from active_goal presence and content (e.g., exploration goal → boost info_gain)
  - ΔW_self: from readiness/capability flags (e.g., low readiness → boost robustness, reduce efficiency)
  - ΔW_metacog: from metacog_flags (e.g., uncertainty → boost info_gain, risk)
- Deltas are applied ON TOP of the learned baseline, not persisted
- After applying deltas, weights are clamped to [0.05, 0.60] and renormalized

### 3. Integration into ValueScorer
- `ValueScorer` accepts optional `DynamicWeightDelta` at scoring time
- `project()` applies: baseline + learned adjustments + dynamic deltas → effective weights
- Then checks thresholds, then computes weighted sum
- `ProjectedScore` includes: final_score, effective_weight_snapshot, threshold_passed

### 4. Assembly/orchestration integration
- `resolved_dynamic_weights()` on WorkingMemoryRequest derives ΔW from context
- Forward into scoring port

## Acceptance Criteria

- [ ] `ValueThresholds` with configurable per-dimension floors
- [ ] `DynamicWeightDelta` with per-dimension signed deltas
- [ ] `derive_dynamic_delta()` from goal/risks/metacog/readiness/capability
- [ ] `ValueScorer.project()` checks thresholds before scoring
- [ ] `ValueScorer.project()` applies dynamic deltas on top of baseline
- [ ] Actions below threshold score 0.0 regardless of other dimensions
- [ ] No delta produces identical scoring to current behavior
- [ ] Tests for thresholds, dynamic deltas, combined behavior
- [ ] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, `cargo check --tests` pass

## Out of Scope

- Persisting dynamic deltas (they are ephemeral per-request)
- New database tables
- Changing metacog veto (it remains an independent gate)
- Threshold gates that replace metacog veto entirely

## Technical Notes

- Primary files: `src/cognition/value.rs`, `src/agent/orchestration.rs`
- Threshold gates are inside the scoring formula, not a separate gate
- Dynamic deltas are computed fresh for each request, not accumulated
