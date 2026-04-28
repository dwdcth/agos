# Skill Memory Contracts

> Executable contracts for explicit skill-memory projection inside cognition and working-memory assembly.

---

## Scenario: Skill Memory Foundation Projection

### 1. Scope / Trigger

- Trigger: Phase 14 introduced an explicit skill-memory foundation so reusable procedural templates can project into working-memory candidate actions without replacing the existing manual action-seed path.
- Why this needs code-spec depth: this is a cross-layer contract change touching cognition module structure, working-memory assembly, branch materialization, and downstream agent-search compatibility.

### 2. Signatures

- `src/cognition/skill_memory.rs`
  - `Preconditions`
  - `ActionTemplate`
  - `ExpectedOutcome`
  - `Boundaries`
  - `SkillMemoryTemplate`
  - `SkillProjectionContext`
  - `ProjectedSkillCandidate`
- `src/cognition/assembly.rs`
  - `WorkingMemoryRequest { ..., action_seeds: Vec<ActionSeed>, skill_templates: Vec<SkillMemoryTemplate>, ... }`
  - `WorkingMemoryRequest::with_skill_template(skill_template)`
  - `project_skill_action_seeds(request, world_fragments) -> Vec<ActionSeed>`
  - `materialize_branch(seed, world_fragments) -> Result<ActionBranch, WorkingMemoryAssemblyError>`
- `src/cognition/working_memory.rs`
  - outward compatibility stays on `WorkingMemory.branches: Vec<ActionBranch>`

### 3. Contracts

#### Projection contract

- Skill memory is an internal cognition seam, not a new outward working-memory payload.
- `SkillMemoryTemplate` projects into `ActionSeed` first, then uses the existing `materialize_branch(...)` path.
- `WorkingMemory.branches` must remain `Vec<ActionBranch>` with no contract drift for downstream consumers.

#### Compatibility contract

- Manual `action_seeds` remain valid and must preserve their existing branch materialization behavior.
- Skill-generated candidates append to, rather than replace, manual `action_seeds`.
- Existing agent-search, value scoring, metacognitive gating, CLI rendering, and rumination consumers must continue reading ordinary `ActionBranch` values.
- Agent-search value scoring may satisfy a skill-generated branch with:
  - an exact `AgentSearchBranchValue` match on `kind + summary`, or
  - a unique same-`kind` value template when no exact summary match exists
- A same-`kind` fallback is only valid when exactly one value template exists for that `ActionKind`; otherwise the existing missing-value failure remains the safe behavior.

#### Preconditions and boundary contract

- `Preconditions` gates whether a reusable skill template is applicable in the current assembly context.
- `Boundaries` defines:
  - risk markers projected onto the resulting branch
  - optional supporting record ids that the projected branch should cite
  - active-risk conditions that suppress projection
- Skill projection must fail safe:
  - if a template's preconditions do not match, it is skipped
  - if a template's supporting record ids are not present in `world_fragments`, it is skipped
- Skill projection must not create a new branch-materialization error path beyond existing manual seed validation.

#### Adjacent seam contract

- Skill memory may read the assembled world fragments and working-memory request metadata to decide whether a template applies.
- Skill memory must not collapse the separate attention, self-model, or world-model seams back into assembler-local ad hoc logic.
- Future persistence or rumination promotion work may feed `skill_templates`, but this foundation phase does not define that long-cycle storage contract.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| `WorkingMemoryRequest` has only manual `action_seeds` | Branches materialize exactly as before |
| `WorkingMemoryRequest` has matching `skill_templates` | Additional candidate branches are projected through `ActionSeed` and materialized normally |
| A skill template precondition does not match | Template is skipped; assembly still succeeds |
| A skill template requires supporting record ids not present in `world_fragments` | Template is skipped; assembly still succeeds |
| A skill template is blocked by an active risk listed in `Boundaries.blocked_active_risks` | Template is skipped; manual seeds and assembly still succeed |
| Agent-search sees a skill-generated branch with no exact summary value mapping but exactly one `AgentSearchBranchValue` for that `ActionKind` | Reuse that unique same-kind value template and continue into metacognitive gating |
| Agent-search sees a skill-generated branch with no exact summary value mapping and zero or multiple same-kind templates | Preserve the existing missing-value failure instead of guessing |
| Manual `action_seed` references a missing supporting record id | Existing `MissingSupportingRecord` error behavior remains unchanged |
| Downstream agent-search or working-memory consumers inspect branches | They still receive ordinary backward-compatible `ActionBranch` values |
| A projected branch carries soft- or hard-veto risk markers | Existing metacognitive fail-safe gating consumes them through the ordinary `ActionBranch` path |

### 5. Good / Base / Bad Cases

- Good:
  - Introduce new reusable procedural templates in `skill_memory.rs`.
  - Convert skill templates into `ActionSeed` and reuse `materialize_branch(...)`.
  - Add focused tests that cover projection, manual-seed compatibility, and precondition gating.
- Base:
  - Existing code that only uses `with_action_seed(...)` continues working unchanged.
- Bad:
  - Building `ActionBranch` directly from skill templates in a second path.
  - Letting skill projection bypass evidence support, risk markers, or branch-materialization rules.
  - Mixing this foundation work with persistence schema or long-cycle workflow redesign.

### 6. Tests Required

- `tests/skill_memory_projection.rs`
  - Assert matching skill templates project into action branches
  - Assert manual action seeds remain intact when skill templates are also present
  - Assert unmet preconditions skip projection without failing assembly
  - Assert `blocked_active_risks` suppresses skill projection without disturbing manual seeds
  - Assert projected risk markers still trigger downstream fail-safe gating
- `tests/working_memory_assembly.rs`
  - Keep manual action-seed regression coverage green
- `tests/agent_search.rs`
  - Keep orchestration integration coverage green so projected branches still fit existing downstream contracts
  - Assert a skill-generated branch can reuse a unique same-kind value template and still reach metacognitive gating

### 7. Wrong vs Correct

#### Wrong

- Keep candidate-action generation entirely ad hoc in request assembly
- Add a second direct `SkillBranch` output contract
- Make skill projection authoritative over manual seeds

#### Correct

- Introduce explicit skill-memory template types
- Project skill templates into existing `ActionSeed` / `ActionBranch` seams
- Preserve the outward working-memory and agent-search contracts

### Design Decision: Make Skill Memory a Seed-Producing Seam, Not a Parallel Branch System

**Context**: The theory requires explicit skill memory between long-term cognition and foreground planning, but the codebase already has stable downstream contracts built around `ActionSeed` and `ActionBranch`.

**Decision**: Add explicit skill-memory templates internally, but materialize them by projecting into the existing `ActionSeed` seam before branch construction.

**Why**:

- It keeps the action pipeline single-path and explainable.
- It preserves current downstream contracts.
- It leaves room for later persistence or rumination promotion without redoing branch consumers.

**Related files**:

- `src/cognition/skill_memory.rs`
- `src/cognition/assembly.rs`
- `src/cognition/working_memory.rs`
- `tests/skill_memory_projection.rs`
