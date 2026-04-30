# World-Model Runtime Read Model From Snapshot

## Goal

Bridge persisted world-model snapshots back into runtime working-memory assembly so a subject-scoped current-world slice can be reconstructed from durable state instead of always requiring fresh live retrieval.

This phase should stay internal-only and preserve the existing outward `WorkingMemory.present.world_fragments` contract.

## What I Already Know

- The world-model line already has:
  - explicit internal projection seam
  - durable snapshot persistence keyed by `subject_ref + world_key`
- Current assembly still uses live retrieval to produce `world_fragments`.
- There is no runtime read-model bridge from `world_model_snapshots` into working-memory assembly yet.
- World-model snapshots already preserve:
  - `record_id`
  - `snippet`
  - `citation`
  - `provenance`
  - `truth_context`
  - `dsl`
  - `trace`
  - `score`
- A conservative next step is to load the persisted `"current"` world slice for a `subject_ref`, while keeping explicit integrated results and ordinary live retrieval precedence where appropriate.

## Assumptions

- Runtime load should be subject-scoped through `subject_ref`.
- The first runtime bridge should target `world_key = "current"`.
- Explicit caller-provided `integrated_results` should remain higher-precedence than snapshot recovery.
- If no snapshot exists, assembly should preserve existing live retrieval behavior.

## Requirements

- Add a runtime read-model helper for loading the persisted current world-model snapshot for a `subject_ref`.
- Reconstruct a `ProjectedWorldModel` from the snapshot and project it back to `EvidenceFragment`s or equivalent world-model seam output.
- Integrate the runtime read model into working-memory assembly conservatively:
  - explicit `integrated_results` remain first-class
  - if no explicit integrated results are provided and a suitable snapshot exists, the assembly may use the persisted current-world slice
  - if no snapshot exists, existing live retrieval behavior remains unchanged
- Preserve the existing outward `WorkingMemory.present.world_fragments` contract.
- Preserve the explicit world-model seam; do not reintroduce assembler-local fragment stitching.
- Add focused tests for:
  - snapshot-backed runtime assembly
  - no-snapshot fallback to live retrieval
  - explicit integrated-results precedence over snapshot load
  - downstream compatibility
- Do not add new tables, HTTP/MCP/UI surfaces, or prediction logic.

## Acceptance Criteria

- [ ] Runtime helper can load the persisted `"current"` world-model snapshot by `subject_ref`.
- [ ] Working-memory assembly can use the snapshot-backed current-world slice without contract drift.
- [ ] Explicit integrated results still outrank snapshot-backed world state.
- [ ] Live retrieval fallback still works when no snapshot exists.
- [ ] Focused tests cover snapshot-backed runtime loading and precedence rules.
- [ ] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new schema added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- New world-model statuses or lifecycle review workflow.
- Prediction/simulation.
- CLI/HTTP/MCP/UI world snapshot inspection.
- Full replacement of live retrieval in all cases.

## Technical Notes

- Primary files likely affected:
  - `src/cognition/world_model.rs`
  - `src/cognition/assembly.rs`
  - `src/memory/repository.rs`
  - `tests/world_model_projection.rs`
  - `tests/working_memory_assembly.rs`
  - `tests/agent_search.rs`
- Contracts to preserve:
  - `.trellis/spec/backend/world-model-contracts.md`
  - `.trellis/spec/backend/cognition-retrieval-contracts.md`
  - `.trellis/spec/backend/self-model-contracts.md`

## ADR-lite

**Context**: World-model snapshots exist, but the runtime path still always reconstructs world state from live retrieval.

**Decision**: Add a conservative runtime read-model bridge for the persisted `"current"` world-model snapshot, while preserving explicit integrated-results precedence and live-retrieval fallback.

**Consequences**:
- The world-model persistence line becomes usable at runtime, not just durable on disk.
- The system preserves explainability and backward-compatible outward contracts.
- Later phases can refine when snapshot-backed state should outrank or merge with fresh retrieval.
