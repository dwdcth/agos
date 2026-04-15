---
phase: 01-foundation-kernel
plan: 03
subsystem: cli
tags: [rust, clap, sqlite, diagnostics, status, doctor]
requires:
  - phase: 01-01
    provides: typed config, retrieval-mode contract, and shared runtime readiness
  - phase: 01-02
    provides: sqlite bootstrap, schema version inspection, and foundation storage schema
provides:
  - truthful Phase 1 status reporting with explicit configured/effective mode, dependency state, and index readiness
  - blocking doctor diagnostics for invalid or impossible mode/backend combinations
  - developer CLI commands for init, status, doctor, and inspect schema
affects: [phase-1, foundation-kernel, cli, startup, diagnostics]
tech-stack:
  added: []
  patterns:
    - status stays informational and side-effect free while doctor/init apply command-path-specific blocking rules
    - schema inspection reuses typed readiness data instead of ad-hoc SQL output in the CLI layer
key-files:
  created:
    - src/core/status.rs
    - src/core/doctor.rs
    - src/interfaces/cli.rs
    - tests/status_cli.rs
  modified:
    - src/core/app.rs
    - src/core/mod.rs
    - src/interfaces/mod.rs
    - src/main.rs
key-decisions:
  - "Kept `status` side-effect free: it reports missing schema/database state explicitly instead of creating the SQLite file on read."
  - "Used command-path-sensitive doctor policies so `init` only blocks invalid mode/backend combinations while `doctor` also flags reserved embedding runtimes as non-ready."
  - "Made dependency loading and index readiness first-class capability states (`disabled`, `deferred`, `missing`, `not_built_in_phase_1`) instead of hiding them behind a single readiness boolean."
patterns-established:
  - "CLI commands delegate to typed status/doctor reports and print human-readable sections rather than embedding validation logic in `main`."
  - "Reserved retrieval modes remain visible in output and tests, never silently coerced to `lexical_only`."
requirements-completed: [FND-01, FND-03]
duration: 6min
completed: 2026-04-15
---

# Phase 1 Plan 3: Foundation Kernel Summary

**Phase 1 startup diagnostics with honest three-mode status reporting, blocking doctor checks, and schema inspection commands**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-15T10:07:23Z
- **Completed:** 2026-04-15T10:13:11Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added a typed readiness/status layer that reports configured mode, effective mode, schema state, dependency loading state, and index readiness without collapsing reserved modes.
- Added blocking `doctor` logic for invalid or impossible Phase 1 runtime requests while keeping `status` informational and always successful.
- Wired `init`, `status`, `doctor`, and `inspect schema` into the CLI with integration coverage for exit semantics and human-readable output.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement startup validation and explicit readiness diagnostics**
   `9ec507a` (test), `43c76ec` (feat)
2. **Task 2: Wire CLI commands for init, status, doctor, and foundation inspection**
   `e02165d` (test), `e593526` (feat)

## Files Created/Modified

- `src/core/status.rs` - collects schema/readiness state and renders human-readable status output.
- `src/core/doctor.rs` - applies command-path blocking rules for invalid and impossible runtime combinations.
- `src/interfaces/cli.rs` - owns the Phase 1 `init`, `status`, `doctor`, and `inspect schema` command surface.
- `tests/status_cli.rs` - integration coverage for reserved-mode status behavior, doctor failures, init exit semantics, and schema inspection output.
- `src/core/app.rs` - resolves the configured DB path and keeps shared runtime readiness notes for the status layer.
- `src/core/mod.rs` - exports the new diagnostics modules.
- `src/interfaces/mod.rs` - re-exports the dedicated CLI module.
- `src/main.rs` - remains a thin entrypoint that parses args, loads config, and delegates command execution.

## Decisions Made

- `status` inspects the configured DB path without creating it, so missing database/schema state stays observable instead of being hidden by implicit initialization.
- `init` uses the same readiness model as `doctor`, but only blocks structurally invalid combinations such as `embedding_only + disabled`; reserved Phase 1 embedding modes remain visible as warnings rather than hard failures.
- `inspect schema` reports schema/base-table/index state from the typed status contract so future retrieval phases can extend readiness output without duplicating schema probes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added temporary command wiring in existing interface files to satisfy Task 1 command-level TDD**
- **Found during:** Task 1 (Implement startup validation and explicit readiness diagnostics)
- **Issue:** The plan’s first task verifies command-level `status`/`doctor` behavior, but the dedicated `src/interfaces/cli.rs` surface was scheduled for Task 2. Without minimal dispatch in the existing interface entrypoints, the red tests could not exercise the binary command path.
- **Fix:** Wired `status`/`doctor` through `src/interfaces/mod.rs` and `src/main.rs` in Task 1, then moved the full command surface into `src/interfaces/cli.rs` during Task 2 as planned.
- **Files modified:** `src/interfaces/mod.rs`, `src/main.rs`, `src/core/mod.rs`
- **Verification:** `cargo test --test status_cli status_exits_successfully_for_reserved_modes -- --nocapture`
- **Committed in:** `43c76ec`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The deviation was required to make the command-level TDD gate executable. Final structure still matches the planned `src/interfaces/cli.rs` ownership.

## Issues Encountered

- `cargo clippy --all-targets -- -D warnings` was clean for project code; the only warning during verification came from the local Cargo environment’s deprecated `/home/tongyuan/.cargo/config`, not from repository changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 2 can build lexical ingest and retrieval on top of an explicit CLI/operator surface for database initialization, schema inspection, and capability reporting.
- Reserved `embedding_only` and `hybrid` semantics are now locked into tests and output, so later embedding work must extend those states rather than rewriting them.

## Self-Check: PASSED

- Verified `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md` exists on disk.
- Verified commits `9ec507a`, `43c76ec`, `e02165d`, and `e593526` exist in `git log`.
