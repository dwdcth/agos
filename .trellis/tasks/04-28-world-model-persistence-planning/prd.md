# World-Model Persistence Planning

## Goal

Implement the first world-model persistence track step after completed self-model persistence, with world-model persistence going before skill-memory persistence.

This first step must rebuild the explicit internal world-model foundation before adding durable persistence mechanics. The current codebase no longer exposes `src/cognition/world_model.rs`, while the backend spec still requires explicit world-model projection. Durable world-model persistence should not bind directly to assembler-local fragment stitching.

## What I already know

- Self-model persistence has now completed:
  - ledger-first read model
  - lifecycle core
  - snapshot / compaction
  - governance / conflict review
- The user has already chosen ordering:
  1. world-model persistence
  2. skill-memory persistence
- Current world-model state is weaker than the earlier self-model line:
  - the theory and code-spec for world-model projection exist
  - but the current worktree does not expose a dedicated `src/cognition/world_model.rs` module
  - the active code still relies on working-memory `world_fragments` assembly rather than a durable world-model persistence layer
- This means the next persistence phase needs to start by rebuilding an explicit internal world-model seam before durable persistence behavior.

## Assumptions

- The next milestone remains single-threaded, just like the self-model line did.
- The first world-model persistence phase stays internal-only.
- Persistence mechanics come after explicit world-model projection is restored.

## Open Questions

- None for this phase.

## Requirements (evolving)

- World-model persistence goes before skill-memory persistence.
- Scope should remain world-model-only.
- The first phase should be internal-only.
- The first phase should be narrow enough to implement safely.
- Reintroduce a dedicated `src/cognition/world_model.rs` module with explicit projection types matching `.trellis/spec/backend/world-model-contracts.md`.
- Working-memory assembly must build `present.world_fragments` through the explicit world-model projection path.
- Preserve the outward compatibility contract: `WorkingMemory.present.world_fragments` remains `Vec<EvidenceFragment>`.
- Preserve all existing citation, provenance, truth context, DSL, trace, score, and attention metadata through projection.
- Do not introduce world-model persistence tables, snapshots, compaction, MCP/HTTP surfaces, or skill-memory persistence in this phase.

## Acceptance Criteria (evolving)

- [x] Persistence ordering is chosen: world-model first, skill-memory second.
- [x] First world-model phase boundary is chosen: rebuild explicit internal world-model projection first.
- [x] `src/cognition/world_model.rs` exists and is exported from `src/cognition/mod.rs`.
- [x] Working-memory assembly routes fragment materialization through `ProjectedWorldModel` / `WorldFragmentProjection` or equivalent explicit types.
- [x] Existing downstream consumers keep receiving compatible `EvidenceFragment` world fragments.
- [x] Tests cover projection metadata preservation and assembly compatibility.
- [x] `cargo test --quiet` passes, or any failure is documented as pre-existing/environmental with focused reruns.

## Completion Notes

- Implemented explicit internal world-model projection in `src/cognition/world_model.rs`.
- `WorkingMemoryAssembler` now materializes world fragments through `WorldFragmentProjection` and `ProjectedWorldModel`.
- `WorkingMemory.present.world_fragments` remains outward-compatible as `Vec<EvidenceFragment>`.
- Focused verification passed:
  - `rtk cargo test --quiet --test world_model_projection`
  - `rtk cargo test --quiet --test working_memory_assembly`
  - `rtk cargo test --quiet --test agent_search`
  - `rtk cargo test --quiet`
- Independent Trellis check additionally passed `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, `cargo check --tests`, and relevant integration suites.

## Decision (ADR-lite)

**Context**: The next persistence track is world-model persistence, but the current worktree does not expose a dedicated explicit world-model module or an outward inspection surface.

**Decision**: Keep the first world-model persistence phase internal-only and start by restoring the explicit world-model projection foundation before durable persistence mechanics.

**Consequences**:
- The next phase should focus on internal cognition structure and persistence substrate, not developer/operator presentation.
- If inspection is still needed later, it should come after the world-model persistence seam is stable.
- The immediate implementation target is the explicit projection foundation, not new storage tables.

## Out of Scope

- Skill-memory persistence in the same phase
- New MCP / HTTP surfaces by default
- World-model snapshot/compaction schema
- Long-horizon prediction or simulation engine

## Technical Notes

- Existing references:
  - `doc/0415-世界模型.md`
  - `.trellis/spec/backend/world-model-contracts.md`
- Current code appears to need a fresh explicit world-model implementation seam before persistence can go deep.
- The archived Phase 13 PRD is a close implementation reference:
  - `.trellis/tasks/archive/2026-04/04-26-v12-phase-13-world-model-foundation/prd.md`
