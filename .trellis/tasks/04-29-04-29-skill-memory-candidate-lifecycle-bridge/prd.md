# Skill-Memory Candidate Lifecycle Bridge

## Goal

Add a narrow typed lifecycle bridge for persisted `skill_template` candidates so the system can explicitly move them through review states such as `Pending -> Consumed/Rejected/Archived` without relying on ad hoc generic `RuminationCandidate` mutations.

This phase should stay internal-only and reuse the existing `rumination_candidates` table and status fields.

## What I Already Know

- Skill-memory now has:
  - explicit internal skill-memory seam
  - durable candidate persistence via `rumination_candidates(skill_template)`
  - runtime activation of `Consumed` candidates only
- Runtime activation already depends on `Consumed` status.
- The repository currently exposes only generic `update_rumination_candidate(...)`, not a typed helper focused on `skill_template` lifecycle.
- Without a typed bridge, upstream code would need to hand-edit generic `RuminationCandidate` rows, which is error-prone and weakens the seam.

## Assumptions

- Lifecycle changes should remain on the existing status lattice:
  - `Pending`
  - `Consumed`
  - `Rejected`
  - `Archived`
- Only `skill_template` candidates should flow through this new typed helper.
- The first phase can be repository/cognition-layer only; no UI or CLI review workflow is needed.

## Requirements

- Add typed helpers for `skill_template` candidate lifecycle transitions on top of `rumination_candidates`.
- Preserve generic repository support, but provide a narrower API for:
  - consume a skill-template candidate
  - reject a skill-template candidate
  - archive a skill-template candidate
- Enforce candidate kind at the helper seam so non-`skill_template` rows are rejected.
- Update timestamps consistently during lifecycle transitions.
- Preserve payload, evidence refs, and source lineage during status transitions.
- Add focused tests for:
  - successful `Pending -> Consumed`
  - successful `Pending -> Rejected`
  - successful `Consumed -> Archived`
  - rejection of wrong candidate kind or missing candidate
  - runtime activation observing `Consumed` after transition
- Do not add new tables or external review surfaces.

## Acceptance Criteria

- [x] Typed repository or cognition helpers exist for `skill_template` lifecycle transitions.
- [x] Helpers reject non-`skill_template` candidates explicitly.
- [x] Status transitions preserve payload/evidence/source metadata.
- [x] Runtime read model continues to activate only `Consumed` candidates after transition.
- [x] Focused tests cover transition behavior and error boundaries.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Added typed lifecycle helpers on top of the existing `rumination_candidates` substrate for:
  - consume a skill-template candidate
  - reject a skill-template candidate
  - archive a skill-template candidate
- Helpers explicitly reject:
  - missing candidates
  - wrong candidate kind
- Successful transitions preserve:
  - payload
  - evidence refs
  - source queue lineage
  - subject ref
  - governance ref
  - `created_at`
- Transitions only mutate:
  - `status`
  - `updated_at`
- Locked the lifecycle bridge to fail closed:
  - legacy or invalid `skill_template` payloads are validated before persistence mutation
  - invalid payload rows do not partially transition state
- Added/updated focused coverage for:
  - `Pending -> Consumed`
  - `Pending -> Rejected`
  - `Consumed -> Archived`
  - wrong-kind rejection
  - missing-candidate rejection
  - invalid-payload no-mutation regression
  - runtime activation observing `Consumed` only after transition
- No new schema or external review surface was added in this phase.

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new schema added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- UI/CLI/MCP review workflow.
- New candidate tables or queues.
- Automatic review policy or heuristics.

## Technical Notes

- Primary files likely affected:
  - `src/memory/repository.rs`
  - `src/cognition/skill_memory.rs`
  - `tests/memory_repository_store.rs`
  - `tests/working_memory_assembly.rs`
- Contracts to preserve:
  - `.trellis/spec/backend/skill-memory-contracts.md`
  - `.trellis/spec/backend/quality-guidelines.md`

## ADR-lite

**Context**: Skill-template candidates are now durable and `Consumed` candidates already activate at runtime, but the project still lacks a typed lifecycle seam for moving candidates between review states.

**Decision**: Add a narrow typed lifecycle bridge for `skill_template` candidates on top of the existing `rumination_candidates` substrate instead of letting callers mutate generic candidate rows ad hoc.

**Consequences**:
- Candidate state changes become explicit and safer.
- Runtime activation stays aligned with lifecycle semantics.
- Later UI or review workflows can call the same typed seam.
