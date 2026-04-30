# Attention System Deepening

## Goal

Deepen the attention system from a lightweight additive bonus to the theory's three-modulation architecture: ContextBase (situation-driven), EmotionModulator (multiplicative mask), Inhibition (self-model constraint), and Baseline persistence with smooth update.

## What I Already Know

- `AttentionBaseline` is currently an empty struct
- `derive_delta()` only produces task-level cues from request metadata
- Scoring only has additive attention_bonus; no multiplicative modulation, no inhibition
- Theory requires: `Score(m) = cosine_sim(q, v_m) + α·Salience(m) - β·Inhibition(m, self_model) + γ·GoalAssociation(m, g)`
- Theory requires: `E(c, e, i) = c ⊙ (1 + i·Mₑ)` for emotion modulation
- Theory requires: `Baseline(t+1) = Baseline(t) + η·(ObservedState - Baseline(t))` for session continuity
- Theory requires: `Inhibition(m) = Σ wᵢ·Match(self_constraintᵢ, m.features)` for self-model gating

## Requirements

### ContextBase
- `AttentionBaseline` becomes a struct with configurable dimensions (initially: time_pressure, cognitive_load, uncertainty_level, exploration_mode)
- Each dimension is f32 in [0.0, 1.0], defaulting to 0.5 (neutral)
- Baseline persists in-memory within a session (no DB table needed for MVP)
- Smooth update: `Baseline(t+1) = Baseline(t) + η·(Observed - Baseline(t))` with η=0.1

### EmotionModulator
- `EmotionModulator` struct with emotion label + intensity + per-dimension mask
- Predefined emotion profiles: `neutral`, `cautious`, `curious`, `urgent`
- Each profile defines a mask vector that multiplicatively scales ContextBase dimensions
- Formula: `E(c, e, i) = c ⊙ (1 + i · Mₑ)`
- Integrated into `compute_attention_bonus`: modulated baseline affects cue matching sensitivity

### Inhibition
- `Inhibition` struct computed from self-model constraints (capability flags, readiness flags, risk markers)
- Each constraint creates a negative bias for matching candidates
- Formula: `Inhibition(m) = Σ wᵢ·Match(self_constraintᵢ, m.features)`
- Integrated into scoring: `attention_bonus = modulated_bonus - inhibition_penalty`

### MetacogModifier
- `MetacogModifier` struct that can adjust: goal weight multiplier, diversity temperature, inhibition strength
- Derived from metacog flags: Warning → slight risk boost, SoftVeto → strong risk boost
- Applied to attention delta before scoring

### Baseline Persistence
- `AttentionBaseline::update(observed, learning_rate)` method
- Called after each assembly to slowly adapt baseline
- In-memory only (no DB) for MVP

## Acceptance Criteria

- [ ] `AttentionBaseline` has dimension fields, update method, smooth learning
- [ ] `EmotionModulator` with predefined profiles and multiplicative mask
- [ ] `Inhibition` computed from self-model constraints
- [ ] `MetacogModifier` derived from metacog flags
- [ ] `compute_attention_bonus` uses modulated baseline, applies inhibition
- [ ] Scoring formula reflects the theory's structure (not just flat additive)
- [ ] All existing tests still pass
- [ ] New tests for each modulation path
- [ ] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, `cargo check --tests` pass

## Out of Scope

- Baseline persistence to database (in-memory session only)
- Physical/biological dimensions (season, temperature, hunger) — using cognitive dimensions instead
- External API for setting emotion state

## Technical Notes

- Primary files: `src/cognition/attention.rs`, `src/search/score.rs`, `src/cognition/assembly.rs`
- Predefined emotion profiles keep the system deterministic without LLM
- Inhibition reuses the existing cue-matching logic but with negative weight
