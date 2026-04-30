# Self-Model Persistence Governance Conflict Review

## Goal

Implement self-model-only governance and conflict review on top of the existing ledger + snapshot + read-model substrate, so conflicting or high-risk self-model writes stop being resolved only by mechanical precedence rules and instead pass through explicit review/gating semantics.

## Requirements

- Stay strictly self-model-only.
- Build on the current durable substrate:
  - `local_adaptation_entries`
  - `self_model_snapshots`
  - `SelfModelReadModel`
- Add governance/conflict-review behavior for self-model persistence, including at least:
  - conflict detection for same logical key writes that disagree materially
  - explicit resolution state for unresolved vs accepted vs rejected outcomes
  - fail-closed behavior when conflict review is required but unresolved
  - deterministic read behavior when governance metadata is present
- Keep the phase internal-only:
  - no new CLI/inspect surface
  - no MCP/HTTP/API addition
- Preserve current outward cognition contracts:
  - `ProjectedSelfModel`
  - `SelfStateSnapshot`
  - ordinary retrieval / world-model / skill-memory seams

## Acceptance Criteria

- [ ] There is an explicit governance/conflict-review seam for self-model persistence.
- [ ] Conflicting self-model writes no longer rely only on precedence; they can be detected and represented explicitly.
- [ ] Read-model reconstruction has deterministic behavior for unresolved, accepted, and rejected conflict states.
- [ ] Existing self-model projection contracts remain intact.
- [ ] No world-model or skill-memory persistence is introduced.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Definition of Done

- Production code updated.
- Focused tests added or updated.
- Current outward contracts remain stable.
- No new external inspection/API surface is added in this phase.

## Technical Approach

- Add an internal governance/conflict-review layer over self-model persistence rather than changing outward cognition types first.
- Keep the read-model seam explicit and make it consume reviewed/accepted state deterministically.
- Fail closed where conflict state is unresolved instead of silently picking a winner.
- Avoid changing unrelated cognition seams.

## Out of Scope

- World-model persistence
- Skill-memory persistence
- New external inspection surfaces
- Full human review productization

## Technical Notes

- Existing durable substrate:
  - `src/cognition/self_model.rs`
  - `src/memory/repository.rs`
  - `src/cognition/assembly.rs`
- Existing lifecycle/compaction work should remain valid; this phase adds governance on top of it rather than replacing it.
