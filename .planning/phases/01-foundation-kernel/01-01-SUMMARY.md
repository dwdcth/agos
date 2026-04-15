---
phase: 01-foundation-kernel
plan: 01
subsystem: foundation
tags: [rust, clap, serde, toml, rusqlite, bootstrap, config]
requires: []
provides:
  - single-crate Rust binary scaffold for Agent Memos
  - typed TOML retrieval-mode and embedding-backend config contracts
  - shared bootstrap readiness types for later database and status plans
affects: [phase-1, foundation-kernel, config, startup]
tech-stack:
  added: [anyhow, clap, directories, rusqlite, rusqlite_migration, serde, thiserror, toml, tracing]
  patterns:
    - single-crate single-binary bootstrap
    - typed retrieval intent enums instead of boolean feature switches
    - readiness contracts that preserve configured and effective retrieval modes
key-files:
  created:
    - Cargo.toml
    - Cargo.lock
    - src/lib.rs
    - src/main.rs
    - src/core/config.rs
    - src/core/app.rs
    - src/interfaces/mod.rs
    - config/agent-memos.toml.example
  modified:
    - src/core/mod.rs
key-decisions:
  - "Used a single crate with a thin clap-driven entrypoint to match the mempal-style bootstrap without pulling later-phase retrieval or agent dependencies."
  - "Kept retrieval intent (`lexical_only`, `embedding_only`, `hybrid`) separate from embedding backend state so later semantic backends can remain optional."
  - "Preserved reserved modes as typed readiness states by keeping `effective_mode` aligned with the configured mode and using `ready` plus notes to explain deferred behavior."
patterns-established:
  - "Foundation config uses typed enums plus serde defaults, never a free-form mode string or boolean embedding flag."
  - "Main loads config into `AppContext` and leaves command behavior outside the binary entrypoint."
requirements-completed: [FND-01]
duration: 4min
completed: 2026-04-15
---

# Phase 1 Plan 1: Foundation Kernel Summary

**Single-crate Rust bootstrap with typed three-mode retrieval config and shared runtime readiness contracts**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-15T09:43:22Z
- **Completed:** 2026-04-15T09:46:54Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Bootstrapped the repository as a single-binary Rust crate with only foundation dependencies and a thin `main`.
- Added typed TOML config contracts for `lexical_only`, `embedding_only`, and `hybrid`, plus separate embedding backend state.
- Added `AppContext` and `RuntimeReadiness` so later plans can extend startup, schema, and status checks without redesigning config semantics.

## Task Commits

Each task was committed atomically:

1. **Task 1: Scaffold the single-crate module layout and binary entrypoint**
   `bbbfb4f` (test), `feabc22` (feat)
2. **Task 2: Implement typed TOML config loading and shared runtime readiness contracts**
   `068b52f` (test), `2adcfc7` (feat)
3. **Verification follow-up**
   `f4b0ff2` (fix)

## Files Created/Modified

- `Cargo.toml` - single-crate manifest with only bootstrap/runtime dependencies.
- `Cargo.lock` - locked foundation dependency graph for deterministic local builds.
- `src/lib.rs` - minimal public module graph exposing only `core` and `interfaces`.
- `src/main.rs` - thin binary entrypoint that parses CLI input, loads config, and builds `AppContext`.
- `src/core/mod.rs` - exports the shared `app` and `config` foundation modules.
- `src/core/config.rs` - typed TOML loader, retrieval mode enums, embedding backend enums, and config tests.
- `src/core/app.rs` - `AppContext` and `RuntimeReadiness` bootstrap contracts.
- `src/interfaces/mod.rs` - initial clap CLI surface with config-path selection.
- `config/agent-memos.toml.example` - documented example config for all three retrieval intents.

## Decisions Made

- Used `directories::ProjectDirs` for the default config path while keeping the database default stable and local-first.
- Treated `embedding_only` and `hybrid` as explicit deferred states in readiness instead of silently downgrading them to `lexical_only`.
- Kept placeholder embedding settings confined to the example TOML and excluded all later-phase retrieval/agent dependencies from the manifest.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wired `src/core/mod.rs` during Task 2 so new contracts compiled**
- **Found during:** Task 2
- **Issue:** The plan's Task 2 file list omitted `src/core/mod.rs`, but the new `app` and `config` files could not compile or be imported from `main` without exporting them.
- **Fix:** Added `pub mod app;` and `pub mod config;` to `src/core/mod.rs` as part of the red-phase test scaffolding.
- **Files modified:** `src/core/mod.rs`
- **Verification:** `cargo test config -- --nocapture`
- **Committed in:** `068b52f`

**2. [Rule 3 - Blocking] Fixed clippy verification by deriving enum defaults**
- **Found during:** Plan-level verification
- **Issue:** `cargo clippy --all-targets -- -D warnings` rejected manual `Default` impls for the config enums.
- **Fix:** Replaced manual impls with `#[derive(Default)]` and `#[default]` on the correct enum variants.
- **Files modified:** `src/core/config.rs`
- **Verification:** `cargo test --quiet` and `cargo clippy --all-targets -- -D warnings`
- **Committed in:** `f4b0ff2`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were required to keep the bootstrap compiling and the validation contract green. No scope creep beyond the plan's foundation seam.

## Issues Encountered

- `cargo clippy --all-targets -- -D warnings` enforced `derivable_impls` on the new enums; the fix was mechanical and did not change behavior.

## Known Stubs

- `config/agent-memos.toml.example:8` and `config/agent-memos.toml.example:9` keep `model` and `endpoint` empty intentionally because Phase 1 must not ship provider credentials or concrete embedding integrations.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `01-02` can build SQLite bootstrap and typed memory entities on top of `AppContext`, `RuntimeReadiness`, and the stable config contract.
- The retrieval-mode contract is locked early, so later lexical and optional embedding work can extend behavior without changing the config shape.

## Self-Check: PASSED

- Verified `.planning/phases/01-foundation-kernel/01-01-SUMMARY.md` exists on disk.
- Verified all plan task commits are present in `git log`.
