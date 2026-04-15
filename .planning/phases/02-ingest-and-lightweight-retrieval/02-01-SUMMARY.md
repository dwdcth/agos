---
phase: 02-ingest-and-lightweight-retrieval
plan: 01
subsystem: database
tags: [rust, sqlite, ingest, normalization, chunking, provenance]
requires:
  - phase: 01-foundation-kernel
    provides: typed memory records, rusqlite migrations, and the Phase 1 retrieval-mode contract
provides:
  - additive ingest metadata columns on `memory_records`
  - synchronous detect-normalize-chunk-persist ingest service
  - regression tests for normalization, chunk anchors, and ingest persistence
affects: [phase-2, ingest, retrieval, authority-store, citations]
tech-stack:
  added: []
  patterns:
    - authority-store metadata on `memory_records`
    - pure detect/normalize/chunk helpers
    - synchronous library-first ingest service
key-files:
  created:
    - migrations/0002_ingest_foundation.sql
    - src/ingest/mod.rs
    - src/ingest/detect.rs
    - src/ingest/normalize.rs
    - src/ingest/chunk.rs
    - tests/ingest_pipeline.rs
  modified:
    - src/core/migrations.rs
    - src/lib.rs
    - src/memory/record.rs
    - src/memory/repository.rs
    - tests/foundation_schema.rs
key-decisions:
  - "Kept `memory_records` as the only authority store and added chunk/validity metadata additively instead of introducing a second ingest table."
  - "Implemented ingest as synchronous detect -> normalize -> chunk -> persist services so ordinary retrieval remains usable without Rig, embeddings, or async runtime changes."
patterns-established:
  - "Authority-store-first ingest: chunk provenance and validity are persisted on each `memory_records` row before lexical indexing exists."
  - "Pure ingest helpers: `detect`, `normalize`, and `chunk` stay side-effect free; persistence lives in `IngestService`."
requirements-completed: [ING-01, ING-02]
duration: 9min
completed: 2026-04-15
---

# Phase 2 Plan 1: Ingest Foundation Summary

**Additive ingest schema with nullable validity windows and a synchronous detect-normalize-chunk pipeline persisted into `memory_records`**

## Performance

- **Duration:** 9min
- **Started:** 2026-04-15T11:56:47Z
- **Completed:** 2026-04-15T12:05:25Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Extended the authority schema to version 2 by adding chunk-order, anchor, content-hash, and nullable validity-window fields directly on `memory_records`.
- Added typed Rust models and repository hydration for chunk metadata so provenance and validity survive round-trip reads instead of being reconstructed later.
- Implemented a synchronous `IngestService` with pure `detect`, `normalize`, and `chunk` helpers that ingest plain text, note-like text, and conversation-like exports into persisted chunk records.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add additive ingest schema and typed chunk/validity metadata** - `b8fe03d`, `947ed31` (test, feat)
2. **Task 2: Implement detect-normalize-chunk-persist ingest orchestration** - `9daece0`, `d129283` (test, feat)

**Plan metadata:** Recorded in the final docs commit for this plan.

## Files Created/Modified

- `migrations/0002_ingest_foundation.sql` - adds chunk metadata columns and validity-window indexes to the authority table.
- `src/core/migrations.rs` - registers the Phase 2 ingest migration without changing earlier migration behavior.
- `src/memory/record.rs` - defines typed chunk anchors, chunk metadata, and validity-window fields on `MemoryRecord`.
- `src/memory/repository.rs` - persists and hydrates chunk metadata and validity windows from `memory_records`.
- `src/ingest/mod.rs` - exposes the synchronous ingest service, request/report types, and deterministic record-id generation.
- `src/ingest/detect.rs` - detects plain text and supported conversation-export container formats.
- `src/ingest/normalize.rs` - converts supported inputs into a stable `NormalizedSource` payload with preserved source metadata.
- `src/ingest/chunk.rs` - produces stable chunk drafts with order, anchors, and deterministic content hashes.
- `src/lib.rs` - exports the ingest module for library callers.
- `tests/foundation_schema.rs` - verifies schema version 2 and round-trip chunk/validity metadata.
- `tests/ingest_pipeline.rs` - verifies normalization, chunk anchor behavior, and persisted ingest metadata.

## Decisions Made

- Kept `memory_records` authoritative for ingested chunks so later lexical indexing can layer on top without introducing a second source of truth.
- Stored validity as explicit nullable `valid_from` / `valid_to` fields distinct from `recorded_at`, matching the Phase 2 research decision for later filtering and citation work.
- Used a synchronous service boundary to preserve the existing single-crate, no-LLM, no-Rig ordinary-retrieval baseline.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `02-02` can add `libsimple` and FTS5 lexical recall on top of already-persisted chunk anchors, hashes, and validity metadata.
- The Phase 1 three-mode retrieval config contract remains untouched; only lexical ingest groundwork became real in this plan.

## Self-Check: PASSED

- Verified `.planning/phases/02-ingest-and-lightweight-retrieval/02-01-SUMMARY.md` exists on disk.
- Verified task commits `b8fe03d`, `947ed31`, `9daece0`, and `d129283` exist in git history.
- Re-ran `cargo test --test foundation_schema -- --nocapture` and `cargo test --test ingest_pipeline -- --nocapture`; both passed.
- Scanned all files created or modified by the plan for placeholder/stub patterns and found none.
