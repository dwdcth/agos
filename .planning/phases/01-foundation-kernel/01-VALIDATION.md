---
phase: 01
slug: foundation-kernel
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-15
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` (created by Plan `01-01` Task 1 before any later task-level test commands run) |
| **Quick run command** | `cargo test --quiet` |
| **Full suite command** | `cargo test && cargo clippy --all-targets -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --quiet`
- **After every plan wave:** Run `cargo test && cargo clippy --all-targets -- -D warnings`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | FND-01 | T-01-01 / T-01-02 | Bootstrap stays thin, rejects malformed config input, and introduces no retrieval/vector/agent critical-path dependencies. | build | `cargo check` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | FND-01 | T-01-01 | Config parsing accepts only typed retrieval/backend states, defaults to `lexical_only + disabled`, and preserves explicit reserved-mode semantics. | unit | `cargo test config -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 2 | FND-01 | T-01-04 / T-01-06 | SQLite bootstrap is idempotent, additive, and does not create FTS/vector/Rig tables in Phase 1. | integration | `cargo test --test foundation_schema foundation_migration_bootstraps_clean_db -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 2 | FND-02 | T-01-05 | `MemoryRecord` persistence preserves source, timestamp, scope, record type, truth layer, and provenance without lossy free-form coercion. | integration | `cargo test --test foundation_schema memory_record_round_trip_preserves_foundation_metadata -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 3 | FND-03 | T-01-07 / T-01-08 | `status` always reports configured/effective mode truthfully, and reserved modes remain visible instead of silently downgrading. | integration | `cargo test --test status_cli status_exits_successfully_for_reserved_modes -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 3 | FND-01, FND-03 | T-01-09 | `init`, `doctor`, and inspection commands expose honest readiness/schema state and only fail for invalid or impossible command-path requests. | integration | `cargo test --test status_cli -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `Cargo.toml` plus `src/lib.rs` / `src/main.rs` are explicitly owned by Plan `01-01` Task 1; no later automated command runs before that bootstrap lands.
- [x] `config`-scoped tests are explicitly owned by Plan `01-01` Task 2 and unblock typed config/readiness verification in the same wave.
- [x] `tests/foundation_schema.rs` is explicitly owned by Plan `01-02` Task 1 and reused by Plan `01-02` Task 2 for schema and round-trip coverage.
- [x] `tests/status_cli.rs` is explicitly owned by Plan `01-03` Task 1 and reused by Plan `01-03` Task 2 for command-path and exit-semantics coverage.
- [x] No external test framework install is required beyond the Rust toolchain already verified in `01-RESEARCH.md`.

---

## Manual-Only Verifications

All phase behaviors have automated verification. Manual CLI spot-checks in plan prose are optional confidence checks, not Nyquist dependencies.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-15
