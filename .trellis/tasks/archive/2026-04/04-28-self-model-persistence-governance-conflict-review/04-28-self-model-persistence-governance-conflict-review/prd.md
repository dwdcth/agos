# Self-Model Persistence Governance Conflict Review

## Goal

Implement self-model-only governance and conflict review on top of the existing ledger + snapshot + read-model substrate, so conflicting self-model writes are governed explicitly instead of relying only on precedence rules.

## What I already know

- The self-model persistence line already has:
  - explicit `ProjectedSelfModel`
  - ledger-first `SelfModelReadModel`
  - lifecycle core rules
  - snapshot + tail-ledger reconstruction
- The milestone remains self-model-only:
  - no world-model persistence
  - no skill-memory persistence
  - no new external API surface by default
- Current repository substrate already includes:
  - `local_adaptation_entries`
  - `self_model_snapshots`
- Current self-model read path is deterministic and fail-closed for inactive tombstones, but it does not yet provide an explicit governance/review layer for materially conflicting writes.

## Assumptions

- This phase should build on top of the current persistence substrate, not redesign it.
- This phase stays internal-only and does not add a new developer/operator inspection surface.

## Open Questions

- None for phase scoping. The next step is to execute the internal-only governance/conflict review phase.

## Requirements (evolving)

- Stay strictly self-model-only.
- Add explicit governance/conflict review over the current ledger + snapshot + read-model substrate.
- Preserve outward `ProjectedSelfModel -> SelfStateSnapshot` compatibility.
- Keep the phase narrow enough to avoid mixing in world-model or skill-memory persistence.
- Keep the phase internal-only, with no new inspection/API surface.

## Acceptance Criteria (evolving)

- [x] Governance/conflict review behavior is scoped.
- [x] Phase boundary is explicit enough to start implementation safely.

## Decision (ADR-lite)

**Context**: The self-model persistence line still needed explicit governance/conflict review, but adding an inspection surface in the same phase would widen scope and mix policy work with operator tooling.

**Decision**: Keep this phase internal-only. Implement governance/conflict review semantics over the existing ledger + snapshot + read-model substrate without adding a new inspection or API surface.

**Consequences**:
- The phase can focus on conflict detection, resolution state, and fail-closed read behavior.
- Operator/developer inspection remains a separate later phase if still needed.

## Out of Scope

- World-model persistence
- Skill-memory persistence
- MCP / HTTP surface

## Technical Notes

- Existing relevant contracts:
  - `.trellis/spec/backend/self-model-contracts.md`
- Existing relevant implementation:
  - `src/cognition/self_model.rs`
  - `src/memory/repository.rs`
  - `src/cognition/assembly.rs`
