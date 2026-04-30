# World Model Prediction Capability

## Goal

Add LLM-based prediction/simulation to the world model so the system can answer "if I take action A, what would likely happen?" — enabling the action system to evaluate consequences before committing.

## What I Already Know

- Theory doc defines: `Simulate(s_t, a) → ŝ_{t+1}, r̂, û` (next state, expected risk, expected uncertainty)
- `ProjectedWorldModel` currently only has `CurrentWorldSlice` (description layer only)
- `ActionCandidate` already has `expected_effects: Vec<String>` and `intent`
- `ActionBranch` already has `supporting_evidence` and `risk_markers`
- `SkillMemoryTemplate` has `ExpectedOutcome` with `success_criteria` and `failure_modes`
- The system uses `rig` for LLM integration via `TypedPrompt<T>` pattern (see `summary.rs`)
- World model snapshots are keyed by `subject_ref + world_key`; "current" is used for runtime

## Requirements

- Define `PredictedWorldSlice` with: affected fragment IDs, predicted changes, new risks, uncertainty delta
- Define `SimulationResult` combining: predicted slice, confidence, trace
- Implement `simulate(world, action, llm_backend) -> SimulationResult` using rig TypedPrompt
- The prompt sends current world fragments + action summary + expected effects as context
- The LLM returns structured JSON parsed into `PredictedWorldSlice`
- Prediction must not mutate the current world model
- Integration point: assembly can optionally request prediction for action branches
- Prediction is opt-in: no prediction unless explicitly requested

## Acceptance Criteria

- [ ] `PredictedWorldSlice` struct with affected fragments, risk changes, uncertainty delta
- [ ] `SimulationResult` struct with predicted slice, confidence, trace
- [ ] `simulate()` function using rig TypedPrompt for structured LLM output
- [ ] Prompt construction includes world fragments and action details
- [ ] Prediction does not mutate current world model
- [ ] Graceful degradation when LLM is unavailable (returns error, not panic)
- [ ] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, `cargo check --tests` pass
- [ ] Focused tests cover prompt construction and result parsing

## Definition of Done

- Production code updated
- Tests added/updated
- No new database tables (predictions are ephemeral)
- No external surface added
- Trellis check passes

## Out of Scope

- New database tables or snapshot keys for predictions (ephemeral only)
- Structural pattern-matching fallback (can be added later)
- Full causal graph or transition model
- Automatic prediction on every assembly (must be opt-in)
- CLI/HTTP/MCP/UI for predictions
- Predictions that auto-mutate the current world model

## Technical Notes

### Primary files

- `src/cognition/world_model.rs` — `PredictedWorldSlice`, `SimulationResult`, `simulate()`
- `src/cognition/mod.rs` — re-exports
- `src/core/config.rs` — LLM config (already has `RootLlmConfig`)

### Key patterns to follow

- Use `rig::completion::TypedPrompt` + `schemars::JsonSchema` for structured output (same as `summary.rs`)
- Use `RigSummaryBackend` pattern for LLM backend abstraction
- Prompt should be self-contained and include: world fragment summaries, action kind/summary/intent/expected_effects

## Decision (ADR-lite)

**Context**: Theory requires `Simulate(s_t, a) → ŝ_{t+1}, r̂, û`. Multiple implementation paths possible.

**Decision**: LLM-based simulation using rig TypedPrompt for structured output. Predictions are ephemeral (not persisted), opt-in (not automatic), and never mutate the current world model.

**Consequences**: Flexible prediction that works across domains. Future phases can add structural pattern-matching as a fast-path fallback, and persist predictions as needed.
