# v1.2 Phase 15 Cognitive Loop Integration

## Goal

Integrate the explicit attention, self-model, world-model, and skill-memory foundations into one stable cognitive loop:

`Attention -> Retrieval -> World Model -> Self Model -> Working Memory -> Candidate Actions -> Value -> Metacog`

This phase should tighten the end-to-end contracts and regression coverage without changing outward product surfaces, adding new storage schema, or introducing MCP/HTTP interfaces.

## Requirements

- Keep the existing outward surfaces stable:
  - ordinary retrieval response contracts
  - `WorkingMemory`
  - `ActionBranch`
  - agent-search report shapes
- Ensure the four explicit seams now cooperate end to end:
  - attention affects retrieval trace and ranking
  - retrieval/truth projection feeds world-model projection
  - self-model projection feeds `self_state`
  - skill-memory templates feed candidate action generation
- Strengthen integration coverage across these seams in realistic end-to-end flows.
- Keep scope integration-only: no new cognition layer, no schema changes, no provider/API surfaces.

## Acceptance Criteria

- [ ] End-to-end tests prove all four explicit seams participate in one stable cognitive loop.
- [ ] Existing outward contracts remain backward compatible.
- [ ] Cross-seam regression coverage exists for retrieval trace, world fragments, self state, skill-generated branches, and downstream metacognitive gating.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Out of Scope

- New persistence schema
- New CLI/API/MCP surfaces
- Simulation/prediction engine work
- Long-cycle persistence/productization for skill memory

## Technical Notes

- This phase should prefer integration glue, validation helpers, and regression coverage over adding another concept layer.
- Existing contract specs that must remain true:
  - `.trellis/spec/backend/cognition-retrieval-contracts.md`
  - `.trellis/spec/backend/self-model-contracts.md`
  - `.trellis/spec/backend/world-model-contracts.md`
  - `.trellis/spec/backend/skill-memory-contracts.md`
