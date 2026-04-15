---
phase: 08-embedding-backend-and-index-foundation
plan: 02
subsystem: ingest
tags: [rust, ingest, embeddings, schema, repository]
requires:
  - phase: 08-embedding-backend-and-index-foundation
    provides: typed embedding backend foundation from 08-01
provides:
  - additive embedding schema and sidecar tables
  - chunk-aligned embedding persistence at ingest time
  - lexical-only ingest compatibility when embeddings are disabled
affects: [phase-8, embedding-foundation, ingest, schema]
tech-stack:
  added: []
  patterns:
    - embedding persistence stays additive and keyed to authority records
    - lexical-only ingest remains usable when semantic substrate is disabled
key-files:
  created:
    - migrations/0006_embedding_foundation.sql
    - .planning/phases/08-embedding-backend-and-index-foundation/08-02-SUMMARY.md
  modified:
    - src/core/migrations.rs
    - src/memory/repository.rs
    - src/ingest/mod.rs
    - tests/ingest_pipeline.rs
    - tests/foundation_schema.rs
key-decisions:
  - "Stored embeddings in additive `record_embeddings` side tables keyed by `memory_records(id)` rather than extending the authority table."
  - "Used a deterministic builtin hash-based embedding generator for the Phase 8 foundation so chunk-aligned persistence is real and testable without external runtime dependency."
  - "Added `record_embedding_index_state` as an explicit vector-sidecar/index substrate table for later diagnostic and bootstrap work."
patterns-established:
  - "Semantic substrate storage should follow the same chunk grain as the lexical/citation contract."
  - "Foundation-phase embedding persistence should be optional at ingest time and must never break lexical-only operation."
requirements-completed: [EMB-02]
duration: 1 plan
completed: 2026-04-16
---

# Phase 8 Plan 2: Embedding Backend And Index Foundation Summary

**Additive chunk-aligned embedding persistence and vector-sidecar schema foundation**

## Accomplishments

- Added migration `0006_embedding_foundation.sql` with additive `record_embeddings` and `record_embedding_index_state` tables.
- Extended `MemoryRepository` with typed record-embedding persistence and listing methods.
- Extended `IngestService` with optional embedding-aware ingest so chunk-aligned authority records can receive builtin embedding sidecars while lexical-only ingest remains unaffected when embeddings are disabled.
- Added ingest and schema regressions proving chunk-aligned embedding persistence and additive sidecar bootstrap.

## Verification

- `cargo test --test ingest_pipeline -- --nocapture`
- `cargo test --test foundation_schema -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

- The embedding generator used for this foundation phase is a deterministic builtin hash-based vectorizer rather than an external model runtime. This keeps the substrate real and locally testable while preserving the milestone’s optional/local-first constraint.

## Issues Encountered

- Existing schema-version assertions in `tests/foundation_schema.rs` had to be updated from `5` to `6` once the new additive embedding migration landed.

## User Setup Required

None.

## Next Phase Readiness

- Phase 8 now has real embedding persistence and sidecar schema, so Plan 08-03 can expose vector-sidecar readiness truthfully through operator surfaces.

## Self-Check: PASSED

- Verified `cargo test --test ingest_pipeline -- --nocapture` passes.
- Verified `cargo test --test foundation_schema -- --nocapture` passes.
- Verified `cargo clippy --all-targets -- -D warnings` passes.
