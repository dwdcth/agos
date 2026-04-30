# Self-Model Persistence Compaction Snapshot

## Goal

Add compaction and snapshot support to the ledger-first self-model persistence layer so the durable substrate can be reduced, recovered, and read efficiently without changing the outward self-model projection contracts or widening scope beyond self-model.

## Requirements

- Stay strictly self-model-only.
- Build on the existing `SelfModelReadModel` and `local_adaptation_entries` substrate.
- Add internal compaction/snapshot behavior for durable self-model state.
- Preserve the current lifecycle-core semantics:
  - request overlays outrank persisted entries
  - same-key resolution remains deterministic
  - inactive tombstones remain suppressive
  - read order remains deterministic
- Keep the phase internal-only:
  - no new CLI/inspect surface
  - no MCP/HTTP/API addition
- Do not expand into governance/conflict review yet.

## Acceptance Criteria

- [ ] There is an explicit compaction/snapshot seam for persisted self-model state.
- [ ] Read-model reconstruction can use compacted/snapshotted state without breaking current self-model projection contracts.
- [ ] Current lifecycle-core rules remain true after compaction/snapshot application.
- [ ] No world-model or skill-memory persistence is introduced.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Out of Scope

- Governance/conflict review
- New schema for world or skill persistence
- New external inspection surface

## Technical Notes

- Existing substrate:
  - `src/cognition/self_model.rs`
  - `src/memory/repository.rs`
  - `src/cognition/assembly.rs`
- This phase should prefer internal storage/read-path stabilization, not product surfaces.
