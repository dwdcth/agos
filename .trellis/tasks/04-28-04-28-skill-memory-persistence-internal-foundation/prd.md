# Skill-Memory Persistence Internal Foundation

## Goal

Restore the explicit internal skill-memory cognition seam that current persistence work needs before durable storage mechanics can be added safely.

This phase should reintroduce a dedicated `src/cognition/skill_memory.rs` module, reconnect working-memory assembly to explicit skill-memory projection, and preserve the existing outward `WorkingMemory.branches: Vec<ActionBranch>` contract. It remains internal-only and does not add durable skill-memory tables yet.

## What I Already Know

- The user chose ordering:
  1. world-model persistence
  2. skill-memory persistence
- The codebase already has skill-memory theory and executable contract docs:
  - `doc/0415-技能记忆.md`
  - `.trellis/spec/backend/skill-memory-contracts.md`
- The archived phase for skill-memory foundation exists:
  - `.trellis/tasks/archive/2026-04/04-26-v12-phase-14-skill-memory-foundation/prd.md`
- The current worktree does **not** expose:
  - `src/cognition/skill_memory.rs`
  - `pub mod skill_memory;` in `src/cognition/mod.rs`
  - active `WorkingMemoryRequest.skill_templates`
  - active skill-template projection into `ActionSeed`
- This means direct durable skill-memory persistence would currently hard-bind storage work to ad hoc or absent assembly logic.
- World-model persistence just followed the same pattern:
  - restore explicit seam first
  - add durable mechanics second

## Assumptions

- The first skill-memory persistence step should stay internal-only.
- This phase should restore explicit template projection first, not persistence tables.
- Skill-memory should continue projecting through existing `ActionSeed` / `ActionBranch` seams rather than inventing a second branch system.

## Requirements

- Add `src/cognition/skill_memory.rs` with explicit skill-memory types matching the existing spec:
  - `Preconditions`
  - `ActionTemplate`
  - `ExpectedOutcome`
  - `Boundaries`
  - `SkillMemoryTemplate`
  - `SkillProjectionContext`
  - `ProjectedSkillCandidate`
- Export the module from `src/cognition/mod.rs`.
- Extend `WorkingMemoryRequest` to carry `skill_templates`.
- Add a projection path from skill templates into `ActionSeed`.
- Keep branch materialization single-path:
  - skill templates project into `ActionSeed`
  - existing `materialize_branch(...)` still produces `ActionBranch`
- Preserve existing manual `action_seeds`.
- Preserve downstream agent-search, metacog, value, and CLI compatibility.
- Add focused tests for:
  - skill-template projection
  - manual seed compatibility
  - precondition gating
  - blocked-risk suppression
  - downstream compatibility where relevant
- Do not add durable skill-memory tables, snapshots, extraction pipelines, MCP/HTTP/UI surfaces, or world-model prediction logic in this phase.

## Acceptance Criteria

- [x] `src/cognition/skill_memory.rs` exists and is exported.
- [x] `WorkingMemoryRequest` can carry skill templates.
- [x] Matching skill templates project into ordinary `ActionBranch` values via the existing seed/branch path.
- [x] Manual action seeds remain backward-compatible.
- [x] Focused tests cover projection and fail-safe gating.
- [x] Existing working-memory, agent-search, and relevant CLI/retrieval tests pass.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Added `src/cognition/skill_memory.rs` with explicit internal skill-memory types:
  - `Preconditions`
  - `ActionTemplate`
  - `ExpectedOutcome`
  - `Boundaries`
  - `SkillMemoryTemplate`
  - `SkillProjectionContext`
  - `ProjectedSkillCandidate`
- Exported `pub mod skill_memory;` from `src/cognition/mod.rs`.
- Extended `WorkingMemoryRequest` with `skill_templates`.
- Restored the single-path projection flow:
  - `SkillMemoryTemplate -> ActionSeed -> materialize_branch -> ActionBranch`
- Preserved manual `action_seeds` as additive and ordered before skill-generated seeds.
- Added internal-only `ActionSource` tagging for downstream unique same-kind fallback logic while keeping serialized outward payloads unchanged with `#[serde(skip_serializing)]`.
- Added focused regression coverage in:
  - `tests/skill_memory_projection.rs`
  - `tests/agent_search_skill_fallback.rs`
- Kept this phase intentionally narrow:
  - no persistence tables or snapshots
  - no extraction pipeline redesign
  - no external CLI/HTTP/MCP/UI surface
  - no world-model persistence changes
- Verification passed in implementation and independent Trellis check:
  - `cargo fmt --check`
  - `cargo check --tests`
  - `cargo clippy --tests -- -D warnings`
  - `cargo test --test skill_memory_projection`
  - `cargo test --test agent_search_skill_fallback`

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new storage schema added in this phase.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- Skill-memory persistence tables or snapshots.
- Rumination-driven skill extraction redesign.
- World-model persistence changes.
- Prediction/simulation logic.
- CLI/HTTP/MCP/UI surfaces.

## Technical Notes

- Primary files likely affected:
  - `src/cognition/skill_memory.rs`
  - `src/cognition/mod.rs`
  - `src/cognition/assembly.rs`
  - `tests/skill_memory_projection.rs`
  - `tests/working_memory_assembly.rs`
  - `tests/agent_search.rs`
- Contracts to preserve:
  - `.trellis/spec/backend/skill-memory-contracts.md`
  - `.trellis/spec/backend/world-model-contracts.md`
  - `.trellis/spec/backend/self-model-contracts.md`
  - `.trellis/spec/backend/cognition-retrieval-contracts.md`

## ADR-lite

**Context**: Skill-memory persistence is next in sequence, but the current codebase no longer exposes the explicit internal skill-memory foundation required by the existing spec.

**Decision**: Restore the explicit internal skill-memory projection seam first, while preserving the existing outward `ActionSeed` / `ActionBranch` and `WorkingMemory.branches` contracts.

**Consequences**:
- Durable skill-memory persistence can be added on top of a stable cognition seam.
- Existing downstream consumers remain compatible.
- This phase remains narrow and internal-only.
