---
phase: 07-follow-up-evidence-integration
plan: 02
subsystem: agent-search
tags: [rust, agent-search, orchestration, working-memory, follow-up-evidence]
requires:
  - phase: 07-follow-up-evidence-integration
    provides: additive assembler-side merged evidence seam from 07-01
provides:
  - orchestration wiring that passes merged follow-up evidence into assembly
  - report/working-memory/decision alignment for follow-up evidence
  - integration regressions proving follow-up evidence affects the decision surface
affects: [phase-7, agent-search, cognition, orchestration]
tech-stack:
  added: []
  patterns:
    - agent-search report trace and runtime cognition now share one evidence set
    - follow-up query provenance remains visible after cognition integration
key-files:
  created:
    - .planning/phases/07-follow-up-evidence-integration/07-02-SUMMARY.md
  modified:
    - src/agent/orchestration.rs
    - tests/agent_search.rs
key-decisions:
  - "Kept the bounded follow-up retrieval loop intact and injected merged results into `WorkingMemoryRequest` instead of creating a second orchestration-only cognition path."
  - "Updated scripted assembler behavior in tests to consume `request.integrated_results`, so report/decision alignment is verified at the same seam used by production code."
  - "Locked Phase 7 on the invariant that follow-up-only evidence can appear simultaneously in `retrieval_steps`, top-level citations, `working_memory`, and selected-branch support."
patterns-established:
  - "If agent-search reports evidence as materially relevant, orchestration must feed that evidence into assembly before scoring and gate evaluation."
  - "Integration tests should assert on `working_memory`, citations, and decision output together when fixing cognition/report divergence."
requirements-completed: [AGT-02, AGT-03, COG-01]
duration: 1 plan
completed: 2026-04-16
---

# Phase 7 Plan 2: Follow-up Evidence Integration Summary

**Orchestration-side wiring of merged follow-up evidence into working memory, report, and decision selection**

## Accomplishments

- Updated `AgentSearchOrchestrator::execute(...)` to accumulate retrieved results across primary and follow-up queries, then pass them back into assembly through the new additive `WorkingMemoryRequest` seam.
- Added `tests/agent_search.rs` regressions proving follow-up-only evidence appears in `working_memory.present.world_fragments`, remains visible in top-level citations and `retrieval_steps`, and influences the selected branch support surface.
- Closed the remaining audit mismatch between “reported evidence” and “used evidence” for agent-search.

## Verification

- `cargo test --test agent_search -- --nocapture`
- `cargo test --tests`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

- The scripted test assembler was upgraded to consume `request.integrated_results` instead of staying completely static. This was required so the orchestration tests could verify the real Phase 7 integration seam rather than merely asserting on report metadata.

## Issues Encountered

- Cargo still emits the non-repo `/home/tongyuan/.cargo/config` deprecation warning, but repository code and tests are clean.

## User Setup Required

None.

## Next Phase Readiness

- Phase 7 is now execution-complete: follow-up evidence is integrated into working memory and the decision surface, so the milestone gap-closure work is ready for transition, verification/UAT, and milestone re-audit.

## Self-Check: PASSED

- Verified `cargo test --test agent_search -- --nocapture` passes.
- Verified `cargo test --tests` passes.
- Verified `cargo clippy --all-targets -- -D warnings` passes.
