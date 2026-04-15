---
phase: 03-truth-layer-governance
plan: 03
subsystem: database
tags: [rust, sqlite, truth-layer, governance, testing]
requires:
  - phase: 03-02
    provides: synchronous T3 promotion governance, derived T2 authority rows, and typed truth projections
provides:
  - candidate-only T2 to T1 ontology proposal creation with source-layer and basis-record validation
  - repository-backed pending review and pending candidate governance queues
  - integration coverage proving candidate creation never mutates T1 authority records or retrieval behavior
affects: [phase-3, truth-governance, storage, retrieval, ordinary-search]
tech-stack:
  added: []
  patterns:
    - candidate-first T2 to T1 governance that writes only reviewable side-table rows
    - pending governance queues exposed through service methods while ordinary retrieval stays untouched
key-files:
  created: []
  modified:
    - src/memory/governance.rs
    - src/memory/repository.rs
    - tests/truth_governance.rs
key-decisions:
  - "Kept T2 to T1 evolution candidate-only by persisting ontology proposals in `truth_ontology_candidates` without creating or mutating T1 authority rows."
  - "Validated basis-record references during candidate creation so governance state cannot persist opaque or dangling proposal evidence."
  - "Exposed pending reviews and pending candidates as repository-backed governance queues instead of overloading ordinary retrieval APIs."
patterns-established:
  - "Truth-governance service methods may wrap repository truth projections directly when the API is governance-only and non-mutating."
  - "Pending-state queue reads filter on explicit review/candidate lifecycle enums rather than inferring queue membership from search results."
requirements-completed: [TRU-01, TRU-04]
duration: 4min
completed: 2026-04-15
---

# Phase 3 Plan 3: Candidate-First Governance Summary

**Candidate-only T2 to T1 ontology proposals with explicit pending governance queues and no automatic T1 mutation**

## Performance

- **Duration:** 4min
- **Started:** 2026-04-15T15:04:27Z
- **Completed:** 2026-04-15T15:08:34Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added a T2-to-T1 candidate creation command that persists explicit ontology proposal rows, validates source/basis records, and never rewrites shared T1 authority data.
- Exposed governance-oriented pending review and pending candidate queue APIs while preserving typed T1/T2/T3 truth projections through the service layer.
- Extended `truth_governance` integration coverage to prove candidate-only behavior, queue semantics, and continued separation from ordinary retrieval.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add candidate-first T2 -> T1 commands that never mutate T1 automatically** - `b133267` (test), `dadeb7a` (feat)
2. **Task 2: Expose governance-oriented queue APIs for pending reviews and candidates** - `a3bb193` (test), `24ea4b1` (feat)

## Files Created/Modified

- `src/memory/governance.rs` - adds T2 candidate requests, basis/source validation, typed truth lookup, and pending governance queue service APIs.
- `src/memory/repository.rs` - persists ontology candidate rows and provides pending review/candidate read methods filtered by explicit lifecycle state.
- `tests/truth_governance.rs` - covers candidate-only T2 to T1 behavior, invalid-source rejection, and distinct pending governance queues.

## Decisions Made

- Kept candidate creation in the governance service and side tables only so Phase 2 lexical retrieval and authority-row semantics remain unchanged.
- Treated basis-record validation as a required governance invariant because proposal evidence crosses into durable state and must stay auditable.
- Reused the existing `TruthRecord` projection model for service reads instead of inventing a queue-specific raw-row API.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 3 is now complete: T1/T2/T3 storage, T3 promotion governance, and T2 candidate governance are all in place.
- Phase 4 can consume pending review/candidate queues and typed truth projections without changing ordinary lexical retrieval or introducing Rig/working-memory scope here.

## Self-Check: PASSED

- Verified `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`, `src/memory/governance.rs`, `src/memory/repository.rs`, and `tests/truth_governance.rs` exist on disk.
- Verified commits `b133267`, `dadeb7a`, `a3bb193`, and `24ea4b1` exist in git history.
- Confirmed plan verification is green with `cargo test --test truth_governance -- --nocapture`, `cargo test --tests`, and `cargo clippy --all-targets -- -D warnings`.
