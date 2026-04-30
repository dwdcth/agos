# v1.2 Phase 13 World Model Foundation

## Goal

Introduce an explicit world-model foundation that upgrades the current `world_fragments` path from a flat evidence list into a first-class cognition seam. This phase should make world-model projection explicit while preserving the outward `WorkingMemory.present.world_fragments` contract and avoiding simulation, prediction engines, or storage-schema changes.

## Requirements

- Add explicit world-model types that distinguish:
  - a projected current-world slice
  - the fragment-level evidence that still feeds working memory
- Reuse existing inputs already present in the system:
  - retrieved `SearchResult`s
  - truth projections
  - citation / provenance / temporal metadata
  - optional DSL sidecars
- Keep `WorkingMemory.present.world_fragments` backward compatible for downstream consumers.
- Introduce a dedicated world-model projection seam so later phases can extend toward richer structure and prediction without rewriting working-memory assembly again.
- Keep scope foundation-only: no simulation engine, no new database schema, and no world-state persistence layer in this task.

## Acceptance Criteria

- [ ] There is a dedicated world-model module with explicit projection types.
- [ ] Working-memory assembly builds world-state through the new world-model seam instead of directly constructing only `EvidenceFragment`s inline.
- [ ] `WorkingMemory.present.world_fragments` stays outwardly compatible.
- [ ] Citation, truth-context, provenance, DSL, and temporal correctness remain preserved through the refactor.
- [ ] Existing working-memory, agent-search, and retrieval behavior stays compatible.
- [ ] Tests cover the explicit world-model projection path.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Definition of Done

- Production code updated.
- Existing relevant tests remain green and new tests cover the explicit world-model seam.
- No retrieval semantics regress.
- No schema or migration changes are introduced in this foundation task.

## Technical Approach

- Add a new cognition world-model module with explicit projection types for a current-world slice and its evidence-backed fragments.
- Refactor working-memory assembly so truth projection and fragment materialization pass through the world-model layer.
- Keep `EvidenceFragment` and `PresentFrame.world_fragments` as the outward compatibility contract for now.
- Preserve all existing citation / trace / truth-context data by projecting from the existing retrieved and truth-governed records rather than inventing a new source path.

## Decision (ADR-lite)

**Context**: The theory requires world model to be a first-class cognition layer, but current code builds `world_fragments` directly inside working-memory assembly.

**Decision**: Build an explicit world-model foundation as an internal cognition seam first, while preserving the outward `WorkingMemory.present.world_fragments` contract.

**Consequences**:
- Later phases can extend world modeling without another broad assembler rewrite.
- Current tests and consumers stay stable.
- This task does not yet deliver simulation or predictive branching.

## Out of Scope

- Prediction / simulation engine
- World-state persistence tables
- Self-model redesign
- Skill-memory extraction work
- MCP / HTTP surfaces
- New SQLite schema or migrations

## Technical Notes

- Theory references:
  - `doc/0415-世界模型.md`
  - `doc/0415-工作记忆.md`
  - `doc/0415-真值层.md`
- Existing implementation seams likely affected:
  - `src/cognition/working_memory.rs`
  - `src/cognition/assembly.rs`
  - `src/memory/truth.rs`
  - tests around working memory and agent search
- Current invariants to preserve:
  - `world_fragments` remains consumable by agent-search and decision reporting
  - truth context, citation, provenance, DSL, and trace data survive projection
  - no new storage schema in this phase
