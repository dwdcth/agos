---
phase: 03-truth-layer-governance
plan: 02
subsystem: database
tags: [rust, sqlite, truth-layer, governance, testing]
requires:
  - phase: 03-01
    provides: additive truth-governance tables, typed truth metadata, and repository truth projections
provides:
  - synchronous T3 promotion governance service with review, evidence, and gate-state lifecycle commands
  - derived T2 authority record creation that leaves source T3 records intact and auditable
  - integration coverage for pending, rejected, and approved promotion flows
affects: [phase-3, truth-governance, storage, retrieval, ordinary-search]
tech-stack:
  added: []
  patterns:
    - synchronous governance orchestration over repository methods instead of embedding SQL in services
    - derived T2 authority rows with provenance links instead of in-place T3 truth-layer mutation
key-files:
  created:
    - src/memory/governance.rs
  modified:
    - src/memory/mod.rs
    - src/memory/repository.rs
    - src/memory/truth.rs
    - tests/truth_governance.rs
key-decisions:
  - "Kept truth governance library-first and synchronous, matching the existing ingest/search service shape while routing all persistence through `MemoryRepository`."
  - "Treat T3 promotion as derived T2 record creation with preserved source audit state rather than mutating the source record's truth layer."
  - "Refresh `truth_t3_state.last_reviewed_at` on review mutations so pending, approved, and rejected promotion flows remain auditable from the source hypothesis."
patterns-established:
  - "Truth-governance services may coordinate multiple repository operations, but SQL stays inside repository methods."
  - "Source T3 records retain review history and governance timestamps even after shared-truth promotion succeeds."
requirements-completed: [TRU-02, TRU-03]
duration: 8min
completed: 2026-04-15
---

# Phase 3 Plan 2: Truth Promotion Governance Summary

**Governed T3 promotion reviews with explicit gate states, derived T2 record creation, and preserved source-hypothesis audit trails**

## Performance

- **Duration:** 8min
- **Started:** 2026-04-15T14:51:07Z
- **Completed:** 2026-04-15T14:58:37Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `TruthGovernanceService` as a synchronous orchestration layer for promotion review creation, evidence attachment, gate-state updates, rejection, and approval.
- Extended repository-backed truth projections so T3 records expose review history and T2 records expose candidate history without disturbing the Phase 2 lexical-first retrieval path.
- Enforced all-gates-passed approval that writes a derived T2 authority row, stamps source T3 review timestamps, and keeps the source hypothesis intact for later revocation and audit.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement promotion-review lifecycle commands over the governance repository seam** - `f95b02d` (test), `a100427` (feat)
2. **Task 2: Enforce all-gates-passed approval and derive a new T2 authority record** - `1d27b52` (test), `9ec119d` (feat)

## Files Created/Modified

- `src/memory/governance.rs` - defines promotion review requests, reports, error types, and governance lifecycle orchestration.
- `src/memory/mod.rs` - exports the governance module to the library surface.
- `src/memory/repository.rs` - persists promotion reviews, evidence rows, ontology candidate reads, and T3 review timestamps.
- `src/memory/truth.rs` - expands truth projections to expose review and candidate collections alongside existing truth-layer metadata.
- `tests/truth_governance.rs` - covers pending, rejected, and approved T3 promotion flows plus source-audit preservation.

## Decisions Made

- Kept governance logic above repository CRUD so truth-gate policy stays testable without hiding ad hoc SQL inside the service.
- Treated T3 promotion approval as creation of a new T2 authority row to preserve the source T3 record's revocation and provenance trail.
- Updated `truth_t3_state.last_reviewed_at` during review mutations so the source hypothesis remains auditable after both approval and rejection.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Expanded truth projections to match the plan's governance read model**
- **Found during:** Task 1
- **Issue:** The live `03-01` code exposed narrower `TruthRecord` variants than the `03-02` plan contract, which blocked review-history assertions and governance inspection tests.
- **Fix:** Extended `TruthRecord` to carry T3 review collections and T2 candidate collections, and loaded those via repository helpers while keeping lexical retrieval on the existing `memory_records` backbone.
- **Files modified:** `src/memory/truth.rs`, `src/memory/repository.rs`
- **Verification:** `cargo test --test truth_governance -- --nocapture`; `cargo test --tests && cargo clippy --all-targets -- -D warnings`
- **Committed in:** `a100427`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The deviation aligned the implementation with the existing plan contract and was necessary for auditable governance reads. No Phase 4/5 scope was introduced.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `03-03` can build T2-to-T1 candidate creation on top of the same repository-backed governance seam without rewriting promotion logic.
- Ordinary lexical retrieval remains compatible because promotion governance stayed additive to `memory_records` and its side tables.

## Self-Check: PASSED

- Verified `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md`, `src/memory/governance.rs`, and `tests/truth_governance.rs` exist on disk.
- Verified commits `f95b02d`, `a100427`, `1d27b52`, and `9ec119d` exist in git history.
- Confirmed plan verification is green with `cargo test --tests` and `cargo clippy --all-targets -- -D warnings`.
