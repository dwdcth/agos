---
phase: 09
slug: dual-channel-retrieval-fusion
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `09-01`, `09-02`, `09-03`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test dual_channel_retrieval -- --nocapture` |
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
| 09-01-01 | 01 | 1 | DCR-01 / DCR-04 | T-09-01 | Retrieval-relevant config parsing can derive lexical-only, embedding-only, and hybrid runtime variants from a shared config contract. | integration | `cargo test --test dual_channel_retrieval parses_root_config_into_dual_channel_variants -- --nocapture` | `tests/dual_channel_retrieval.rs` ❌ W0 | ⬜ pending |
| 09-01-02 | 01 | 1 | DCR-01 | T-09-01 | Generated config variants drive three explicit retrieval-mode test paths instead of unrelated ad hoc fixtures. | integration | `cargo test --test dual_channel_retrieval generated_mode_matrix_covers_lexical_embedding_and_hybrid -- --nocapture` | `tests/dual_channel_retrieval.rs` ✅ Existing after `09-01-01` | ⬜ pending |
| 09-02-01 | 02 | 2 | DCR-01 / DCR-03 | T-09-02 | Ordinary retrieval can run lexical-first plus embedding second-channel recall and dedupe on record identity. | integration | `cargo test --test dual_channel_retrieval hybrid_search_merges_lexical_and_embedding_candidates_by_record_identity -- --nocapture` | `tests/dual_channel_retrieval.rs` ✅ Existing | ⬜ pending |
| 09-02-02 | 02 | 2 | DCR-02 | T-09-02 | Lexical-only behavior remains stable while embedding-only and hybrid modes use the semantic substrate appropriately. | integration | `cargo test --test dual_channel_retrieval mode_specific_search_behaviors_match_generated_configs -- --nocapture` | `tests/dual_channel_retrieval.rs` ✅ Existing | ⬜ pending |
| 09-03-01 | 03 | 3 | DCR-04 | T-09-03 | Final result traces can explain lexical-only, embedding-only, or dual-channel contribution. | integration | `cargo test --test dual_channel_retrieval result_trace_reports_channel_contribution -- --nocapture` | `tests/dual_channel_retrieval.rs` ✅ Existing | ⬜ pending |
| 09-03-02 | 03 | 3 | DCR-02 / DCR-03 / OPS-03 | T-09-03 | Agent-search consumers can still rely on the ordinary retrieval contract after dual-channel trace/rerank extensions. | integration | `cargo test --test retrieval_cli -- --nocapture` | `tests/retrieval_cli.rs` ✅ Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Existing retrieval integration tests already cover lexical-only behavior.
- [ ] `tests/dual_channel_retrieval.rs` will be created in Plan 09-01 as the focused dual-channel/config-matrix regression harness.
- [x] `tests/retrieval_cli.rs` remains the compatibility anchor for the ordinary retrieval contract.

---

## Manual-Only Verifications

All Phase 09 behaviors should be covered by automated tests. Manual ranking inspection is confidence work only.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-16
