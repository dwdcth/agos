---
phase: 03
slug: truth-layer-governance
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-15
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test truth_governance -- --nocapture` |
| **Full suite command** | `cargo test --tests && cargo clippy --all-targets -- -D warnings` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test truth_governance -- --nocapture`
- **After every plan wave:** Run `cargo test --tests && cargo clippy --all-targets -- -D warnings`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | TRU-01, TRU-02 | T-03-01 / T-03-02 | Additive migration preserves the `memory_records` + FTS authority backbone while introducing typed T3/review/candidate governance state. | integration | `cargo test --test foundation_schema foundation_migration_bootstraps_clean_db -- --nocapture` | `tests/foundation_schema.rs` ✅ Existing; `tests/truth_governance.rs` ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | TRU-01, TRU-02 | T-03-03 / T-03-04 | Repository reads expose typed truth-layer state without changing Phase 2 lexical retrieval/citation behavior. | integration | `cargo test --test truth_governance -- --nocapture && cargo test --test retrieval_cli library_search_returns_citations_and_filter_trace -- --nocapture && cargo clippy --all-targets -- -D warnings` | `tests/truth_governance.rs` ❌ W0; `tests/retrieval_cli.rs` ✅ Existing; `tests/ingest_pipeline.rs` ✅ Existing | ⬜ pending |
| 03-02-01 | 02 | 2 | TRU-02, TRU-03 | T-03-05 / T-03-08 | Promotion reviews track evidence and gate states explicitly, and invalid or revoked T3 inputs never bypass governance. | integration | `cargo test --test truth_governance promotion_review_tracks_gate_states_without_promoting -- --nocapture` | `tests/truth_governance.rs` ✅ Existing after `03-01-01` | ⬜ pending |
| 03-02-02 | 02 | 2 | TRU-03 | T-03-05 / T-03-06 / T-03-07 | Shared-truth promotion requires all gate states to pass and creates a derived T2 row instead of mutating the source T3 row. | integration | `cargo test --test truth_governance t3_promotion_requires_all_gate_checks -- --nocapture` | `tests/truth_governance.rs` ✅ Existing | ⬜ pending |
| 03-03-01 | 03 | 3 | TRU-04 | T-03-09 / T-03-10 | T2 -> T1 requests stay candidate-only and never rewrite T1 automatically. | integration | `cargo test --test truth_governance t2_to_t1_creates_candidate_without_t1_mutation -- --nocapture` | `tests/truth_governance.rs` ✅ Existing | ⬜ pending |
| 03-03-02 | 03 | 3 | TRU-01, TRU-04 | T-03-11 / T-03-12 | Governance queue APIs stay explicit, typed, and separate from ordinary retrieval so candidate state does not leak into shared search results. | integration | `cargo test --test truth_governance -- --nocapture` | `tests/truth_governance.rs` ✅ Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/foundation_schema.rs` already exists from earlier phases and remains the migration/bootstrap regression harness for schema version 4.
- [ ] `tests/truth_governance.rs` must be created by Plan `03-01` Task 1 before any truth-governance task-specific verification commands run.
- [x] `tests/retrieval_cli.rs` already exists and is reused to prove Phase 2 lexical retrieval/citation compatibility on the upgraded schema.
- [x] `tests/ingest_pipeline.rs` already exists; Plan `03-01` Task 2 clears the known unused-import clippy blocker so the full Nyquist suite can run cleanly.
- [x] No external test framework install is required beyond the Rust toolchain already verified in `03-RESEARCH.md`.

---

## Manual-Only Verifications

All Phase 03 behaviors have automated verification. Any manual database inspection or CLI spot-check remains optional confidence work, not a Nyquist dependency.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** revised 2026-04-15
