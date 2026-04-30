# Skill-Memory Active Template Tombstone Semantics

## Goal

Prevent older consumed skill-template candidates from reappearing in the active runtime read model when a later candidate with the same `template_id` has been rejected or archived.

This phase should refine the active read model semantics only. It does not add new states or new tables.

## What I Already Know

- The runtime bridge now loads an active read model from consumed skill-template candidates.
- The active read model already dedupes duplicates among consumed rows by `template_id`, `updated_at`, and `candidate_id`.
- The lifecycle bridge now supports:
  - `Pending -> Consumed`
  - `Pending -> Rejected`
  - `Consumed -> Archived`
- Current remaining gap:
  - the active read model ignores non-consumed rows entirely
  - a later `Archived` or `Rejected` row for the same `template_id` can therefore fail to suppress an older consumed row
- That would make “deactivate this template” semantically leaky.

## Assumptions

- Logical identity remains `template_id` within `subject_ref`.
- The active read model should consider the latest row across all relevant statuses for a template, not only consumed rows.
- Only a latest `Consumed` winner should activate.
- A latest `Rejected` or `Archived` winner should suppress activation entirely.

## Requirements

- Refine the active skill-template read model to resolve latest row per `template_id` before filtering for activation.
- Winner selection remains deterministic by:
  - later `updated_at`
  - then later `candidate_id`
- Activation rule:
  - latest row is `Consumed` → active template loads
  - latest row is `Rejected` or `Archived` → template stays inactive
  - `Pending` should also stay inactive
- Preserve explicit request-template precedence after persisted aggregation.
- Preserve the existing single runtime projection path:
  - active read model -> `SkillMemoryTemplate`
  - `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Add focused tests for:
  - later archived row suppresses earlier consumed row
  - later rejected row suppresses earlier consumed row
  - later consumed row still overrides earlier consumed row
  - explicit request templates still merge with higher precedence
- Do not add new schema, new statuses, or external surfaces.

## Acceptance Criteria

- [x] Active read model resolves latest candidate per `template_id` across relevant statuses.
- [x] Later `Archived` or `Rejected` rows suppress earlier consumed rows.
- [x] Later `Consumed` rows still activate normally.
- [x] Explicit request templates remain higher-precedence.
- [x] Focused tests cover tombstone suppression and normal consume overwrite behavior.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Refined the active read model so it resolves the latest candidate per `template_id` before deciding runtime activation.
- Winner selection remains deterministic by:
  - later `updated_at`
  - then later `candidate_id`
- Activation semantics are now:
  - latest `Consumed` winner activates
  - latest `Rejected` winner suppresses activation
  - latest `Archived` winner suppresses activation
  - latest `Pending` winner stays inactive
- This closes the semantic leak where an older consumed template could reappear after a later deactivation row.
- Explicit request templates still retain higher precedence after persisted aggregation.
- The runtime projection path remains unchanged:
  - `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Added/updated focused coverage for:
  - archived tombstone suppression
  - rejected tombstone suppression
  - later consumed overwrite
  - explicit-template merge precedence
  - downstream branch compatibility
- No schema or external surfaces were added in this phase.

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new schema added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- New statuses.
- UI/CLI review workflow.
- New tables or snapshots.

## Technical Notes

- Primary files likely affected:
  - `src/cognition/skill_memory.rs`
  - `src/memory/repository.rs`
  - `tests/skill_memory_projection.rs`
  - `tests/working_memory_assembly.rs`
  - `.trellis/spec/backend/skill-memory-contracts.md`

## ADR-lite

**Context**: The active read model already dedupes consumed rows, but without tombstone semantics a later archived/rejected candidate can fail to suppress an older consumed version of the same logical template.

**Decision**: Make the active read model pick the latest candidate per `template_id` first, then activate only if that latest winner is `Consumed`.

**Consequences**:
- Deactivation becomes semantically stable.
- Runtime activation reflects the latest lifecycle state, not just the latest consumed subset.
