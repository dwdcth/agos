---
phase: 07
slug: follow-up-evidence-integration
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `07-01`, `07-02`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test agent_search -- --nocapture` |
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
| 07-01-01 | 01 | 1 | COG-01 / AGT-02 | T-07-01 / T-07-02 | Follow-up retrieval evidence is merged into `WorkingMemory.present.world_fragments` and remains citation-bearing instead of staying report-only. | integration | `cargo test --test working_memory_assembly assembler_integrates_follow_up_evidence_into_world_fragments -- --nocapture` | `tests/working_memory_assembly.rs` ✅ Existing | ⬜ pending |
| 07-01-02 | 01 | 1 | COG-01 / AGT-02 | T-07-01 | Branch supporting evidence can reference integrated follow-up fragments through the same runtime evidence list used by present-state assembly. | integration | `cargo test --test working_memory_assembly assembler_uses_integrated_follow_up_fragments_for_branch_support -- --nocapture` | `tests/working_memory_assembly.rs` ✅ Existing | ⬜ pending |
| 07-02-01 | 02 | 2 | AGT-02 / AGT-03 | T-07-02 / T-07-03 | Agent-search orchestration returns reports whose `working_memory`, `decision`, and citations all reflect the same merged follow-up evidence set. | integration | `cargo test --test agent_search orchestrator_integrates_follow_up_evidence_into_working_memory_and_report -- --nocapture` | `tests/agent_search.rs` ✅ Existing | ⬜ pending |
| 07-02-02 | 02 | 2 | AGT-03 / COG-01 | T-07-03 | Scoring and gate results remain aligned with integrated follow-up evidence rather than only the primary-query evidence. | integration | `cargo test --test agent_search integrated_follow_up_evidence_influences_decision_surface -- --nocapture` | `tests/agent_search.rs` ✅ Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Existing Rust integration test harness already covers working-memory assembly and agent-search seams.
- [x] `tests/working_memory_assembly.rs` exists and is the right seam for assembler-level evidence integration checks.
- [x] `tests/agent_search.rs` exists and is the right seam for orchestration/report/decision consistency checks.
- [x] No new framework, daemon, or external service is required for Phase 07 verification.

---

## Manual-Only Verifications

All Phase 07 behaviors should be covered by automated tests. Manual CLI inspection is confidence work only and is not required for Nyquist compliance.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-16
