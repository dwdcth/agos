# Self Model Foundation Notes

## Why this phase exists

The codebase already has runtime `self_state` output, but it is still assembled from scattered request fields and local adaptation overlays. The theory documents require self model to be a distinct layer rather than a byproduct of working-memory assembly.

## Theory constraints from `doc/`

- Self model is a control model, not a personality wrapper.
- It should distinguish slower-changing stable capability boundaries from fast-changing runtime status.
- `self_state` in working memory is only a snapshot, not the full self model.
- Short-cycle rumination should update self/risk state, making self model a central write-back target.

## Current codebase constraints

- `SelfStateSnapshot` is already part of the `WorkingMemory` contract.
- `WorkingMemoryRequest` already carries task context, capability flags, readiness flags, active goal/risk, and local adaptation entries.
- `AdaptiveSelfStateProvider` already overlays `LocalAdaptationEntry` data into `self_state`.
- The repo already persists local adaptation entries and uses them in rumination write-back tests.

## Recommended design for this foundation task

### 1. Add explicit self-model types

Introduce a new cognition module for:

- stable self-profile signals
- runtime self-state signals
- projected self-model snapshot

### 2. Preserve outward compatibility

Do not replace `SelfStateSnapshot` yet. Instead, derive it from the new self-model layer so downstream consumers continue to work.

### 3. Re-route overlays through the self-model seam

Do not invent a second local adaptation flow. The current rumination overlay path should become one input to the self-model projection.

### 4. Keep persistence unchanged in this task

No new migrations or tables should be added here. This task is about making cognition structure explicit, not yet about redesigning storage.

## Risks to avoid

- Do not break `WorkingMemory` report shape.
- Do not mix self-model refactor with world-model or skill-memory work.
- Do not create a new persistence model before the cognition seam is stable.
- Do not bypass existing local adaptation repository paths.
