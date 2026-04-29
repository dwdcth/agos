# Skill-Memory Runtime Read Model From Consumed Candidates

## Goal

Bridge durable skill-memory candidates into a conservative runtime read model by loading only `Consumed` `skill_template` candidates from `rumination_candidates` and reconstructing them as active `SkillMemoryTemplate` values for working-memory assembly.

This phase should preserve the internal-only model, keep `Pending` candidates out of runtime projection, and avoid introducing a second activation path outside the existing `WorkingMemoryRequest.skill_templates` seam.

## What I Already Know

- Skill-memory now has:
  - an explicit internal cognition seam
  - durable candidate persistence via `rumination_candidates(skill_template)`
- The durable candidate payload is now structured and versioned.
- Repository helpers already exist for typed skill-template candidate loading.
- Current runtime assembly still only sees:
  - explicit caller-provided `skill_templates`
  - no persisted skill-template auto-load path
- `RuminationCandidateStatus` already includes:
  - `Pending`
  - `Consumed`
  - `Rejected`
  - `Archived`
- The conservative next step is to treat only `Consumed` skill-template candidates as active runtime templates.

## Assumptions

- `Pending` skill-template candidates must not become active runtime templates automatically.
- `Rejected` and `Archived` candidates must remain inactive.
- Runtime loading should be subject-scoped through `subject_ref`.
- Explicit request-provided `skill_templates` should remain additive and higher-trust than repository-loaded runtime templates if both are present.

## Requirements

- Add a narrow runtime read-model helper for persisted skill-memory templates derived from `Consumed` candidates.
- Filter by:
  - `candidate_kind = skill_template`
  - `status = consumed`
  - `subject_ref`
- Reconstruct those candidates into `SkillMemoryTemplate`.
- Integrate the read-model into working-memory assembly only when `subject_ref` is present.
- Preserve explicit caller-provided `skill_templates`; they must remain additive and should not be overwritten by repository-loaded templates.
- Keep projection single-path:
  - repository-loaded active templates still flow through `WorkingMemoryRequest.skill_templates`
  - then through `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Add focused tests for:
  - loading only `Consumed` candidates
  - ignoring `Pending`, `Rejected`, and `Archived`
  - merging persisted runtime templates with explicit request templates
  - preserving downstream branch compatibility
- Do not add new tables, public surfaces, or generic candidate-review UI.

## Acceptance Criteria

- [x] Repository exposes a helper or read-model path for active runtime skill templates based on `Consumed` candidates.
- [x] Working-memory assembly loads persisted runtime skill templates for `subject_ref` without activating `Pending` candidates.
- [x] Explicit request skill templates remain additive and compatible.
- [x] Existing branch materialization remains single-path and backward-compatible.
- [x] Focused tests cover status filtering and runtime merge behavior.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Added a subject-scoped repository helper that loads only `Consumed` `skill_template` candidates.
- Added runtime reconstruction helpers that turn persisted consumed candidates into `SkillMemoryTemplate` values.
- Integrated persisted runtime templates into `WorkingMemoryAssembler` only when `subject_ref` is present.
- Preserved explicit caller-provided `skill_templates` as additive and higher-precedence by `template_id`.
- Kept the single projection path unchanged:
  - `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Preserved conservative activation semantics:
  - `Pending` candidates stay inactive
  - `Rejected` candidates stay inactive
  - `Archived` candidates stay inactive
- Preserved fail-closed behavior for malformed consumed payloads through typed repository/decode errors.
- Added/updated focused coverage for:
  - consumed/status/subject filtering
  - additive merge with explicit templates
  - runtime assembly compatibility
- No new schema or external surfaces were added in this phase.

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new schema added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- Candidate review UI or CLI.
- Automatic promotion from `Pending` to `Consumed`.
- New tables or snapshots.
- Non-subject-scoped global skill-template activation.

## Technical Notes

- Primary files likely affected:
  - `src/memory/repository.rs`
  - `src/cognition/skill_memory.rs`
  - `src/cognition/assembly.rs`
  - `tests/memory_repository_store.rs`
  - `tests/skill_memory_projection.rs`
  - `tests/working_memory_assembly.rs`
- Contracts to preserve:
  - `.trellis/spec/backend/skill-memory-contracts.md`
  - `.trellis/spec/backend/self-model-contracts.md`
  - `.trellis/spec/backend/world-model-contracts.md`

## ADR-lite

**Context**: Durable skill-memory candidates now exist, but the system still lacks a conservative activation bridge from persisted long-cycle outputs into runtime `skill_templates`.

**Decision**: Activate only `Consumed` skill-template candidates into the runtime read model, keyed by `subject_ref`, and merge them additively with explicit request templates.

**Consequences**:
- Runtime skill projection remains explainable and conservative.
- Unreviewed `Pending` candidates stay out of the foreground.
- Later phases can refine candidate-review and activation workflows without changing the branch materialization seam.
