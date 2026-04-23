---
phase: 08
slug: embedding-backend-and-index-foundation
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 08 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `08-01`, `08-02`, `08-03`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test status_cli -- --nocapture` |
| **Full suite command** | `cargo test --tests && cargo clippy --all-targets -- -D warnings` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-specific automated command from the plan.
- **After every plan wave:** Run `cargo test --tests && cargo clippy --all-targets -- -D warnings`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | EMB-01 | T-08-01 | Config and diagnostics expose real embedding backend states without changing lexical-only defaults. | integration | `cargo test --test status_cli embedding_foundation_status_reports_backend_readiness_truthfully -- --nocapture` | `tests/status_cli.rs` ✅ Existing | ⬜ pending |
| 08-01-02 | 01 | 1 | EMB-01 | T-08-01 | `doctor` / operational diagnostics keep impossible/reserved-state semantics truthful after embedding backend extensions. | integration | `cargo test --test status_cli embedding_foundation_doctor_preserves_lexical_first_contract -- --nocapture` | `tests/status_cli.rs` ✅ Existing | ⬜ pending |
| 08-02-01 | 02 | 2 | EMB-02 | T-08-02 | Ingest persists chunk-aligned embeddings additively without changing authority-row ownership. | integration | `cargo test --test ingest_pipeline ingest_persists_chunk_aligned_embedding_sidecars -- --nocapture` | `tests/ingest_pipeline.rs` ✅ Existing | ⬜ pending |
| 08-02-02 | 02 | 2 | EMB-02 | T-08-02 | Embedding persistence can remain absent/disabled without breaking lexical-only ingest behavior. | integration | `cargo test --test ingest_pipeline lexical_only_ingest_remains_usable_when_embeddings_are_disabled -- --nocapture` | `tests/ingest_pipeline.rs` ✅ Existing | ⬜ pending |
| 08-03-01 | 03 | 3 | EMB-03 | T-08-03 | Additive schema/bootstrap creates embedding tables and optional vector-sidecar readiness state without replacing the authority store. | integration | `cargo test --test foundation_schema embedding_foundation_schema_bootstraps_additive_vector_sidecars -- --nocapture` | `tests/foundation_schema.rs` ✅ Existing | ⬜ pending |
| 08-03-02 | 03 | 3 | EMB-03 / OPS-01 | T-08-03 | Status inspection can detect missing vs ready vector-sidecar state truthfully after schema bootstrap. | integration | `cargo test --test status_cli embedding_foundation_status_reports_vector_sidecar_state -- --nocapture` | `tests/status_cli.rs` ✅ Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Existing Rust integration test harness already covers config/status, schema, and ingest seams.
- [x] `tests/status_cli.rs` is the right anchor for backend/readiness/operator-surface checks.
- [x] `tests/foundation_schema.rs` is the right seam for additive embedding schema assertions.
- [x] `tests/ingest_pipeline.rs` is the right seam for chunk-aligned embedding persistence checks.

---

## Manual-Only Verifications

All Phase 08 behaviors should be covered by automated tests. Manual inspection of DB contents is confidence work only.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-16
