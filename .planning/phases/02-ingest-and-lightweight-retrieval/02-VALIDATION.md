---
phase: 02
slug: ingest-and-lightweight-retrieval
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-15
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --quiet` |
| **Full suite command** | `cargo test && cargo clippy --all-targets -- -D warnings` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --quiet`
- **After every plan wave:** Run `cargo test && cargo clippy --all-targets -- -D warnings`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | ING-01, ING-02 | T-02-02 | Additive ingest schema preserves authority rows, chunk provenance, and nullable validity windows without semantic tables. | integration | `cargo test --test foundation_schema -- --nocapture` | ✅ Existing | ⬜ pending |
| 02-01-02 | 01 | 1 | ING-01, ING-02 | T-02-01 / T-02-03 | Detect-normalize-chunk-persist accepts supported text inputs and bounds chunk generation while preserving anchors and source linkage. | integration | `cargo test --test ingest_pipeline -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | ING-03 | T-02-04 / T-02-06 | `libsimple` bootstrap and additive lexical sidecar migrate cleanly and remain rebuildable from authority rows. | integration | `cargo test --test foundation_schema -- --nocapture` | ✅ Existing | ⬜ pending |
| 02-02-02 | 02 | 2 | RET-01, RET-02 | T-02-04 / T-02-05 / T-02-06 | Lexical candidate recall and scoring stay parameterized, deterministic, and status readiness is asserted through the canonical status CLI contract. | integration | `cargo test --test lexical_search --test status_cli -- --nocapture` | `tests/status_cli.rs` ✅ Existing; `tests/lexical_search.rs` ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 3 | RET-03, RET-04, RET-05 | T-02-08 / T-02-09 | Ranked results expose structured citations, applied filters, score breakdowns, and explicit validity metadata. | integration | `cargo test --test retrieval_cli library_search_returns_citations_and_filter_trace -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 3 | AGT-01 | T-02-07 | CLI ingest/search remain thin wrappers over library services and require no Rig or LLM runtime. | integration | `cargo test --test retrieval_cli -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/foundation_schema.rs` already exists from Phase 1 and is reused for additive schema checks in Plans `02-01` and `02-02`.
- [ ] `tests/ingest_pipeline.rs` must be created by Plan `02-01` Task 2 before any ingest-specific verification commands run.
- [ ] `tests/lexical_search.rs` must be created by Plan `02-02` Task 2 before lexical recall and scoring verification runs.
- [x] `tests/status_cli.rs` already exists and remains the canonical readiness/status contract; Plan `02-02` extends it for lexical capability and deferred semantic states.
- [ ] `tests/retrieval_cli.rs` must be created by Plan `02-03` Task 1 before CLI/library end-to-end retrieval checks run.
- [x] No external test framework install is required beyond the Rust toolchain already verified in `02-RESEARCH.md`.

---

## Manual-Only Verifications

All phase behaviors have automated verification. Any manual CLI spot-checks in plan prose are optional confidence checks, not Nyquist dependencies.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** revised 2026-04-15
