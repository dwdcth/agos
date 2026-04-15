---
phase: 01-foundation-kernel
plan: 02
subsystem: database
tags: [rust, sqlite, rusqlite, rusqlite_migration, serde_json, memory-model]
requires:
  - phase: 01-01
    provides: typed runtime config, three-mode retrieval contract, and shared bootstrap module layout
provides:
  - deterministic SQLite bootstrap with additive phase-1 migrations
  - foundation memory_records schema with source, timestamp, scope, record type, truth layer, and provenance fields
  - typed memory repository APIs for insert, get, list, count, and scope inspection
affects: [phase-1, foundation-kernel, storage, status, ingest]
tech-stack:
  added: [serde_json]
  patterns:
    - database bootstrap owns directory creation and migration application
    - memory metadata remains strongly typed in Rust and serialized explicitly at the repository boundary
    - provenance is stored as first-class JSON instead of reconstructed later
key-files:
  created:
    - migrations/0001_foundation.sql
    - src/core/db.rs
    - src/core/migrations.rs
    - src/memory/mod.rs
    - src/memory/record.rs
    - src/memory/repository.rs
    - tests/foundation_schema.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/lib.rs
    - src/core/mod.rs
key-decisions:
  - "Kept the phase-1 schema limited to a single `memory_records` table plus base indexes so later retrieval and truth-governance phases can extend additively."
  - "Stored provenance as explicit JSON text to preserve auditability without introducing later-phase ranking or cognition fields."
  - "Moved SQL CRUD into `MemoryRepository` so `Database` stays focused on connection lifecycle and schema state."
patterns-established:
  - "Foundation persistence uses `rusqlite_migration` for deterministic schema application and `PRAGMA user_version` for status-friendly version inspection."
  - "Typed enums/newtypes cross the SQL boundary through small `as_str`/`parse` helpers rather than free-form strings in application code."
requirements-completed: [FND-01, FND-02]
duration: 5min
completed: 2026-04-15
---

# Phase 1 Plan 2: Foundation Kernel Summary

**SQLite bootstrap, additive foundation migration, and typed memory record persistence with provenance-aware repository APIs**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-15T09:54:02Z
- **Completed:** 2026-04-15T09:58:40Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added deterministic SQLite open-and-migrate bootstrap that creates parent directories, applies the phase-1 migration once, and surfaces schema version for later status commands.
- Introduced a foundation `memory_records` schema that persists source, timestamps, scope, record type, truth layer, provenance, and content without leaking FTS/vector/agent tables into Phase 1.
- Implemented typed memory models plus a repository that round-trips records, lists stored entries, counts records, and reports scope counts for inspection workflows.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add deterministic SQLite bootstrap and foundation migrations**
   `2a3a4d0` (test), `c966b86` (feat)
2. **Task 2: Implement typed memory base entities and repository persistence**
   `e7add24` (test), `7e600e8` (feat)

## Files Created/Modified

- `migrations/0001_foundation.sql` - additive phase-1 schema for `memory_records` and base inspection indexes.
- `src/core/db.rs` - `Database` open/bootstrap path, schema version probe, and connection accessors.
- `src/core/migrations.rs` - migration registration and `rusqlite_migration` execution seam.
- `src/memory/mod.rs` - memory module exports.
- `src/memory/record.rs` - typed source, timestamp, scope, record type, truth layer, provenance, and `MemoryRecord` contracts.
- `src/memory/repository.rs` - repository insert/get/list/count/scope-count APIs and row-to-struct mapping.
- `tests/foundation_schema.rs` - integration coverage for bootstrap idempotence, phase-1-only schema shape, and memory record round-trip behavior.
- `Cargo.toml` - added `serde_json` for inspectable provenance serialization.
- `Cargo.lock` - locked provenance serialization dependency graph.
- `src/lib.rs` - exported the new `memory` module.
- `src/core/mod.rs` - exported database and migrations modules.

## Decisions Made

- Used `rusqlite_migration` as the migration runner, but read `PRAGMA user_version` directly from `Database::schema_version()` so the upcoming status commands can inspect schema state cheaply.
- Kept `source_kind`, `scope`, `record_type`, and `truth_layer` as typed enums in application code and converted them only at the repository boundary.
- Left retrieval, vector, working-memory, and promotion-gate fields entirely out of the foundation schema to preserve the three-mode retrieval contract from Plan `01-01`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `serde_json` to support explicit provenance JSON persistence**
- **Found during:** Task 2 (typed memory base entities and repository persistence)
- **Issue:** The plan requires provenance to remain first-class and inspectable, and the schema already stores `provenance_json`; without a JSON serializer the repository would need a lossy ad-hoc encoding.
- **Fix:** Added `serde_json` to the crate and used it to serialize and deserialize `Provenance` at the repository boundary.
- **Files modified:** `Cargo.toml`, `Cargo.lock`, `src/memory/repository.rs`, `src/memory/record.rs`
- **Verification:** `cargo test --test foundation_schema -- --nocapture`
- **Committed in:** `7e600e8`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The added dependency was required for correct provenance persistence. No scope creep beyond the foundation storage contract.

## Issues Encountered

- `rusqlite::Statement::query_map` requires closures returning `rusqlite::Result`, so the repository list path was switched to explicit row iteration to preserve richer repository error types.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `01-03` can inspect schema version and record counts from stable database and repository seams.
- Phase 2 can extend storage additively with ingest and lexical retrieval artifacts without rewriting the base entity shape.

## Self-Check: PASSED

- Verified `.planning/phases/01-foundation-kernel/01-02-SUMMARY.md` and all created code files exist on disk.
- Verified commits `2a3a4d0`, `c966b86`, `e7add24`, and `7e600e8` exist in `git log`.
