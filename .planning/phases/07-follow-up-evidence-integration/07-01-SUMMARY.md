---
phase: 07-follow-up-evidence-integration
plan: 01
subsystem: cognition
tags: [rust, working-memory, assembly, follow-up-evidence, tests]
requires:
  - phase: 04-working-memory-and-agent-search
    provides: working-memory assembly and agent-search report seams
provides:
  - additive integrated-evidence input on `WorkingMemoryRequest`
  - assembler support for merged primary + follow-up evidence
  - regression coverage for merged world fragments and branch support
affects: [phase-7, cognition, working-memory, assembly]
tech-stack:
  added: []
  patterns:
    - follow-up evidence enters cognition through one additive assembler input path
    - branch support and present-state evidence are sourced from one merged fragment set
key-files:
  created:
    - .planning/phases/07-follow-up-evidence-integration/07-01-SUMMARY.md
  modified:
    - src/cognition/assembly.rs
    - tests/working_memory_assembly.rs
key-decisions:
  - "Kept `WorkingMemory { present, branches }` unchanged and added an additive integrated-results input to `WorkingMemoryRequest` instead of inventing a second evidence channel."
  - "Merged externally supplied follow-up results with the primary-query search result inside `WorkingMemoryAssembler::assemble(...)`, preserving backward-compatible behavior when no integrated evidence is supplied."
  - "Locked branch-support behavior to the same merged fragment list so follow-up-only evidence can influence both `present.world_fragments` and branch evidence."
patterns-established:
  - "Assembly-level evidence integration should happen before truth projection and fragment materialization, not later in reporting."
  - "When a phase fixes an integration seam, regression tests should inject the exact missing data path directly rather than relying on incidental query recall."
requirements-completed: [COG-01, AGT-02]
duration: 1 plan
completed: 2026-04-16
---

# Phase 7 Plan 1: Follow-up Evidence Integration Summary

**Assembler-side merge of primary and follow-up evidence into real working-memory state**

## Accomplishments

- Extended `WorkingMemoryRequest` and `WorkingMemoryAssembler` with an additive integrated-results input path so follow-up evidence can enter cognition without changing the top-level working-memory contract.
- Made `assemble(...)` merge supplied follow-up results with the primary-query result set before truth projection and fragment materialization.
- Added `tests/working_memory_assembly.rs` regressions proving integrated follow-up evidence appears in `present.world_fragments` and can satisfy branch supporting-evidence references.

## Verification

- `cargo test --test working_memory_assembly -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`

## Deviations from Plan

- The branch-support regression was narrowed to assert the follow-up-only evidence path directly, rather than also requiring a primary-record support lookup in the same assertion. This kept the test focused on the exact Phase 7 gap being repaired.

## Issues Encountered

- Cargo still emits the non-repo `/home/tongyuan/.cargo/config` deprecation warning, but repository code and tests are clean.

## User Setup Required

None.

## Next Phase Readiness

- The assembler seam is now repaired, so Phase 7 Plan 2 can wire that merged evidence through `AgentSearchOrchestrator` and lock report/decision consistency.

## Self-Check: PASSED

- Verified `cargo test --test working_memory_assembly -- --nocapture` passes.
- Verified `cargo clippy --all-targets -- -D warnings` passes.
