# Self-Model Persistence Follow-up Phases

## Goal

Plan the next two phases after the completed `self-model persistence / ledger-first / read model + lifecycle core / internal-only` task, while keeping the milestone strictly self-model-only.

## What I already know

- The following are already complete in code:
  - explicit attention seam
  - explicit self-model seam
  - explicit world-model seam
  - explicit skill-memory seam
  - cognitive-loop integration across these seams
- The first self-model persistence phase is also complete:
  - ledger-first read model
  - lifecycle core rules
  - internal-only surface
- The user has explicitly confirmed two follow-up needs:
  - `compaction / snapshot`
  - `governance / conflict review`
- The milestone should remain self-model-only and should not mix in world-model or skill-memory persistence.

## Assumptions

- Both follow-up needs belong in the same broader self-model persistence milestone, but not necessarily in the same implementation phase.
- The highest-value remaining planning question is phase order, not whether both are needed.

## Open Questions

- None for ordering. The next step is to turn the chosen first phase into an implementation task.

## Requirements (evolving)

- Keep the next work strictly self-model-only.
- Deliver both:
  - compaction / snapshot
  - governance / conflict review
- Break them into a sequence that keeps phase scope coherent and testable.
- Execute them in this order:
  1. compaction / snapshot
  2. governance / conflict review

## Acceptance Criteria (evolving)

- [x] Confirm that both compaction/snapshot and governance/conflict review are needed.
- [x] Choose the order of those two phases.
- [ ] Keep scope boundaries explicit enough to create the next implementation task.

## Decision (ADR-lite)

**Context**: Both compaction/snapshot and governance/conflict review are required to complete the next stage of self-model persistence, but they do not need to land in the same implementation phase.

**Decision**: Run compaction/snapshot first, then governance/conflict review.

**Consequences**:
- The next implementation task should focus on stabilizing the durable self-model substrate before tightening review/governance policy.
- Governance/conflict review can assume a more stable underlying persisted form once it begins.

## Out of Scope

- World-model persistence
- Skill-memory persistence
- New external inspection / API surfaces

## Technical Notes

- Current self-model persistence substrate is still `local_adaptation_entries`.
- Current self-model read path already has lifecycle core semantics and a preserved explicit seam.
