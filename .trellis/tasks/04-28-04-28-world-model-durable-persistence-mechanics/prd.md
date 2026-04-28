# World-Model Durable Persistence Mechanics

## Goal

Add the first durable persistence mechanics for the explicit internal world-model projection restored in the previous task.

This phase should make `ProjectedWorldModel` persistable and reloadable through SQLite-backed repository APIs, while keeping the first durable step internal-only. It must not add prediction/simulation, external inspection surfaces, skill-memory persistence, or change the outward `WorkingMemory.present.world_fragments: Vec<EvidenceFragment>` contract.

## What I Already Know

- The user chose ordering:
  1. world-model persistence
  2. skill-memory persistence
- The previous step restored the explicit internal world-model projection:
  - `src/cognition/world_model.rs`
  - `ProjectedWorldModel`
  - `CurrentWorldSlice`
  - `WorldFragmentProjection`
  - assembly routes world fragments through this projection path
- Self-model persistence already provides a useful local pattern:
  - migration-backed snapshot table
  - repository DTOs and load/replace APIs
  - cognition-layer conversion helpers
  - focused repository and schema tests
- Current world-model code has no durable table or repository read/write seam.
- Search trace/score/citation types currently serialize for output; durable JSON round-trip may require either `Deserialize` derives on safe shared types or a dedicated persisted DTO.

## Assumptions

- This phase is the durable storage substrate, not a behavior-heavy recall policy phase.
- The first persisted world-model unit should be a snapshot/read-model of projected world fragments, not a raw duplicate of all memory records.
- Storage should preserve evidence-backed explainability metadata rather than storing only record IDs.
- Assembly may remain driven by fresh retrieval in this phase unless a narrow, explicit internal read/write hook is safe to add.

## Requirements

- Add a SQLite migration for world-model snapshots.
- Register the migration in `src/core/migrations.rs`.
- Add repository DTOs and APIs for world-model snapshot persistence.
- Add cognition-layer conversion between `ProjectedWorldModel` and the persisted snapshot DTO.
- Persist enough metadata to reconstruct a `ProjectedWorldModel` without losing:
  - `record_id`
  - `snippet`
  - `citation`
  - `provenance`
  - `truth_context`
  - `dsl`
  - `trace`
  - `score`
- Keep the snapshot keyed by a stable internal identity. Minimum expected key:
  - `subject_ref`
  - `world_key` or equivalent scope key for a current-world slice
  - `snapshot_id`
- Keep APIs internal to Rust modules; do not add CLI, HTTP, MCP, or UI surfaces.
- Preserve the existing outward working-memory shape.
- Do not mix skill-memory persistence into this phase.
- Do not introduce prediction/simulation mechanics in this phase.
- Add focused schema, repository round-trip, and cognition reconstruction tests.

## Acceptance Criteria

- [x] A new migration creates a world-model snapshot table with a uniqueness boundary for `subject_ref` + world scope key.
- [x] Migration registration and foundation schema tests include the new table and important columns/indexes.
- [x] `MemoryRepository` can write and load a world-model snapshot.
- [x] Snapshot JSON round-trips projected world fragments without metadata loss.
- [x] `ProjectedWorldModel` can convert to a persisted snapshot and reconstruct from one.
- [x] Existing world-model projection tests still pass.
- [x] Existing working-memory, agent-search, retrieval CLI, and repository tests pass.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Added `migrations/0010_world_model_snapshots.sql` and registered it in `src/core/migrations.rs`.
- Added dedicated persisted world-model snapshot DTOs and repository APIs in `src/memory/repository.rs`.
- Added `ProjectedWorldModel::to_snapshot(...)` / `from_snapshot(...)` plus fragment-level persisted conversion in `src/cognition/world_model.rs`.
- Preserved snapshot metadata for:
  - `record_id`
  - `snippet`
  - `citation`
  - `provenance`
  - `truth_context`
  - `dsl`
  - `trace`
  - `score`
- Kept the runtime behavior intentionally narrow:
  - no CLI/HTTP/MCP/UI surface
  - no skill-memory persistence
  - no prediction/simulation
  - no switch from live retrieval assembly to snapshot-backed assembly in this phase
- Verification passed in both implementation and independent Trellis check:
  - `cargo fmt --check`
  - `cargo check --tests`
  - `cargo clippy --tests -- -D warnings`
  - `cargo test --test foundation_schema --test memory_repository_store --test world_model_projection --test working_memory_assembly --test agent_search --test retrieval_cli`

## Definition Of Done

- Production code updated.
- Tests added/updated for schema, repository, and cognition snapshot reconstruction.
- No external API surface added.
- No skill-memory persistence added.
- No prediction/simulation engine added.
- Trellis check passes after implementation.

## Out Of Scope

- Skill-memory persistence.
- External inspection, CLI commands, HTTP, MCP, or UI.
- Prediction, simulation, transition functions, or counterfactual branching.
- Replacing lexical-first retrieval.
- Changing `WorkingMemory.present.world_fragments` away from `Vec<EvidenceFragment>`.
- Persisting raw search results as the primary domain object instead of world-model projection entries.

## Technical Notes

- Primary implementation files likely affected:
  - `src/cognition/world_model.rs`
  - `src/memory/repository.rs`
  - `src/core/migrations.rs`
  - `migrations/0010_world_model_snapshots.sql`
  - `tests/foundation_schema.rs`
  - `tests/memory_repository_store.rs`
  - `tests/world_model_projection.rs`
- Contracts to preserve:
  - `.trellis/spec/backend/world-model-contracts.md`
  - `.trellis/spec/backend/cognition-retrieval-contracts.md`
  - `.trellis/spec/backend/database-guidelines.md`
- Self-model persistence reference pattern:
  - `migrations/0009_self_model_snapshots.sql`
  - `src/cognition/self_model.rs`
  - `src/memory/repository.rs`
  - `tests/memory_repository_store.rs`
  - `tests/foundation_schema.rs`

## ADR-lite

**Context**: World-model projection now exists again, but it is still transient. The next persistence step needs durable mechanics without expanding into prediction or public inspection too early.

**Decision**: Add an internal snapshot/read-model persistence substrate for `ProjectedWorldModel` first. Store projected world fragments as evidence-backed snapshot entries so the model can be reloaded with citation, truth, DSL, trace, and score metadata intact.

**Consequences**:
- Later phases can decide how assembly uses persisted world-model snapshots.
- The project gains a durable substrate without changing downstream working-memory consumers.
- The implementation mirrors the successful self-model snapshot pattern while keeping world-model semantics separate.
