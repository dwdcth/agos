# Value Layer Learnable Weights

## Goal

Bridge rumination-generated value adjustment candidates back into runtime `ValueScorer` so that long-cycle rumination can slowly update the baseline weight vector based on action outcomes, while preserving the fixed-rule substrate (hard bounds, normalization, and metacog veto remain untouched).

## What I Already Know

- `ValueConfig` has 5 weights (goal=0.35, info=0.15, risk=0.15, resource=0.20, robust=0.15), currently static constants
- `RuminationCandidateKind::ValueAdjustmentCandidate` exists in the repo but no code generates or applies it
- Theory doc says: `W_baseline_new = W_baseline_old + mu * Delta_outcome`, with very small learning rate
- Theory doc says: fixed-rule substrate must NOT be auto-learned; only soft preferences change
- `ValueScorer` owns a `ValueConfig` and is constructed once per `MetacognitionPort`
- Rumination already has LPQ (long-period queue) and can generate candidates; it just doesn't produce value adjustments yet

## Assumptions

- The first implementation should be conservative: rumination writes a value adjustment candidate, a runtime helper applies it to `ValueConfig` with small learning rate
- No new database tables — use existing `rumination_candidates` table with `ValueAdjustmentCandidate` kind
- The adjustment candidate payload should specify which dimension(s) to adjust and by how much (signed delta)
- Hard floor/ceiling constraints on each weight dimension to prevent drift beyond safe bounds
- Normalization is re-applied after each adjustment to keep weights summing to ~1.0

## Requirements

- Define a `ValueAdjustment` struct that captures dimension-level signed deltas
- Add a `ValueConfig::apply_adjustment(adjustment, learning_rate)` method that:
  - Applies signed delta * learning_rate to each specified dimension
  - Clamps each weight to a safe range (e.g., [0.05, 0.60])
  - Re-normalizes all weights to sum to 1.0
  - Returns the new `ValueConfig`
- Add `ValueConfig::from_persisted_adjustments(base, adjustments, learning_rate)` that folds multiple adjustments
- Wire into rumination LPQ: when long-cycle rumination processes action outcomes, it may produce a `ValueAdjustmentCandidate` with the adjustment payload
- Wire into `ValueScorer`: on construction or before scoring, optionally load persisted adjustments and apply them to the base config
- Preserve the fixed-rule substrate: hard bounds, normalization, metacog veto are never affected by weight adjustments
- Add focused tests for: single adjustment, multiple adjustments, clamping, normalization, floor/ceiling, no-adjustment baseline

## Acceptance Criteria

- [ ] `ValueAdjustment` struct with dimension-level signed deltas
- [ ] `ValueConfig::apply_adjustment()` with learning rate, clamping, and normalization
- [ ] `ValueConfig::from_persisted_adjustments()` for batch application
- [ ] Rumination can generate `ValueAdjustmentCandidate` entries with adjustment payload
- [ ] `ValueScorer` can be constructed from a base config + persisted adjustments
- [ ] Weights never drift beyond [0.05, 0.60] per dimension
- [ ] Weights always sum to 1.0 after adjustment
- [ ] No-adjustment path produces identical `ValueConfig::default()`
- [ ] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, `cargo check --tests` pass
- [ ] Focused tests cover adjustment, clamping, normalization, and batch fold

## Definition of Done

- Production code updated
- Tests added/updated
- No new schema (use existing rumination_candidates table)
- No external surface added
- Trellis check passes

## Out of Scope

- Dynamic weight adjustment based on task/self/metacog context at scoring time (that's `Delta W_task/self/metacog` in the theory, separate from baseline learning)
- Non-compensatory threshold gates (theory section 5.3)
- Role-specific fixed overlays
- New database tables or HTTP/MCP/UI surfaces

## Technical Notes

### Primary files

- `src/cognition/value.rs` — `ValueAdjustment`, `apply_adjustment`, `from_persisted_adjustments`
- `src/cognition/rumination.rs` — generate `ValueAdjustmentCandidate` in LPQ
- `src/agent/orchestration.rs` — construct `ValueScorer` with adjusted config

### Key constraints

- Learning rate should be very small (e.g., 0.01) — baseline drifts slowly
- Floor per dimension: 0.05 (no dimension disappears)
- Ceiling per dimension: 0.60 (no dimension dominates)
- After clamping, renormalize so sum == 1.0

## Decision (ADR-lite)

**Context**: Theory requires value weights to be learnable from action outcomes, but fixed-rule substrate must not drift.

**Decision**: Add `ValueAdjustment` as a signed-delta payload, apply through `ValueConfig::apply_adjustment()` with learning rate, clamping, and renormalization. Rumination LPQ generates adjustments; `ValueScorer` applies them at construction time.

**Consequences**: Value preferences can evolve from experience while staying within safe bounds. Future phases can add per-task dynamic deltas (`Delta W_task/self/metacog`).
