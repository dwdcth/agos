# Next Milestone After v1.2

## Goal

Decide the next implementation milestone after the completed v1.2 cognition explicitization track, so the next coding pass has a single clear product/architecture objective instead of continuing ad hoc.

## What I already know

- v1.0 and v1.1 are the only milestones reflected in `.planning/PROJECT.md`, `.planning/ROADMAP.md`, and `.planning/MILESTONES.md`.
- In code, a full v1.2-style cognition-core explicitization pass has now been completed and archived through five tasks:
  - Phase 11 attention state
  - Phase 12 self-model foundation
  - Phase 13 world-model foundation
  - Phase 14 skill-memory foundation
  - Phase 15 cognitive-loop integration
- The repo now has explicit seams for:
  - `src/cognition/attention.rs`
  - `src/cognition/self_model.rs`
  - `src/cognition/world_model.rs`
  - `src/cognition/skill_memory.rs`
- Backend code-specs were also added for these seams under `.trellis/spec/backend/`.
- The project still has open strategic directions already mentioned in `.planning/PROJECT.md`:
  - richer embedding lifecycle / rebuild tooling
  - MCP / HTTP interfaces
  - governance / quality hardening
- The most obvious new implementation path suggested by the current codebase is to move from cognition foundations to durable long-term model/persistence flows:
  - persistent self-model
  - persistent world-model projections or richer truth-backed views
  - long-cycle skill promotion / extraction

## Assumptions

- The next milestone should stay single-threaded in scope, not mix interfaces, tooling, and cognition persistence together.
- Since v1.2 already strengthened the cognition-core structure, the most natural follow-up is either:
  - long-term memory/persistence and write-back productization, or
  - external interface / operator tooling expansion.

## Open Questions

- None for milestone direction. The next step is to turn this scoped milestone choice into an implementation task.

## Candidate Directions

### Option A: Persistent Cognitive Models (selected)

- Turn self/world/skill foundations into longer-lived, durable model layers.
- Focus areas:
  - self-model persistence and lifecycle
  - world-model projection durability / richer truth-backed model views
  - long-cycle skill extraction feeding explicit skill-memory inputs
- Why now:
  - follows directly from the completed v1.2 seams
  - deepens the core cognition value before adding more surfaces

### Option B: Interface Expansion

- Build MCP and/or HTTP surfaces over the now more explicit cognition core.
- Focus areas:
  - stable tool/API contracts
  - remote invocation / integration ergonomics
  - operator-facing service surface
- Why now:
  - exposes the current system sooner
  - useful if the immediate goal is external integration rather than deeper cognition

### Option C: Retrieval / Embedding Operations

- Focus on embedding lifecycle, rebuild tooling, diagnostics, and operational hardening.
- Focus areas:
  - re-embedding flows
  - index rebuild / reconciliation
  - richer operator tooling and lifecycle safety
- Why now:
  - closes a known open thread from `.planning/PROJECT.md`
  - less architectural than A, less product-surface than B

## Decision (ADR-lite)

**Context**: The codebase has already completed an implicit v1.2 cognition explicitization pass, but planning artifacts still only recognize v1.0 and v1.1. The next milestone needs to choose between deeper cognition persistence, interface expansion, or retrieval/embedding operations.

**Decision**: Prioritize persistent cognitive models as the next milestone direction.

**Consequences**:
- The next milestone should build on the new explicit seams rather than pivoting immediately to MCP/HTTP or operator tooling.
- The first implementation phase should focus on self-model persistence instead of world-model or skill-memory persistence.
- The concrete persistence strategy is ledger-first read model, reusing `local_adaptation_entries` as the initial durable substrate.
- The first implementation phase should include both the durable read-model projection and write-side lifecycle rules, rather than stopping at read-only projection.
- The chosen lifecycle scope is the heaviest option: read model, write-side lifecycle, compaction/snapshot, and stronger governance/conﬂict handling.
- The milestone is strictly self-model-only; world-model and skill-memory persistence remain separate later work.
- The first concrete phase should center on read model + lifecycle core, not compaction-first or governance-first.
- The first concrete phase is internal-only; it does not introduce a new operator/developer inspection surface in the same phase.

## Requirements (evolving)

- Persistent cognitive models is the chosen next milestone direction.
- Self-model persistence is the chosen first implementation sub-track.
- Ledger-first read model is the chosen persistence strategy.
- The first phase includes read-model projection plus write-side lifecycle behavior.
- The first phase uses the heavy-closure variant rather than the minimal or medium variant.
- The milestone remains strictly self-model-only and does not include world-model or skill-memory persistence.
- The first concrete phase is `read model + lifecycle core`.
- The first concrete phase is internal-only, with no new inspection surface.
- Keep the next milestone single-threaded.
- Update planning artifacts to reflect the real post-v1.2 state before implementation starts.
- Choose one concrete self-model phase inside the ledger-first strategy.

## Acceptance Criteria (evolving)

- [x] The next milestone direction is chosen.
- [x] The first persistence/write-back phase is chosen.
- [x] The concrete self-model persistence strategy is chosen.
- [x] The first ledger-first self-model phase is chosen.
- [x] The write-side lifecycle subset is chosen.
- [x] The milestone is confirmed self-model-only.
- [x] The first concrete phase is `read model + lifecycle core`.
- [x] The first concrete phase is internal-only.
- [ ] Scope boundaries for that milestone are explicit.
- [ ] Follow-on coding can be broken into phases/tasks without mixing unrelated themes.

## Out of Scope

- Implementing all three next-step directions in one milestone.
- Retroactively rewriting all old planning archives.

## Technical Notes

- Current planning docs are stale relative to code and archived Trellis tasks.
- Archived v1.2-style tasks live under `.trellis/tasks/archive/2026-04/04-26-*` and `04-27-v12-phase-15-cognitive-loop-integration`.
- Current self-model inputs already exist in code:
  - explicit `ProjectedSelfModel` seam in `src/cognition/self_model.rs`
  - persisted `local_adaptation_entries` in `src/memory/repository.rs`
  - short-cycle rumination writes into `LocalAdaptationTargetKind::{SelfState,RiskBoundary,PrivateT3}`
- Ledger-first implication:
  - first implementation can build a durable self-model read path without introducing a new schema
  - but this phase should also define write-side lifecycle rules over that ledger substrate
- Final scoped phase shape:
  - target: self-model persistence only
  - mode: ledger-first read model
  - scope: read model + lifecycle core
  - surface: internal-only
