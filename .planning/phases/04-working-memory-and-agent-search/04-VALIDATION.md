---
phase: 04
slug: working-memory-and-agent-search
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `04-01`, `04-02`, `04-03`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `rtk cargo test --test working_memory_assembly -- --nocapture` |
| **Full suite command** | `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings` |
| **Estimated runtime** | ~75 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-specific automated command from the plan.
- **After every plan wave:** Run `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 75 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | COG-01, COG-02 | T-04-03 | Immutable working-memory and branch contracts enforce the locked Phase 4 action labels and reject incomplete builder state. | integration | `rtk cargo test --test working_memory_assembly working_memory_builder_requires_present_frame_and_uses_phase4_action_labels -- --nocapture` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | COG-01 | T-04-01 / T-04-02 | Assembly preserves citations/truth context, populates `self_state` from request-local task context + readiness flags + selected truth facts through the minimal provider seam, and keeps runtime working memory in-memory only. | integration | `rtk cargo test --test working_memory_assembly assembler_preserves_citations_truth_context_and_in_memory_runtime_only -- --nocapture` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 2 | COG-03 | T-04-04 | Five-dimension value scoring stays vector-first, uses dynamic typed weights, and keeps all three action kinds comparable on one scoring surface. | integration | `rtk cargo test --test value_metacog value_scorer_projects_five_dimensions_with_dynamic_weights -- --nocapture` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 2 | COG-04 | T-04-05 / T-04-06 | Metacognitive gates keep `warning`, `soft_veto`, `hard_veto`, and `escalate` behavior distinct with structured reports. | integration | `rtk cargo test --test value_metacog metacog_gates_warn_veto_and_escalate_with_typed_reports -- --nocapture` | ✅ Existing after `04-02-01` | ⬜ pending |
| 04-03-01 | 03 | 3 | AGT-02 | T-04-08 / T-04-09 | Agent-search orchestration performs bounded retrieve -> assemble -> score -> gate over internal services and returns a structured cited report. | integration | `rtk cargo test --test agent_search orchestrator_reuses_internal_services_and_returns_structured_report -- --nocapture` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 3 | AGT-03, AGT-04 | T-04-07 / T-04-08 | Rig stays a thin adapter, preserves citations and gate state, and never bypasses search/governance boundaries or introduce write-back. | integration | `rtk cargo test --test agent_search rig_adapter_stays_thin_and_never_bypasses_search_or_truth_gates -- --nocapture` | ✅ Existing after `04-03-01` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/working_memory_assembly.rs` is explicitly owned by Plan `04-01` Task 1 and reused by Plan `04-01` Task 2.
- [x] `tests/value_metacog.rs` is explicitly owned by Plan `04-02` Task 1 and reused by Plan `04-02` Task 2.
- [x] `tests/agent_search.rs` is explicitly owned by Plan `04-03` Task 1 and reused by Plan `04-03` Task 2.
- [x] `src/cognition/mod.rs`, `src/agent/mod.rs`, and `src/lib.rs` remain the module export seams that make the new test files compilable during the same phase.
- [x] Plan `04-01` fixes `self_state` to the minimal `SelfStateProvider` shape: request-local task context, agent capability/readiness flags, and selected truth/governance facts only.
- [x] Live Rig smoke is optional only; Nyquist coverage depends on deterministic local tests and does not require provider API keys.

---

## Manual-Only Verifications

- Optional live Rig smoke may be run only when provider credentials are configured:
  `rtk cargo test --test agent_search -- --ignored live_rig_smoke_requires_provider_env`
- This optional smoke is a confidence check, not a required Nyquist dependency for Phase 4 completion.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** revised 2026-04-16
