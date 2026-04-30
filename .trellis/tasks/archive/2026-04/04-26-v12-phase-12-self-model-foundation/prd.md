# v1.2 Phase 12 Self Model Foundation

## Goal

Introduce an explicit self-model foundation that upgrades the current ad hoc `self_state` assembly into a first-class cognition seam. This phase should make self-model structure explicit and project it into working-memory snapshots without breaking retrieval, rumination, or existing output contracts.

## Requirements

- Add explicit self-model types that distinguish stable self knowledge from runtime self state.
- Reuse current inputs that already exist in the system:
  - `task_context`
  - `capability_flags`
  - `readiness_flags`
  - truth-derived self facts
  - local adaptation entries
- Keep working-memory assembly backward compatible at the top level; downstream consumers should still receive `SelfStateSnapshot` inside `WorkingMemory`.
- Introduce a dedicated self-model projection seam so later phases can expand storage and write-back behavior without rewriting working-memory assembly again.
- Preserve the current short-cycle rumination overlay path and re-express it through the explicit self-model seam instead of inventing a parallel path.
- Keep the scope foundation-only: do not add new database schema or long-lived self-model persistence tables in this task.

## Acceptance Criteria

- [ ] There is a dedicated self-model module with explicit types for stable and runtime self-state structure.
- [ ] Working-memory assembly builds `self_state` through the new self-model seam instead of directly stitching together flags and facts.
- [ ] Local adaptation overlays still surface in `self_state` after the refactor.
- [ ] Existing working-memory, rumination, and agent-search behavior stays compatible.
- [ ] Tests cover the new self-model projection path and preserve prior overlay behavior.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Definition of Done

- Production code updated.
- Existing relevant tests remain green and new tests cover the explicit self-model seam.
- No retrieval semantics regress.
- No new schema or migration is introduced in this foundation task.

## Technical Approach

- Add a new cognition self-model module with types for:
  - stable profile signals
  - runtime state signals
  - projected self-model snapshot
- Keep `SelfStateSnapshot` as the working-memory output contract for now, but derive it from the new self-model layer.
- Refactor the current provider seam in `src/cognition/assembly.rs` so the provider returns or internally builds through the explicit self-model types.
- Keep local adaptation entries as a source of runtime self/risk data; do not replace them yet with a new persistence model.

## Decision (ADR-lite)

**Context**: The theory requires self model to be a first-class layer, but current code only has `SelfStateSnapshot` plus flags/facts assembled directly in working memory.

**Decision**: Build an explicit self-model foundation as an internal cognition seam first, while preserving the outward `WorkingMemory -> SelfStateSnapshot` contract.

**Consequences**:
- Later phases can add richer persistence and policy without another broad working-memory rewrite.
- Current tests and consumers stay stable.
- This task does not yet deliver the full long-lived self-model envisioned by the theory.

## Out of Scope

- New SQLite tables or migrations for self-model persistence
- Long-term self-model lifecycle management
- Full attention/self-model feedback policy redesign
- World-model projection work
- Skill-memory extraction work
- MCP / HTTP surfaces

## Technical Notes

- Theory references:
  - `doc/0415-自我模型.md`
  - `doc/0415-工作记忆.md`
  - `doc/0415-反刍机制.md`
- Existing implementation seams likely affected:
  - `src/cognition/working_memory.rs`
  - `src/cognition/assembly.rs`
  - `src/cognition/rumination.rs`
  - `src/memory/repository.rs`
  - tests around working memory and rumination overlays
- Current invariants to preserve:
  - local adaptation overlay still affects self-state
  - working-memory shape remains consumable by agent-search and decision reporting
  - no new storage schema in this phase
