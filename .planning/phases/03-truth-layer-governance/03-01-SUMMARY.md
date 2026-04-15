---
phase: 03-truth-layer-governance
plan: 01
subsystem: database
tags: [rust, sqlite, truth-layer, repository, governance]
requires:
  - phase: 02-ingest-and-lightweight-retrieval
    provides: authority-backed lexical retrieval, citations, and truth-layer filters over `memory_records`
provides:
  - additive schema version 4 governance tables for T3 state, promotion reviews, promotion evidence, and ontology candidates
  - typed Rust truth-governance models and repository read contracts for T1/T2/T3 projections
  - regression coverage proving Phase 2 ordinary retrieval still works on the upgraded schema
affects: [phase-3, truth-governance, storage, retrieval, ordinary-search]
tech-stack:
  added: []
  patterns:
    - authority row plus governance side tables instead of splitting content across multiple stores
    - typed truth-record projection at the repository boundary without changing lexical search SQL
key-files:
  created:
    - migrations/0004_truth_layer_governance.sql
    - src/memory/truth.rs
    - tests/truth_governance.rs
  modified:
    - src/core/migrations.rs
    - src/memory/mod.rs
    - src/memory/repository.rs
    - tests/foundation_schema.rs
    - tests/retrieval_cli.rs
    - tests/ingest_pipeline.rs
    - tests/status_cli.rs
key-decisions:
  - "Kept `memory_records` and `memory_records_fts` as the single authority backbone, and layered truth governance into additive side tables instead of splitting T1/T2/T3 into separate content stores."
  - "Inserted default T3 governance rows at repository write time so T3 records cannot silently exist without confidence and revocation state."
  - "Exposed typed `TruthRecord` projections from `MemoryRepository` while leaving the Phase 2 lexical retrieval path on the existing authority-table contract."
patterns-established:
  - "Truth governance stays auditable through queryable side tables and enums with `as_str`/`parse` helpers instead of opaque blobs or loose strings."
  - "Schema upgrades must carry compatibility assertions for both retrieval responses and status/inspect surfaces."
requirements-completed: [TRU-01, TRU-02]
duration: 10min
completed: 2026-04-15
---

# Phase 3 Plan 1: Truth Layer Foundation Summary

**Schema v4 truth governance with additive T3/review/candidate tables, typed truth-record repository reads, and preserved lexical-first retrieval compatibility**

## Performance

- **Duration:** 10min
- **Started:** 2026-04-15T14:31:36Z
- **Completed:** 2026-04-15T14:41:36Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added schema version 4 as an additive migration that keeps `memory_records` and `memory_records_fts` authoritative while introducing governance tables for T3 state, promotion reviews/evidence, and ontology candidates.
- Introduced typed truth-governance Rust models plus repository APIs that project T1, T2, and T3 into explicit read models instead of treating `truth_layer` as only a loose label.
- Preserved Phase 2 ordinary retrieval behavior on the upgraded schema, and cleared the pre-existing `tests/ingest_pipeline.rs` clippy blocker so the full verification suite runs cleanly.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add additive truth-governance schema and typed truth metadata models** - `75016db` (test), `2010069` (feat)
2. **Task 2: Extend repository queries with layer-aware semantics and retrieval compatibility checks** - `bf0c77e` (test), `5b8f863` (feat)

**Additional fix:** `db97769` (`fix`) aligned stale status assertions with schema version 4 during plan-level verification.

## Files Created/Modified

- `migrations/0004_truth_layer_governance.sql` - adds additive governance tables and indexes without changing the authority store or FTS sidecar.
- `src/core/migrations.rs` - registers schema version 4 as the truth-governance seam after the lexical sidecar migration.
- `src/memory/mod.rs` - exports the new truth model module.
- `src/memory/truth.rs` - defines typed T3, promotion-review, evidence, candidate, and `TruthRecord` contracts.
- `src/memory/repository.rs` - adds default T3-state persistence plus typed truth-record and T3-state reads.
- `tests/foundation_schema.rs` - verifies schema version 4 and governance-table presence while preserving lexical sidecar assertions.
- `tests/truth_governance.rs` - covers typed truth-layer projections and default T3 governance persistence.
- `tests/retrieval_cli.rs` - proves ordinary retrieval still works on a schema-version-4 database.
- `tests/ingest_pipeline.rs` - removes the unused import that had previously blocked `clippy -D warnings`.
- `tests/status_cli.rs` - updates status/init schema-version expectations after the migration bump.

## Decisions Made

- Kept the truth-governance seam additive so existing lexical recall, citations, and filter semantics remain untouched.
- Modeled confidence, revocation, and governance states as enums with explicit storage conversions to keep audits and SQL predicates stable.
- Made repository-level T3 default-state creation automatic because TRU-02 requires every persisted T3 record to carry governance metadata.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Regression] Updated stale status-surface assertions after the schema version bump**
- **Found during:** plan-level verification after Task 2
- **Issue:** Full-suite verification failed because `tests/status_cli.rs` still expected `schema_version: 3` in `init` and `inspect schema` output after the new truth-governance migration raised the schema to version 4.
- **Fix:** Updated the status/init expectations to schema version 4 so the pre-existing inspection contract remains accurate after the Phase 3 migration.
- **Files modified:** `tests/status_cli.rs`
- **Verification:** `cargo test --tests && cargo clippy --all-targets -- -D warnings`
- **Committed in:** `db97769`

---

**Total deviations:** 1 auto-fixed (1 regression)
**Impact on plan:** The fix was required to keep existing inspection regressions aligned with the new schema foundation. No scope creep beyond Phase 3 compatibility work.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `03-02` can build explicit promotion-review and evidence gate workflows on top of the new governance tables and typed repository seams.
- Ordinary lexical retrieval remains unchanged, so later truth-governance plans can add promotion logic without rewriting Phase 2 search behavior.

## Self-Check: PASSED

- Verified `.planning/phases/03-truth-layer-governance/03-01-SUMMARY.md`, `migrations/0004_truth_layer_governance.sql`, `src/memory/truth.rs`, and `tests/truth_governance.rs` exist on disk.
- Verified commits `75016db`, `2010069`, `bf0c77e`, `5b8f863`, and `db97769` exist in git history.
- Confirmed plan verification is green with `cargo test --tests` and `cargo clippy --all-targets -- -D warnings`.
