---
phase: 04-working-memory-and-agent-search
plan: 01
subsystem: cognition
tags: [rust, cognition, working-memory, retrieval, truth-layer, tdd]
requires:
  - phase: 02-ingest-and-lightweight-retrieval
    provides: structured SearchResponse values with citations, traces, and lexical-first retrieval filters
  - phase: 03-truth-layer-governance
    provides: typed TruthRecord projections and repository-backed truth/governance reads
provides:
  - immutable WorkingMemory and PresentFrame contracts with a builder that rejects incomplete state
  - shared epistemic/instrumental/regulative action branch typing for Phase 4 decision space
  - working-memory assembler that reuses SearchService and truth projections without persisting runtime state
affects: [phase-4, cognition, working-memory, ordinary-search, truth-governance]
tech-stack:
  added: []
  patterns:
    - immutable working-memory rebuilt per assembly cycle
    - provider-built self-state over request context plus selected truth facts
    - branch evidence cloned from cited retrieval fragments instead of raw SQL rows
key-files:
  created:
    - src/cognition/action.rs
    - src/cognition/assembly.rs
    - src/cognition/working_memory.rs
    - tests/working_memory_assembly.rs
  modified:
    - src/cognition/mod.rs
    - src/lib.rs
key-decisions:
  - "Kept WorkingMemory runtime-only and immutable, with builder validation preventing partial present-state execution."
  - "Made self_state a minimal provider seam fed by request-local flags and selected TruthRecord projections instead of a new durable self-model subsystem."
  - "Materialized branch evidence directly from cited retrieval fragments so Phase 2 provenance and Phase 3 truth context remain attached inside the control field."
patterns-established:
  - "Cognition assembly should depend on SearchService and MemoryRepository truth projections, never on raw SQL result shaping."
  - "Action branches may be seeded separately from assembly, then hydrated with shared evidence fragments to keep branch typing stable before value scoring arrives."
requirements-completed: [COG-01, COG-02]
duration: 8min
completed: 2026-04-16
---

# Phase 4 Plan 1: Working-Memory Foundation Summary

**Immutable working-memory assembly over cited retrieval and truth projections, with provider-built self state and shared Phase 4 action branches**

## Performance

- **Duration:** 8min
- **Started:** 2026-04-15T16:27:19Z
- **Completed:** 2026-04-15T16:34:52Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added a new public `cognition` module with immutable `WorkingMemory { present, branches }` and builder validation that refuses incomplete present-state assembly.
- Locked Phase 4 action typing to `epistemic`, `instrumental`, and `regulative`, with shared branch shape for cited supporting evidence.
- Implemented a working-memory assembler that reuses `SearchService` plus typed truth projections, preserves citations and truth metadata on fragments, and rebuilds fresh in-memory frames on each call.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define immutable working-memory and action contracts** - `bfd6720` (`test`), `1deb19c` (`feat`)
2. **Task 2: Implement attention-to-working-memory assembly over retrieval and truth seams** - `cb4c27d` (`test`), `94045ed` (`feat`)

**Plan metadata:** Recorded in the final docs commit for this plan.

## Files Created/Modified

- `src/cognition/mod.rs` - exports the new cognition boundary.
- `src/cognition/action.rs` - defines normalized action kinds, candidates, and typed branches.
- `src/cognition/working_memory.rs` - defines present-frame, evidence-fragment, truth-context, and builder-validated working-memory contracts.
- `src/cognition/assembly.rs` - adds the request/provider/assembler seam over existing retrieval and truth services.
- `src/lib.rs` - exposes `cognition` from the library root.
- `tests/working_memory_assembly.rs` - covers builder invariants, terminology lock, citation/truth preservation, provider-built self-state, and in-memory rebuild behavior.

## Decisions Made

- Kept runtime working memory entirely in memory and limited the new types to assembly/debug-friendly value objects, with no repository writes or schema changes.
- Used a `SelfStateProvider` trait as the minimal extension seam so Phase 4 can assemble request-local context and selected truth facts without inventing a persistent self-model.
- Represented branch evidence as cloned cited fragments from `SearchResponse` so branch reasoning stays aligned with lexical-first retrieval and truth-governance provenance.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `rtk cargo clippy --all-targets -- -D warnings` is still blocked by a pre-existing unused `NormalizedSource` import in `tests/ingest_pipeline.rs:12`. Per executor scope rules, that file was left untouched and logged in `.planning/phases/04-working-memory-and-agent-search/deferred-items.md`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `04-02` can build value scoring and metacognitive gating directly on the new immutable `WorkingMemory`, shared action-branch shape, and assembler/provider seams.
- Ordinary lexical retrieval and Phase 3 truth governance remain untouched; Phase 4 cognition now consumes those seams rather than bypassing them.
- The only verification debt carried forward is the pre-existing `tests/ingest_pipeline.rs:12` clippy warning outside this plan’s scope.

## Self-Check: PASSED

- Verified `.planning/phases/04-working-memory-and-agent-search/04-01-SUMMARY.md`, `src/cognition/action.rs`, `src/cognition/assembly.rs`, `src/cognition/working_memory.rs`, and `tests/working_memory_assembly.rs` exist on disk.
- Verified commits `bfd6720`, `1deb19c`, `cb4c27d`, and `94045ed` exist in git history.
- Confirmed the plan's focused verification was green via `rtk cargo test --test working_memory_assembly -- --nocapture`; `rtk cargo clippy --all-targets -- -D warnings` remains blocked only by the pre-existing deferred item in `tests/ingest_pipeline.rs:12`.
