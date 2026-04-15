---
phase: 08-embedding-backend-and-index-foundation
plan: 01
subsystem: core
tags: [rust, config, status, doctor, embedding, diagnostics]
requires:
  - phase: 06-runtime-gate-enforcement
    provides: truthful runtime-gate and diagnostics pattern
provides:
  - concrete optional embedding backend config variant
  - truthful embedding backend readiness/status semantics
  - lexical-first-preserving doctor behavior for foundation-only semantic modes
affects: [phase-8, embedding-foundation, config, diagnostics]
tech-stack:
  added: []
  patterns:
    - lexical-only remains green while optional embedding substrate is inspected separately
    - semantic-primary modes stay blocked during foundation-only work
key-files:
  created:
    - .planning/phases/08-embedding-backend-and-index-foundation/08-01-SUMMARY.md
  modified:
    - src/core/config.rs
    - src/core/app.rs
    - src/core/status.rs
    - src/core/doctor.rs
    - tests/status_cli.rs
key-decisions:
  - "Introduced `EmbeddingBackend::Builtin` as the first concrete optional backend while keeping `Disabled` as the default."
  - "Kept lexical-only runtime readiness green even when builtin embedding is configured, so Phase 8 foundation work does not destabilize the lexical baseline."
  - "Explicitly blocked `embedding_only` and `hybrid` under builtin foundation-only mode until the dual-channel retrieval phase lands."
patterns-established:
  - "Embedding capability must be modeled as typed backend/readiness states, not as a single enabled flag."
  - "Foundation-only semantic work should appear in status/doctor output without implying semantic retrieval is already active."
requirements-completed: [EMB-01]
duration: 1 plan
completed: 2026-04-16
---

# Phase 8 Plan 1: Embedding Backend And Index Foundation Summary

**Typed builtin embedding backend and truthful diagnostics without breaking lexical-first defaults**

## Accomplishments

- Added a concrete optional `builtin` embedding backend variant to the typed config surface.
- Extended runtime readiness and status reporting so builtin embedding can show `ready` vs `deferred` states while lexical-only remains the default stable path.
- Updated `doctor` semantics so `embedding_only` and `hybrid` remain blocked under foundation-only builtin mode until the dual-channel retrieval phase.

## Verification

- `cargo test --test status_cli -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

None.

## Issues Encountered

- Cargo still emits the non-repo `/home/tongyuan/.cargo/config` deprecation warning, but repository verification stayed green.

## User Setup Required

None.

## Next Phase Readiness

- Phase 8 now has a real embedding backend/readiness contract, so Plan 08-02 can safely add additive embedding persistence and schema support.

## Self-Check: PASSED

- Verified `cargo test --test status_cli -- --nocapture` passes.
- Verified `cargo clippy --all-targets -- -D warnings` passes.
