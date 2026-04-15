---
phase: 05-rumination-and-adaptive-write-back
plan: 01
subsystem: database
tags: [rust, sqlite, rumination, spq, lpq]
requires:
  - phase: 03-truth-layer-governance
    provides: candidate-first truth governance seams and additive side-table patterns
  - phase: 04-working-memory-and-agent-search
    provides: DecisionReport and AgentSearchReport inputs for bounded rumination triggers
provides:
  - explicit durable SPQ and LPQ queue tables with mirrored item shape
  - durable trigger-state throttling for dedupe, cooldown, and budget enforcement
  - a local-first rumination service that claims SPQ ahead of LPQ and consumes Phase 4 reports without bypassing them
affects: [05-02, 05-03, cognition, memory]
tech-stack:
  added: []
  patterns:
    - explicit dual queues over a mixed learning batch
    - repository-backed throttle ledgers and SPQ-first claim ordering
    - report-to-queue normalization without reimplementing retrieval or Rig orchestration
key-files:
  created:
    - migrations/0005_rumination_writeback.sql
    - src/cognition/rumination.rs
    - tests/rumination_queue.rs
  modified:
    - src/core/migrations.rs
    - src/cognition/mod.rs
    - src/memory/repository.rs
    - tests/foundation_schema.rs
key-decisions:
  - "Persisted SPQ and LPQ as separate mirrored queue tables to keep short-cycle and long-cycle work explicit and auditable."
  - "Stored dedupe, cooldown, and budget outcomes in rumination_trigger_state instead of inferring them from queue history."
  - "Normalized DecisionReport and AgentSearchReport into queue payloads while keeping scheduling synchronous and local-first."
patterns-established:
  - "Dual-queue control plane: SPQ handles corrective local work first, LPQ remains candidate-oriented and lower priority."
  - "Throttle-ledger pattern: routing decisions write durable last-decision state alongside queue inserts."
requirements-completed: [LRN-01]
duration: 12min
completed: 2026-04-16
---

# Phase 05 Plan 01: Rumination scheduling summary

**Explicit SPQ/LPQ queue tables, durable throttle ledgers, and a bounded local-first rumination scheduler over Phase 4 reports**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-15T18:17:08Z
- **Completed:** 2026-04-15T18:29:18Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added Phase 5 migration `0005_rumination_writeback.sql` with explicit `spq_queue_items`, `lpq_queue_items`, `rumination_trigger_state`, `local_adaptation_entries`, and `rumination_candidates` side tables.
- Implemented `cognition::rumination` as the bounded scheduling seam that routes locked trigger classes, applies `route -> dedupe -> cooldown -> budget -> enqueue`, and claims SPQ before LPQ.
- Extended repository and regression tests so queue durability, throttling, retry behavior, and additive schema compatibility are all covered without introducing any truth-write path.

## Task Commits

1. **Task 1: Add the Phase 5 durable queue and throttle schema** - `101f957` (feat)
2. **Task 2: Implement trigger normalization and bounded SPQ/LPQ scheduling** - `9f431e6` (feat)

## Files Created/Modified
- `migrations/0005_rumination_writeback.sql` - additive Phase 5 schema for explicit rumination queues, throttle state, local adaptations, and long-cycle candidates.
- `src/core/migrations.rs` - registers the Phase 5 migration as schema version 5.
- `src/cognition/mod.rs` - exports the rumination seam from the cognition module tree.
- `src/cognition/rumination.rs` - normalizes Phase 4 reports into typed triggers and enforces SPQ-first local scheduling.
- `src/memory/repository.rs` - persists queue items, trigger-state ledgers, claim ordering, completion, and retry semantics.
- `tests/foundation_schema.rs` - verifies schema version 5 and the additive rumination side tables.
- `tests/rumination_queue.rs` - proves routing, dedupe, cooldown, budget, retry, and SPQ-over-LPQ priority.

## Decisions Made

- Kept `SPQ` and `LPQ` as separate tables with mirrored columns rather than one mixed queue table so the short-cycle vs. long-cycle split remains explicit in storage and inspection.
- Used `rumination_trigger_state` as the durable throttle ledger for dedupe, cooldown, and budget outcomes so repeated trigger storms are bounded across commands.
- Consumed `DecisionReport` and `AgentSearchReport` as the only Phase 4 input envelopes for scheduling, preserving the existing retrieval, working-memory, value, and Rig boundaries.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `rtk cargo clippy --all-targets -- -D warnings` emits two environment-level warnings about `/home/tongyuan/.cargo/config` deprecation. Repository code is clean under the lint gate.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 5 now has the durable control plane required for `05-02`: short-cycle write-back can consume claimed SPQ items and store local-only overlays in `local_adaptation_entries`.
- Phase 5 also has the queue and candidate scaffolding required for `05-03`: LPQ processors can emit `skill_template`, `promotion_candidate`, and `value_adjustment_candidate` rows into `rumination_candidates`.
- Shared truth still remains protected: this plan did not add any direct T1/T2 mutation path and long-cycle work can stay candidate/proposal-first.

## Self-Check: PASSED

- Verified `.planning/phases/05-rumination-and-adaptive-write-back/05-01-SUMMARY.md`, `migrations/0005_rumination_writeback.sql`, `src/cognition/rumination.rs`, and `tests/rumination_queue.rs` exist on disk.
- Verified commits `101f957` and `9f431e6` exist in git history.
- Confirmed `rtk cargo test --test foundation_schema rumination_schema_bootstraps_version_5_side_tables -- --nocapture`, `rtk cargo test --test rumination_queue -- --nocapture`, and `rtk cargo clippy --all-targets -- -D warnings` are green aside from the non-repo Cargo config deprecation warnings.
