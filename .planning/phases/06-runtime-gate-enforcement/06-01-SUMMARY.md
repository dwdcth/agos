---
phase: 06-runtime-gate-enforcement
plan: 01
subsystem: cli
tags: [rust, clap, diagnostics, runtime-gate, status, doctor]
requires:
  - phase: 01-foundation-kernel
    provides: typed status/doctor diagnostics and command-path gating precedent
  - phase: 02-ingest-and-lightweight-retrieval
    provides: thin CLI wrappers for ingest/search
  - phase: 04-working-memory-and-agent-search
    provides: developer-facing agent-search CLI entrypoint
provides:
  - shared preflight gating for `ingest`, `search`, and `agent-search`
  - operational command-path expansion on top of `DoctorReport`
  - CLI-process regression coverage for reserved and invalid runtime mode combinations
affects: [phase-6, runtime-gate-enforcement, cli, diagnostics]
tech-stack:
  added: []
  patterns:
    - operational commands reuse typed status/doctor diagnostics before DB and service execution
    - reserved semantic modes stay explicit and blocked rather than silently falling back
key-files:
  created:
    - tests/runtime_gate_cli.rs
  modified:
    - src/core/doctor.rs
    - src/interfaces/cli.rs
key-decisions:
  - "Extended `CommandPath` with `Ingest`, `Search`, and `AgentSearch` instead of introducing a second operational gate enum."
  - "Promoted lexical-only runtime-not-ready states into doctor-style failures only for operational command paths, while leaving informational command behavior untouched."
  - "Added one shared CLI preflight helper so runtime gate logic cannot drift across command implementations."
patterns-established:
  - "Operational CLI paths must evaluate typed readiness before `Database::open(...)` or downstream service invocation."
  - "Cross-command runtime gate regressions belong in a dedicated CLI-process test harness rather than being hidden in library tests."
requirements-completed: [FND-01, FND-03, AGT-01]
duration: 1 plan
completed: 2026-04-16
---

# Phase 6 Plan 1: Runtime Gate Enforcement Summary

**Shared runtime preflight for `ingest`, `search`, and `agent-search` with cross-command CLI regression coverage**

## Accomplishments

- Added `tests/runtime_gate_cli.rs` to expose the exact audit gap at the binary entrypoint and cover both blocked semantic-mode configs and lexical-ready success paths.
- Extended `DoctorReport` command paths so operational commands can reuse the same typed diagnostic surface as `doctor`.
- Added one shared CLI preflight helper that blocks `ingest`, `search`, and `agent-search` before `Database::open(...)` when the runtime is not actually executable.

## Verification

- `cargo test --test runtime_gate_cli -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

None.

## Issues Encountered

- Cargo still emits the non-repo `/home/tongyuan/.cargo/config` deprecation warning, but repository verification stayed green.

## User Setup Required

None.

## Next Phase Readiness

- Phase 6 now has the actual runtime gate at the CLI boundary, so the remaining work is regression hardening for lexical not-ready states and informational command semantics.

## Self-Check: PASSED

- Verified `tests/runtime_gate_cli.rs` exists on disk.
- Verified `cargo test --test runtime_gate_cli -- --nocapture` passes.
- Verified `cargo clippy --all-targets -- -D warnings` passes.
