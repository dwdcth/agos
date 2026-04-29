# Skill-Memory Lifecycle Transition Lattice

## Goal

Tighten the `skill_template` candidate lifecycle bridge with an explicit allowed transition lattice so invalid state changes, such as `Archived -> Consumed`, fail deterministically instead of slipping through the typed helper seam.

This phase remains internal-only and reuses the existing `rumination_candidates` table and status set.

## What I Already Know

- The project now has typed skill-template lifecycle helpers:
  - consume
  - reject
  - archive
- Those helpers already reject:
  - missing candidates
  - wrong candidate kind
  - invalid/legacy payloads before mutation
- Current remaining gap:
  - helpers do not yet enforce a strict allowed transition lattice
- Runtime activation already depends only on `Consumed`.

## Assumptions

- The status set remains unchanged:
  - `Pending`
  - `Consumed`
  - `Rejected`
  - `Archived`
- The first useful lattice is narrow and conservative.

## Requirements

- Define explicit allowed transitions for `skill_template` candidates.
- At minimum enforce:
  - `Pending -> Consumed`
  - `Pending -> Rejected`
  - `Consumed -> Archived`
- Explicitly reject invalid transitions such as:
  - `Archived -> Consumed`
  - `Rejected -> Consumed`
  - `Consumed -> Rejected`
  - `Pending -> Archived` unless you discover a strong existing contract saying otherwise
- Return a typed lifecycle error for invalid transitions.
- Preserve all existing fail-closed boundaries:
  - wrong kind
  - missing candidate
  - invalid/legacy payloads
- Preserve metadata and `updated_at` behavior on valid transitions.
- Add focused tests for valid and invalid transition paths.
- Do not add new states, new tables, or external review surfaces.

## Acceptance Criteria

- [x] A typed transition lattice exists for `skill_template` candidate helpers.
- [x] Invalid transitions fail explicitly without mutating the row.
- [x] Existing valid transitions still succeed and preserve metadata.
- [x] Runtime activation behavior remains unchanged and still depends only on `Consumed`.
- [x] Focused tests cover valid and invalid transition paths.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Added an explicit allowed transition lattice for `skill_template` candidates.
- Allowed transitions are now limited to:
  - `Pending -> Consumed`
  - `Pending -> Rejected`
  - `Consumed -> Archived`
- Disallowed transitions now fail through typed `InvalidTransition` errors.
- Invalid transitions return before any repository mutation, so `status` and `updated_at` remain unchanged.
- Existing fail-closed boundaries were preserved:
  - wrong candidate kind
  - missing candidate
  - invalid or legacy payload before mutation
- Runtime activation semantics remain unchanged and still depend only on `Consumed`.
- Added/updated focused tests for:
  - valid transitions
  - invalid transitions without row mutation
  - continued runtime activation semantics
- No new schema or external review surface was added in this phase.

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new schema added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- New statuses.
- UI/CLI review workflow.
- Automatic review policy.

## Technical Notes

- Primary files likely affected:
  - `src/memory/repository.rs`
  - `tests/memory_repository_store.rs`
  - `.trellis/spec/backend/skill-memory-contracts.md`

## ADR-lite

**Context**: The skill-memory lifecycle bridge is now typed and fail-closed on payload/type boundaries, but it still lacks an explicit transition lattice.

**Decision**: Add a narrow, conservative transition lattice for `skill_template` candidates and reject invalid status moves explicitly.

**Consequences**:
- Runtime activation remains aligned with lifecycle semantics.
- Future review workflows can build on a stable status machine instead of ad hoc helper behavior.
