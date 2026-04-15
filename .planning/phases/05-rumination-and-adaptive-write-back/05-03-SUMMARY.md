---
phase: 05-rumination-and-adaptive-write-back
plan: 03
subsystem: cognition
tags: [rust, rumination, lpq, governance, sqlite, testing]
requires:
  - phase: 05-rumination-and-adaptive-write-back
    provides: explicit LPQ queue scheduling and candidate storage from 05-01 plus local-only write-back boundaries from 05-02
  - phase: 03-truth-layer-governance
    provides: pending promotion reviews, ontology candidates, and canonical governance service seams
  - phase: 04-working-memory-and-agent-search
    provides: bounded AgentSearchReport inputs with citations and decision traces
provides:
  - unified LPQ candidate generation for skill, promotion, and value-adjustment outputs
  - governance bridging that routes shared-truth-facing LPQ work into Phase 3 pending queues
  - regression coverage proving long-cycle outputs stay candidate-first and never auto-approve
affects: [phase-5, rumination, governance, cognition, learning]
tech-stack:
  added: []
  patterns:
    - long-cycle synthesis consumes persisted report/citation payloads instead of re-running retrieval
    - promotion candidates reuse Phase 3 governance seams while skill/value candidates remain inside rumination storage
    - canonical governance refs are persisted back onto the same candidate contract without creating a second proposal system
key-files:
  created:
    - tests/rumination_governance_integration.rs
  modified:
    - src/cognition/rumination.rs
    - src/memory/repository.rs
key-decisions:
  - "Kept all three LPQ outputs on one `RuminationCandidate` contract and stored governance ref IDs inside the durable candidate payload instead of introducing a separate long-cycle schema."
  - "Classified promotion candidates by source truth layer: T3 evidence creates pending promotion reviews with attached evidence, while T2 evidence creates pending ontology candidates."
  - "Completed LPQ items only after candidate persistence and any required governance bridge succeeded, so long-cycle work stays auditable and non-self-approving."
patterns-established:
  - "Long-cycle drains are opportunity-driven like SPQ drains: claim one LPQ item, synthesize candidates, bridge governance if needed, then complete the queue item."
  - "Shared-truth-facing LPQ work reuses `TruthGovernanceService` as the only canonical proposal lifecycle."
requirements-completed: [LRN-03]
duration: 8min
completed: 2026-04-16
---

# Phase 05 Plan 03: Long-cycle governance summary

**LPQ now turns accumulated evidence into unified skill, promotion, and value candidates while routing shared-truth-facing outputs into the existing pending governance queues**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-16T02:54:33+08:00
- **Completed:** 2026-04-16T03:02:27+08:00
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added a durable `RuminationCandidate` contract plus repository APIs so LPQ work persists one auditable row shape across skill templates, promotion candidates, and value-adjustment candidates.
- Implemented `drain_long_cycle` in `cognition::rumination` to synthesize long-cycle candidates from queued `AgentSearchReport` / citation payloads without re-running retrieval, working-memory assembly, value scoring, or Rig orchestration.
- Bridged promotion candidates into Phase 3 governance queues, attaching evidence for T3 review paths and creating pending ontology candidates for T2 paths while keeping skill/value outputs proposal-first inside rumination storage.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement unified LPQ candidate generation for the three long-cycle output classes** - `7b8bc8e` (`test`), `2f21b08` (`feat`)
2. **Task 2: Bridge shared-truth-facing LPQ outputs through Phase 3 governance seams** - `007a6eb` (`test`), `2828b91` (`feat`)

**Plan metadata:** recorded in the final docs commit for this plan.

## Files Created/Modified

- `src/cognition/rumination.rs` - adds long-cycle candidate reports, LPQ drain logic, and governance bridging through `TruthGovernanceService`.
- `src/memory/repository.rs` - adds typed rumination-candidate enums, payload serialization/parsing, and durable candidate list/get/insert/update methods.
- `tests/rumination_governance_integration.rs` - proves LPQ emits the three required candidate kinds and that promotion candidates surface in pending governance queues without auto-approval.

## Decisions Made

- Reused `rumination_candidates` as the only long-cycle durable store and embedded `governance_ref_id` into the persisted candidate payload so the contract stays additive and unified.
- Used the candidate source record’s truth layer to decide the governance bridge target instead of inventing a second promotion taxonomy inside rumination.
- Kept `skill_template` and `value_adjustment_candidate` inside the candidate store only; only `promotion_candidate` crosses into Phase 3 governance state.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo clippy` only emitted the existing environment-level `/home/tongyuan/.cargo/config` deprecation warning from Cargo itself; repository code remained clean under `-D warnings`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 5 is now complete: `SPQ`/`LPQ` routing, short-cycle local write-back, and long-cycle candidate/governance integration all satisfy the phase success criteria.
- Shared-truth mutation remains protected: long-cycle outputs are pending proposals or candidates only, with no auto-approval or direct T1/T2 authority writes.

## Self-Check: PASSED

- Verified `.planning/phases/05-rumination-and-adaptive-write-back/05-03-SUMMARY.md` exists on disk.
- Verified commits `7b8bc8e`, `2f21b08`, `007a6eb`, and `2828b91` exist in git history.
- Confirmed `rtk cargo test --test rumination_governance_integration -- --nocapture`, `rtk cargo test --test truth_governance -- --nocapture`, and `rtk proxy cargo clippy --all-targets -- -D warnings` are green; the only warning output is the existing environment-level `/home/tongyuan/.cargo/config` deprecation notice from Cargo.
