---
phase: 05
slug: rumination-and-adaptive-write-back
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned plans: `05-01`, `05-02`, `05-03`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `rtk cargo test --test rumination_queue -- --nocapture` |
| **Full suite command** | `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-specific automated command from the plan.
- **After every plan wave:** Run `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | LRN-01 | T-05-02 / T-05-03 | Migration adds explicit `SPQ` and `LPQ` durable queues plus throttle and candidate side tables without mutating the authority backbone. | integration | `rtk cargo test --test foundation_schema rumination_schema_bootstraps_version_5_side_tables -- --nocapture` | `tests/foundation_schema.rs` ✅ Existing; `tests/rumination_queue.rs` ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | LRN-01 | T-05-01 / T-05-02 / T-05-03 | Trigger routing follows `route -> dedupe -> cooldown -> budget -> enqueue`, persists throttle state durably, and always claims `SPQ` before `LPQ`. | integration | `rtk cargo test --test rumination_queue -- --nocapture` | `tests/rumination_queue.rs` ✅ Existing after `05-01-01` | ⬜ pending |
| 05-02-01 | 02 | 2 | LRN-02 | T-05-05 / T-05-06 | Local adaptation entries stay separate from shared truth and are read back through the self-state overlay seam instead of mutating runtime working memory. | integration | `rtk cargo test --test rumination_writeback short_cycle_overlay_provider_reads_local_adaptations -- --nocapture` | `tests/rumination_writeback.rs` ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 2 | LRN-02 | T-05-04 / T-05-06 | Short-cycle processing drains `SPQ` into local-only `self_state`, `risk_boundary`, and private T3-adjacent adaptations while forbidding shared T1/T2 mutation. | integration | `rtk cargo test --test rumination_writeback short_cycle_writeback_updates_local_state_without_mutating_shared_truth -- --nocapture` | `tests/rumination_writeback.rs` ✅ Existing after `05-02-01` | ⬜ pending |
| 05-03-01 | 03 | 3 | LRN-03 | T-05-08 / T-05-09 | `LPQ` produces only `skill_template`, `promotion_candidate`, and `value_adjustment_candidate` rows under one candidate contract with evidence lineage. | integration | `rtk cargo test --test rumination_governance_integration lpq_generates_unified_candidates_from_accumulated_evidence -- --nocapture` | `tests/rumination_governance_integration.rs` ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 3 | LRN-03 | T-05-07 / T-05-08 / T-05-09 | Shared-truth-facing long-cycle outputs route through `TruthGovernanceService`, surface pending review/candidate objects, and never auto-approve themselves. | integration | `rtk cargo test --test rumination_governance_integration lpq_bridges_to_governance_without_auto_approval -- --nocapture` | `tests/rumination_governance_integration.rs` ✅ Existing after `05-03-01` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/foundation_schema.rs` already exists and remains the migration/bootstrap regression harness for schema version 5.
- [x] `tests/rumination_queue.rs` is explicitly owned by Plan `05-01` Task 1 and then reused by Plan `05-01` Task 2.
- [x] `tests/rumination_writeback.rs` is explicitly owned by Plan `05-02` Task 1 and then reused by Plan `05-02` Task 2.
- [x] `tests/rumination_governance_integration.rs` is explicitly owned by Plan `05-03` Task 1 and then reused by Plan `05-03` Task 2.
- [x] `src/cognition/mod.rs`, `src/cognition/assembly.rs`, `src/cognition/rumination.rs`, and `src/memory/repository.rs` remain the module and repository seams that make the new Phase 5 test files compilable within the same phase.
- [x] Nyquist coverage for session-boundary and idle-window triggers relies on explicit service/API capture plus optional CLI sweep, not on a daemon or external scheduler.
- [x] Nyquist coverage for `skill_template` stops at candidate-first durable rows; no executor, retrieval plugin, or background skill subsystem is required in Phase 5.

---

## Manual-Only Verifications

All Phase 05 behaviors have automated verification. Optional manual database inspection remains confidence work only and is not required for Nyquist compliance.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** revised 2026-04-16
