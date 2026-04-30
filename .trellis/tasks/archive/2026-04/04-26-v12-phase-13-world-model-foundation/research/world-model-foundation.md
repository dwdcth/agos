# World Model Foundation Notes

## Why this phase exists

The codebase already exposes `world_fragments` in working memory, but those fragments are still materialized directly inside assembly. The theory documents require world model to be a distinct cognition layer instead of only an assembler byproduct.

## Theory constraints from `doc/`

- `world_fragments` are the current foreground slice of the world model, not the entire world model.
- World model should represent current world structure before later phases add predictive behavior.
- T1/T2/T3 truth layers remain the substrate; world model is a usable projection over that substrate.

## Current codebase constraints

- `EvidenceFragment` is already part of the public working-memory output contract.
- Retrieval results already carry citation, trace, score, DSL, and temporal metadata.
- Truth projection already happens during working-memory assembly through repository lookups.
- Agent-search and decision reporting already consume `world_fragments` directly.

## Recommended design for this foundation task

### 1. Add explicit world-model types

Introduce a new cognition module for:

- projected world-model slice
- evidence-backed world fragment entries
- projection helpers back to existing `EvidenceFragment`

### 2. Preserve outward compatibility

Do not replace `EvidenceFragment` yet. Instead, derive it from the new world-model layer so downstream consumers continue to work.

### 3. Re-route fragment materialization through the world-model seam

Do not keep growing inline fragment construction inside `WorkingMemoryAssembler`. The assembler should delegate world projection to the new layer.

### 4. Keep persistence unchanged in this task

No new migrations or tables should be added here. This task is about explicit cognition structure, not a storage redesign.

## Risks to avoid

- Do not break `WorkingMemory.present.world_fragments`.
- Do not mix world-model foundation with simulation or prediction logic.
- Do not invent a second truth/citation/provenance source outside the current retrieval + truth projection path.
