---
phase: 06
slug: runtime-gate-enforcement
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `06-01`, `06-02`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test runtime_gate_cli -- --nocapture` |
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
| 06-01-01 | 01 | 1 | FND-01 / FND-03 / AGT-01 | T-06-01 / T-06-02 | Cross-command CLI tests prove `ingest`, `search`, and `agent-search` do not execute under reserved or impossible runtime combinations. | integration | `cargo test --test runtime_gate_cli gated_commands_fail_for_reserved_or_invalid_runtime_modes -- --nocapture` | `tests/runtime_gate_cli.rs` ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | FND-01 / FND-03 / AGT-01 | T-06-01 / T-06-02 / T-06-03 | Shared operational gate stops execution before DB/service work while still allowing lexical-ready commands to proceed. | integration | `cargo test --test runtime_gate_cli gated_commands_succeed_for_ready_lexical_mode -- --nocapture` | `tests/runtime_gate_cli.rs` ✅ Existing after `06-01-01` | ⬜ pending |
| 06-02-01 | 02 | 2 | FND-01 / FND-03 | T-06-02 / T-06-03 | Missing schema, bad local DB files, or missing lexical sidecars block operational commands consistently, with structured diagnostics instead of opaque runtime errors. | integration | `cargo test --test runtime_gate_cli operational_commands_block_when_runtime_readiness_is_not_satisfied -- --nocapture` | `tests/runtime_gate_cli.rs` ✅ Existing | ⬜ pending |
| 06-02-02 | 02 | 2 | FND-03 / AGT-01 | T-06-03 | `status`, `doctor`, and schema inspection preserve their informational contract while operational gate output stays aligned with doctor semantics. | integration | `cargo test --test status_cli diagnostic_commands_remain_informational_while_operational_gate_uses_same_contract -- --nocapture` | `tests/status_cli.rs` ✅ Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Existing Rust test infrastructure already covers CLI process execution through `cargo test`.
- [x] `tests/status_cli.rs` exists and remains the anchor for diagnostic command semantics.
- [ ] `tests/runtime_gate_cli.rs` will be created in Plan `06-01` as the dedicated cross-command regression harness for runtime gate behavior.
- [x] No new framework, daemon, or external service is required for Phase 6 verification.

---

## Manual-Only Verifications

All Phase 06 behaviors have automated verification. Optional manual CLI spot-checks are confidence work only and are not required for Nyquist compliance.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-16
