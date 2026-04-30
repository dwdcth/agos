# Skill-Memory Active Read Model Dedupe

## Goal

Add an explicit active read model for persisted `Consumed` skill-template candidates so runtime loading resolves duplicate `template_id` values deterministically instead of surfacing every consumed row directly.

This phase remains internal-only and builds on top of the existing consumed-only runtime activation path.

## What I Already Know

- Runtime activation already loads only `Consumed` `skill_template` candidates for a `subject_ref`.
- Lifecycle transitions are now typed and lattice-restricted.
- Current runtime loading reconstructs all consumed candidates directly.
- There is no explicit dedupe/read-model rule yet for repeated consumed candidates with the same `template_id`.
- Without a read model, repeated consumes of the same logical template may produce duplicate runtime templates and unstable downstream branch sets.

## Assumptions

- Logical identity for active skill templates should be `template_id`.
- For the same `subject_ref` + `template_id`, latest consumed candidate should win.
- Tie-breaker should be deterministic; prefer:
  - later `updated_at`
  - then later `candidate_id`
- Explicit request-provided `skill_templates` still outrank persisted active templates at merge time.

## Requirements

- Add a typed active read model for persisted consumed skill templates.
- Aggregate by `template_id` within `subject_ref`.
- Resolve duplicates by:
  - later `updated_at`
  - then later `candidate_id`
- Runtime assembly should load from this read model instead of the raw consumed candidate list.
- Preserve the existing single projection path:
  - active read model -> `SkillMemoryTemplate`
  - `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Preserve explicit request template precedence after persisted aggregation.
- Add focused tests for:
  - duplicate consumed candidates same `template_id`
  - deterministic winner selection
  - merge with explicit request templates
  - downstream branch compatibility
- Do not add new schema or external surfaces.

## Acceptance Criteria

- [x] Persisted consumed skill-template candidates are aggregated into an active read model by `template_id`.
- [x] Latest-wins resolution is deterministic by `updated_at`, then `candidate_id`.
- [x] Runtime activation no longer duplicates the same logical persisted template.
- [x] Explicit request templates still remain higher-precedence.
- [x] Focused tests cover duplicate resolution and merge behavior.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Added an explicit `ActiveSkillTemplateReadModel` over persisted consumed skill-template candidates.
- The active read model groups by `template_id` within a subject-scoped consumed candidate set.
- Winner selection is deterministic:
  - later `updated_at`
  - then later `candidate_id`
- Runtime loading now reconstructs templates from the active read model instead of surfacing every consumed row directly.
- Explicit request-provided templates still keep higher precedence at merge time.
- The runtime path remains single-path and unchanged after merge:
  - `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Added focused coverage for:
  - duplicate consumed candidate collapse
  - `updated_at`-first winner selection
  - `candidate_id` tie-break behavior
  - explicit-template precedence
  - downstream `ActionBranch` compatibility
- No schema or external surfaces were added in this phase.

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new schema added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- New statuses or lifecycle rules.
- UI/CLI review workflow.
- New tables or snapshots.

## Technical Notes

- Primary files likely affected:
  - `src/cognition/skill_memory.rs`
  - `src/memory/repository.rs`
  - `src/cognition/assembly.rs`
  - `tests/memory_repository_store.rs`
  - `tests/skill_memory_projection.rs`
  - `tests/working_memory_assembly.rs`

## ADR-lite

**Context**: The runtime bridge now activates consumed skill-template candidates, but it still lacks an explicit active read model for duplicate logical templates.

**Decision**: Aggregate consumed candidates by `template_id` into a deterministic active read model before runtime merge.

**Consequences**:
- Runtime skill projection becomes stable under repeated consume events.
- The project gets a clearer seam between durable candidate history and active skill set.
