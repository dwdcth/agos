# v1.2 Phase 14 Skill Memory Foundation

## Goal

Introduce an explicit skill-memory foundation that upgrades the current ad hoc candidate-action seeding into a first-class cognition seam. This phase should define reusable skill structures and let working-memory candidate actions be generated from them without requiring a full persistence redesign or a full long-cycle learning workflow rewrite.

## Requirements

- Add explicit skill-memory types for reusable procedural templates.
- Represent at least:
  - `Preconditions`
  - `ActionTemplate`
  - `ExpectedOutcome`
  - `Boundaries`
- Keep current outward working-memory and action-branch contracts compatible.
- Allow working-memory assembly or a nearby seam to materialize candidate actions from explicit skill templates.
- Keep scope foundation-only: no new schema, no full long-cycle extraction pipeline redesign, no skill library product surface.

## Acceptance Criteria

- [ ] There is a dedicated skill-memory module with explicit template types.
- [ ] Candidate actions can be generated from the new skill-memory seam without breaking existing manual action seeds.
- [ ] Existing agent-search and working-memory behavior stays compatible.
- [ ] Tests cover skill-template projection into candidate actions.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Out of Scope

- Full skill persistence layer
- Long-cycle rumination redesign
- World-model prediction logic
- MCP / HTTP surfaces

## Technical Notes

- Theory references:
  - `doc/0415-技能记忆.md`
  - `doc/0415-工作记忆.md`
  - `doc/0415-反刍机制.md`
