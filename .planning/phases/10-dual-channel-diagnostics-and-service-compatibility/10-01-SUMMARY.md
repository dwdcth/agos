---
phase: 10-dual-channel-diagnostics-and-service-compatibility
plan: 01
subsystem: operator-surface
tags: [rust, cli, retrieval, diagnostics, doctor, status, hybrid-search]
requires:
  - phase: 09-dual-channel-retrieval-fusion
    provides: lexical-only / embedding-only / hybrid retrieval behavior and trace contract
provides:
  - explicit `search --mode` selection for lexical-only, embedding-only, and hybrid retrieval
  - truthful status output showing active and gated retrieval channels
  - doctor readiness checks that reflect dual-channel operational state without breaking lexical-first defaults
affects: [phase-10, search, status, doctor, retrieval-cli]
tech-stack:
  added: []
  patterns:
    - operator-visible retrieval mode overrides reuse the shared runtime config seam
    - dual-channel readiness is exposed additively through status and doctor surfaces
key-files:
  created:
    - .planning/phases/10-dual-channel-diagnostics-and-service-compatibility/10-01-SUMMARY.md
  modified:
    - src/interfaces/cli.rs
    - src/core/status.rs
    - src/core/doctor.rs
    - src/search/mod.rs
    - src/core/config.rs
    - src/core/app.rs
    - tests/status_cli.rs
    - tests/retrieval_cli.rs
key-decisions:
  - "Kept mode selection on the existing `search` surface instead of adding a separate semantic CLI, so lexical-only remains the default operator path."
  - "Reported `active_channels` and `gated_channels` additively so operators can see channel state without losing the existing capability-state detail."
  - "Made doctor fail on missing embedding readiness only when the configured mode actually requires that second channel."
patterns-established:
  - "Search/runtime mode overrides should load through `AppContext` and the shared `SearchService` seam rather than inventing parallel retrieval wiring."
  - "Dual-channel diagnostics should explain what is active and what is gated, not collapse everything into a single ready/not-ready flag."
requirements-completed: [OPS-01, OPS-02]
duration: 1 plan
completed: 2026-04-17
---

# Phase 10 Plan 1 Summary

**Mode-aware search CLI and truthful dual-channel diagnostics for lexical, embedding, and hybrid retrieval**

## Accomplishments

- Added `search --mode` so operators can intentionally run lexical-only, embedding-only, or hybrid retrieval through the ordinary search surface.
- Extended status output with `active_channels` and `gated_channels`, making dual-channel readiness visible without hiding detailed capability states.
- Updated doctor semantics so lexical-only stays stable while embedding-only and hybrid surface the exact readiness failures they depend on.
- Added regression coverage for operator-visible dual-channel mode selection and diagnostics.

## Verification

- `cargo test --test status_cli dual_channel_status_and_doctor_report_mode_compatibility_truthfully -- --nocapture`
- `cargo test --test retrieval_cli search_surface_respects_dual_channel_mode_selection -- --nocapture`
- `cargo test --test status_cli -- --nocapture`
- `cargo test --test retrieval_cli -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

- `src/core/config.rs` and `src/core/app.rs` also needed additive updates so runtime config and readiness tests could carry the root `vector` settings used by dual-channel operator flows.

## Issues Encountered

- Phase 10 work-in-progress already included partial agent-search mode plumbing. That code was separated back out so this plan stayed limited to ordinary search and operator diagnostics.

## User Setup Required

None.

## Next Phase Readiness

- Ordinary search now has explicit mode control and truthful diagnostics.
- Phase 10 Plan 2 can focus on threading the same mode selection through agent-search while preserving the shared ordinary retrieval seam.

## Self-Check: PASSED

- Verified focused and full `status_cli` regressions pass.
- Verified focused and full `retrieval_cli` regressions pass.
- Verified `cargo clippy --all-targets -- -D warnings` exits successfully in this wave.
