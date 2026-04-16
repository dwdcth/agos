---
phase: 09-dual-channel-retrieval-fusion
plan: 02
subsystem: search
tags: [rust, retrieval, lexical-first, embedding, hybrid, fusion]
requires:
  - phase: 09-dual-channel-retrieval-fusion
    provides: config-derived retrieval mode matrix from 09-01
provides:
  - SearchService dispatch for lexical-only, embedding-only, and hybrid modes
  - embedding recall over additive sidecar data
  - hybrid dedupe/fusion by record identity with lexical-first explanation preserved
affects: [phase-9, ordinary-retrieval, search, dual-channel]
tech-stack:
  added: []
  patterns:
    - one SearchService seam now serves all three retrieval modes
    - hybrid dedupe happens on authority record identity before final result shaping
key-files:
  created:
    - .planning/phases/09-dual-channel-retrieval-fusion/09-02-SUMMARY.md
  modified:
    - src/search/mod.rs
    - src/search/lexical.rs
    - src/search/score.rs
    - tests/dual_channel_retrieval.rs
    - tests/lexical_search.rs
key-decisions:
  - "Kept one ordinary retrieval service and added mode dispatch inside `SearchService` instead of forking semantic retrieval into a separate product path."
  - "Used additive embedding recall over `record_embeddings` and merged candidates on `record_id` so citations and authority identity stay stable."
  - "Preserved lexical-first explanation by keeping lexical snippets/citations as the main result surface even when embedding contributes."
patterns-established:
  - "Dual-channel retrieval should reuse the ordinary retrieval contract, not bypass it."
  - "Generated config variants are now the source of truth for lexical-only / embedding-only / hybrid behavior tests."
requirements-completed: [DCR-01, DCR-02, DCR-03]
duration: 1 plan
completed: 2026-04-16
---

# Phase 9 Plan 2: Dual-Channel Retrieval Fusion Summary

**Shared SearchService dispatch for lexical-only, embedding-only, and hybrid retrieval**

## Accomplishments

- Extended `SearchService` so it can dispatch lexical-only, embedding-only, and hybrid flows through one ordinary retrieval seam.
- Added embedding recall over persisted `record_embeddings` rows, keyed to authority record identity.
- Implemented hybrid candidate merge/dedupe on `record_id` before final result shaping.
- Added dual-channel retrieval regressions proving lexical-only, embedding-only, and hybrid behavior from generated config variants while keeping lexical-only search stable.

## Verification

- `cargo test --test dual_channel_retrieval -- --nocapture`
- `cargo test --test lexical_search -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

- Embedding recall currently uses the builtin deterministic vectorizer over persisted sidecar data rather than a provider-backed embedding runtime. This keeps the dual-channel behavior locally testable and aligned with the Phase 8 foundation.

## Issues Encountered

- Cargo still emits the non-repo `/home/tongyuan/.cargo/config` deprecation warning, but repository verification stayed green.

## User Setup Required

None.

## Next Phase Readiness

- Dual-channel retrieval behavior is now live, so Plan 09-03 can focus purely on making channel contribution explainable in the final trace/response contract.

## Self-Check: PASSED

- Verified `cargo test --test dual_channel_retrieval -- --nocapture` passes.
- Verified `cargo test --test lexical_search -- --nocapture` passes.
- Verified `cargo clippy --all-targets -- -D warnings` passes.
