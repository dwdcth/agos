---
phase: 10
slug: dual-channel-diagnostics-and-service-compatibility
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `10-01`, `10-02`.

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
| 10-01-01 | 01 | 1 | OPS-01 / OPS-02 | T-10-01 | Status/doctor make dual-channel readiness and gating operator-readable without hiding lexical-only stability. | integration | `cargo test --test status_cli dual_channel_status_and_doctor_report_mode_compatibility_truthfully -- --nocapture` | `tests/status_cli.rs` ✅ Existing | ⬜ pending |
| 10-01-02 | 01 | 1 | OPS-02 | T-10-01 | Search CLI can run lexical-only / embedding-only / hybrid mode controls without breaking lexical-only defaults. | integration | `cargo test --test retrieval_cli search_surface_respects_dual_channel_mode_selection -- --nocapture` | `tests/retrieval_cli.rs` ✅ Existing | ⬜ pending |
| 10-02-01 | 02 | 2 | OPS-03 | T-10-02 | Agent-search still reuses ordinary retrieval services under explicit dual-channel mode selection. | integration | `cargo test --test agent_search agent_search_reuses_ordinary_retrieval_under_dual_channel_modes -- --nocapture` | `tests/agent_search.rs` ✅ Existing | ⬜ pending |
| 10-02-02 | 02 | 2 | OPS-03 / OPS-02 | T-10-02 | Agent-search output remains compatible with follow-up evidence integration and metacognitive decision flow after dual-channel mode plumbing. | integration | `cargo test --test agent_search dual_channel_mode_selection_preserves_agent_report_contract -- --nocapture` | `tests/agent_search.rs` ✅ Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Existing operator-surface and retrieval/agent-search integration tests already exist.
- [x] `tests/status_cli.rs`, `tests/retrieval_cli.rs`, and `tests/agent_search.rs` are the correct seams for Phase 10.
- [x] No new framework or external service is required for Phase 10 verification.

---

## Manual-Only Verifications

All Phase 10 behaviors should be covered by automated tests. Manual operator review is confidence work only.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-16
