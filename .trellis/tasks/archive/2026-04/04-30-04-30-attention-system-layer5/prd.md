# Attention System Layer 5

## Goal

Implement the attention system as an independent cognition layer that determines what memories are worth recalling first, based on cognitive state rather than just query relevance. The attention system acts as the engine that drives background structures into the foreground.

## What I Already Know

- The theory doc `doc/0415-注意力状态.md` defines a three-part structure: `AttentionState = (ContextBase, EmotionModulator, GoalBias)`
- The spec `.trellis/spec/backend/cognition-retrieval-contracts.md` already has a detailed "Attention State Retrieval Bias" scenario with signatures, contracts, and test requirements
- `ScoreBreakdown` already has `emotion_bonus: f32` (currently hardcoded to `0.0`) and `final_score`
- `ResultTrace` already exists but has no `attention` field
- `WorkingMemoryRequest` already carries `active_goal`, `active_risks`, `metacog_flags`, `capability_flags`, `readiness_flags` but has no `attention_state` field
- The spec contract says: attention is additive in scoring, must not replace lexical explanation, and must produce a trace when active
- The spec contract says: explicit empty `AttentionState` on `WorkingMemoryRequest` disables derived fallback bias
- The spec contract says: `resolved_attention_state()` uses explicit state first, then derives from goal/risk/metacog/readiness/capability flags

## Assumptions

- The first implementation should follow the spec that's already written in `cognition-retrieval-contracts.md`
- `ContextBase` with full biological/physical dimensions (season, temperature, hunger, fatigue) is out of scope for MVP; we derive attention from the request metadata that already exists on `WorkingMemoryRequest`
- `EmotionModulator` as a mask-vector multiplicative modulator is out of scope for MVP; emotion_bonus stays as an additive scalar
- Baseline persistence (across sessions) is out of scope for MVP
- The MVP focuses on: type definitions, Delta derivation from request metadata, integration into scoring and trace, and assembly wiring

## Open Questions

None blocking — the spec is detailed enough to proceed.

## Requirements

- Define attention types: `AttentionCue`, `AttentionBaseline`, `AttentionDelta`, `AttentionState`, `AttentionTrace`, `AttentionContribution`
- Add `attention_state: Option<AttentionState>` to `SearchRequest`
- Add `attention_state: Option<AttentionState>` to `WorkingMemoryRequest`
- Implement `WorkingMemoryRequest::resolved_attention_state()` that derives from goal/risk/metacog/readiness/capability when no explicit state is given
- Add `attention_bonus: f32` to `ScoreBreakdown` (rename from `emotion_bonus`)
- Add `attention: Option<AttentionTrace>` to `ResultTrace`
- Integrate attention into `score_candidates`: additive bonus when cues match candidate fields
- Integrate attention into assembly: forward resolved attention into search requests
- Preserve lexical-first baseline when no attention is active
- Explicit empty `AttentionState` suppresses derived bias

## Acceptance Criteria

- [ ] `src/cognition/attention.rs` exists with all attention types
- [ ] `SearchRequest` accepts optional `AttentionState`
- [ ] `WorkingMemoryRequest` accepts optional `AttentionState` and has `resolved_attention_state()`
- [ ] `ScoreBreakdown` includes `attention_bonus`
- [ ] `ResultTrace` includes `attention` field
- [ ] Attention bonus is additive and caps at a reasonable maximum
- [ ] No attention state means backward-compatible scoring (all bonuses 0.0, trace null)
- [ ] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, `cargo check --tests` pass
- [ ] Tests cover: no-attention baseline, explicit attention, derived attention, empty-attention suppression, trace output, agent-search forwarding

## Definition of Done

- Production code updated
- Tests added/updated
- Spec compliance verified via trellis-check
- No new schema added (attention is runtime-only)

## Out of Scope

- Baseline persistence across sessions
- EmotionModulator as multiplicative mask vector
- ContextBase with biological/physical dimensions
- Inhibition side-path from self-model
- MetacogModifier side-path adjusting attention parameters
- Dynamic value weight adjustment from attention signals
- New database tables or HTTP/MCP/UI surfaces

## Technical Notes

### Primary files to create/modify

- **Create**: `src/cognition/attention.rs` — type definitions + Delta derivation
- **Modify**: `src/search/mod.rs` — `SearchRequest.with_attention_state()`
- **Modify**: `src/search/score.rs` — `ScoreBreakdown`, `score_candidates()` integration
- **Modify**: `src/search/rerank.rs` — `ResultTrace`, trace output
- **Modify**: `src/cognition/assembly.rs` — `WorkingMemoryRequest` attention field + `resolved_attention_state()`
- **Modify**: `src/agent/orchestration.rs` — forward resolved attention
- **Tests**: `tests/attention_state.rs`, `tests/working_memory_assembly.rs`, `tests/agent_search.rs`, `tests/retrieval_cli.rs`

### Existing spec to follow

- `.trellis/spec/backend/cognition-retrieval-contracts.md` — "Attention State Retrieval Bias" scenario
- All signatures, contracts, validation matrix, and test requirements from that spec

### Key constraints

- Attention must not create a parallel retrieval path outside `SearchService`
- Attention must not replace lexical explanation
- `attention_bonus` contributes to `final_score` but lexical scoring and citations remain primary
- When no attention bias is active: `attention_bonus == 0.0` and `trace.attention == null`

## Decision (ADR-lite)

**Context**: The theory requires attention to influence what enters the foreground, but the project requires lexical-first explainability and backward-compatible retrieval.

**Decision**: Implement attention as an additive, deterministic bonus inside Rust-side scoring/rerank, with an explicit trace, following the spec already written in `cognition-retrieval-contracts.md`.

**Consequences**:
- Retrieval stays explainable
- Lexical citations and source metadata remain the primary explanation surface
- Future phases can add Baseline persistence, EmotionModulator masks, and self-model Inhibition side-paths
