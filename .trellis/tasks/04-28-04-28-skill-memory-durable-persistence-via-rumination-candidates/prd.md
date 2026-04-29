# Skill-Memory Durable Persistence Via Rumination Candidates

## Goal

Add the first durable persistence mechanics for skill memory by reusing the existing `rumination_candidates` substrate for `candidate_kind = skill_template`.

This phase should stay internal-only, avoid new tables, and make persisted long-cycle skill candidates reconstructable as explicit `SkillMemoryTemplate` values. It should not add public inspection surfaces or a new parallel storage system.

## What I Already Know

- The explicit internal skill-memory foundation has just been restored:
  - `src/cognition/skill_memory.rs`
  - `WorkingMemoryRequest.skill_templates`
  - projection path `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- The repository already has a durable long-cycle candidate table:
  - `rumination_candidates`
  - `candidate_kind` includes `skill_template`
- Long-cycle rumination already emits skill-template candidates, but the current payload is only a placeholder summary:
  - `template_summary`
  - `trigger_kind`
  - `source_report`
  - `evidence_count`
- That payload is not sufficient to reconstruct a full `SkillMemoryTemplate`.
- The existing theory says long-cycle rumination is the source of skill-memory extraction:
  - `doc/0415-技能记忆.md`
  - `doc/0415-反刍机制.md`
- This makes `rumination_candidates` the correct first durable substrate before considering any dedicated skill-memory snapshot table.

## Assumptions

- The first durable step should reuse the existing `rumination_candidates` table instead of introducing a new table.
- The first read-model can remain candidate-oriented and internal-only.
- Runtime assembly does not have to auto-load persisted skill templates in this phase unless that hook stays narrow and obvious.

## Requirements

- Define a persisted skill-template payload contract that is rich enough to reconstruct `SkillMemoryTemplate`.
- Update long-cycle rumination `SkillTemplate` candidate emission to populate that structured payload.
- Add cognition-layer conversion between persisted `RuminationCandidate` skill-template rows and `SkillMemoryTemplate`.
- Add repository helpers that load skill-template candidates in a narrow, typed way.
- Preserve existing `rumination_candidates` table and its generic governance role.
- Preserve evidence lineage and source report metadata where relevant.
- Keep the implementation internal-only: no CLI/HTTP/MCP/UI surface.
- Do not add a new skill-memory table or snapshot in this phase.
- Do not redesign the full extraction policy beyond the minimum payload needed for reconstruction.
- Add focused tests for:
  - long-cycle skill candidate payload shape
  - repository load/filter behavior
  - round-trip or reconstruction into `SkillMemoryTemplate`
  - existing rumination/governance compatibility

## Acceptance Criteria

- [x] A persisted skill-template candidate payload can reconstruct a `SkillMemoryTemplate`.
- [x] Long-cycle rumination writes structured `skill_template` candidate payloads instead of placeholder-only payloads.
- [x] Repository exposes a typed or narrow helper for loading persisted skill-template candidates.
- [x] Existing `rumination_candidates` schema remains unchanged.
- [x] Existing rumination governance tests remain compatible.
- [x] Focused tests cover reconstruction and persistence behavior.
- [x] `cargo fmt --check`, `cargo clippy --tests -- -D warnings`, and `cargo check --tests` pass.

## Completion Notes

- Reused the existing `rumination_candidates` substrate for `candidate_kind = skill_template`; no new SQLite table was added.
- Replaced the placeholder skill-template payload with a structured, versioned payload that can reconstruct `SkillMemoryTemplate`.
- Added typed repository helpers for loading persisted skill-template candidates.
- Added cognition-layer reconstruction from persisted skill-template candidates into `SkillMemoryTemplate`.
- Preserved existing evidence lineage and `source_report` metadata.
- Locked the durable boundary down with explicit version handling:
  - shared `SKILL_TEMPLATE_PAYLOAD_VERSION`
  - fail-closed rejection for legacy placeholder rows
  - fail-closed rejection for unsupported payload versions
- Added/updated regression coverage for:
  - repository load/filter behavior
  - legacy placeholder rejection
  - unsupported payload version rejection
  - long-cycle rumination skill-template candidate emission
  - reconstruction into `SkillMemoryTemplate`
- Kept this phase intentionally narrow:
  - no new schema
  - no external CLI/HTTP/MCP/UI surface
  - no automatic runtime promotion into active `skill_templates`

## Definition Of Done

- Production code updated.
- Tests added/updated.
- No new SQLite table added.
- No external surface added.
- Trellis check passes after implementation.

## Out Of Scope

- Dedicated skill-memory snapshot tables.
- Auto-promotion from candidate to active runtime `skill_templates` in working-memory assembly unless a tiny helper is strictly needed.
- Full long-cycle extraction intelligence redesign.
- UI/CLI/MCP/API exposure.

## Technical Notes

- Primary files likely affected:
  - `src/cognition/skill_memory.rs`
  - `src/cognition/rumination.rs`
  - `src/memory/repository.rs`
  - `tests/rumination_governance_integration.rs`
  - `tests/memory_repository_store.rs`
  - `tests/skill_memory_projection.rs`
- Contracts to preserve:
  - `.trellis/spec/backend/skill-memory-contracts.md`
  - `.trellis/spec/backend/self-model-contracts.md`
  - `.trellis/spec/backend/world-model-contracts.md`
  - `.trellis/spec/backend/cognition-retrieval-contracts.md`

## ADR-lite

**Context**: Skill-memory now has an explicit internal cognition seam, but durable persistence should attach to the project's existing long-cycle learning substrate rather than inventing a second storage lane.

**Decision**: Reuse `rumination_candidates` with `candidate_kind = skill_template` as the first durable persistence substrate, and make those persisted candidates reconstructable as typed `SkillMemoryTemplate` values.

**Consequences**:
- Long-cycle rumination becomes the durable source of skill-memory candidates.
- The system avoids unnecessary schema sprawl.
- Later phases can choose when and how persisted skill candidates become active runtime templates.
