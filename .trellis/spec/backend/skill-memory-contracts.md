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

---

## Scenario: Durable Skill-Memory Candidates Via Rumination Substrate

### 1. Scope / Trigger

- Trigger: Phase 15 adds the first durable persistence mechanics for skill memory without introducing a dedicated snapshot table.
- Why this needs code-spec depth: the change crosses long-cycle rumination payload shape, repository typed loading, and cognition-layer reconstruction while explicitly preserving the generic `rumination_candidates` governance substrate.

### 2. Signatures

- `src/cognition/rumination.rs`
  - long-cycle `RuminationCandidateKind::SkillTemplate` emission now writes a structured payload instead of a placeholder summary
- `src/memory/repository.rs`
  - `PersistedSkillMemoryTemplatePayload`
  - `PersistedSkillMemoryTemplateCandidate`
  - `MemoryRepository::list_skill_template_candidates() -> Result<Vec<PersistedSkillMemoryTemplateCandidate>, RepositoryError>`
  - `MemoryRepository::list_skill_template_candidates_for_subject(subject_ref) -> Result<Vec<PersistedSkillMemoryTemplateCandidate>, RepositoryError>`
- `src/cognition/skill_memory.rs`
  - `SkillMemoryTemplate::to_candidate_payload(...) -> PersistedSkillMemoryTemplatePayload`
  - `SkillMemoryTemplate::from_rumination_candidate(...) -> Result<SkillMemoryTemplate, SkillMemoryTemplateDecodeError>`

### 3. Contracts

#### Durable substrate contract

- Durable skill-memory persistence reuses `rumination_candidates` with `candidate_kind = skill_template`.
- Do not add a parallel skill-memory table or snapshot for this phase.
- Generic rumination governance behavior remains authoritative for candidate lifecycle, status, timestamps, evidence refs, and queue-item lineage.

#### Payload reconstruction contract

- The persisted payload must be rich enough to reconstruct a `SkillMemoryTemplate` without re-running retrieval.
- The structured payload must carry:
  - `payload_version`
  - `template_id`
  - `template_summary`
  - `preconditions`
  - `action`
  - `expected_outcome`
  - `boundaries`
  - `trigger_kind`
  - `source_report`
  - `evidence_count`
- `source_report` remains the preserved explainability snapshot for long-cycle extraction lineage.
- `evidence_refs` remain stored on the generic `RuminationCandidate` row and must not be folded into a separate table.

#### Cognition reconstruction contract

- Skill-memory reconstruction must happen through explicit conversion from a typed persisted skill candidate into `SkillMemoryTemplate`.
- Unsupported `payload_version` values must fail deterministically through a typed decode error.
- Invalid persisted action kinds must fail deterministically through a typed decode error instead of silently guessing.
- Reconstructed templates remain internal cognition values in this durable-persistence phase; runtime activation is defined separately by the consumed-candidate read-model contract below.

#### Repository helper contract

- Repository helpers must load only `skill_template` candidates and expose typed payloads instead of raw `serde_json::Value`.
- Legacy placeholder-only `skill_template` rows from the pre-structured phase must fail with an explicit repository boundary error instead of an opaque JSON parse failure.
- Subject-scoped filtering belongs in the repository helper seam, not in ad hoc test or service code.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| LPQ emits a skill candidate | Payload stores structured template fields plus `source_report` lineage |
| Repository lists generic rumination candidates | Existing candidate ordering and governance behavior remain compatible |
| Repository lists skill-template candidates | Only `candidate_kind = skill_template` rows are returned |
| Subject filter is applied | Only matching subject rows are returned |
| A stored skill payload uses the old placeholder-only shape | Repository helper fails with an explicit legacy-payload boundary error |
| Persisted `payload_version` is unsupported | Repository helper or cognition reconstruction fails deterministically |
| Persisted action kind is invalid | `SkillMemoryTemplate::from_rumination_candidate(...)` fails with a typed decode error |
| Promotion / ontology governance runs on other candidate kinds | Existing bridging behavior remains unchanged |

### 5. Good / Base / Bad Cases

- Good:
  - Reuse the existing `rumination_candidates` table.
  - Persist a structured skill-template payload that can round-trip into `SkillMemoryTemplate`.
  - Keep source reports and evidence lineage intact.
- Base:
  - Existing promotion and value-adjustment candidates continue using the same generic substrate.
- Bad:
  - Add a new `skill_memory_templates` table for the first durable phase.
  - Store only a human summary string and expect later code to infer the full template.
  - Expose persisted skill candidates through CLI/HTTP/MCP in this phase.

### 6. Tests Required

- `tests/rumination_governance_integration.rs`
  - Assert LPQ skill candidates write the structured payload shape and preserve source-report lineage
- `tests/memory_repository_store.rs`
  - Assert repository typed helpers load only skill-template candidates and filter by subject
- `tests/skill_memory_projection.rs`
  - Assert persisted skill candidates reconstruct into `SkillMemoryTemplate`
- Regression:
  - Keep existing long-cycle governance compatibility coverage green

### 7. Wrong vs Correct

#### Wrong

- Treat skill-template payloads as an opaque summary blob
- Parse raw skill-template JSON ad hoc at each callsite
- Introduce a new storage lane before the candidate substrate is proven

#### Correct

- Define a typed persisted skill-template payload
- Reconstruct `SkillMemoryTemplate` through one explicit cognition seam
- Preserve the generic rumination candidate table and lifecycle semantics

---

## Scenario: Runtime Read Model From Consumed Skill Candidates

### 1. Scope / Trigger

- Trigger: Phase 16 bridges persisted skill-template candidates into runtime working-memory assembly without adding a second activation system.
- Why this needs code-spec depth: the change crosses repository status filtering, cognition reconstruction, and working-memory assembly while preserving the existing `SkillMemoryTemplate -> ActionSeed -> ActionBranch` projection seam.

### 2. Signatures

- `src/memory/repository.rs`
  - `MemoryRepository::list_consumed_skill_template_candidates_for_subject(subject_ref) -> Result<Vec<PersistedSkillMemoryTemplateCandidate>, RepositoryError>`
- `src/cognition/skill_memory.rs`
  - `load_runtime_skill_templates_for_subject(repository, subject_ref) -> Result<Vec<SkillMemoryTemplate>, RuntimeSkillTemplateLoadError>`
  - `merge_runtime_skill_templates(explicit_templates, persisted_templates) -> Vec<SkillMemoryTemplate>`
- `src/cognition/assembly.rs`
  - `WorkingMemoryAssembler::assemble(...) -> Result<WorkingMemory, WorkingMemoryAssemblyError>`
  - runtime loading happens only when `WorkingMemoryRequest.subject_ref` is present

### 3. Contracts

#### Activation contract

- Only persisted skill-template candidates with all of these properties may become runtime templates:
  - `candidate_kind = skill_template`
  - `status = consumed`
  - `subject_ref = WorkingMemoryRequest.subject_ref`
- `Pending`, `Rejected`, and `Archived` skill-template candidates must remain inactive.
- Runtime loading is subject-scoped only; there is no global activation path.

#### Single-path projection contract

- Persisted runtime templates must reconstruct into ordinary `SkillMemoryTemplate` values first.
- Persisted consumed candidates must first compact into an explicit active read model keyed by `template_id` within the current `subject_ref`.
- The active read model resolves duplicate logical templates by latest-wins ordering:
  - later `updated_at`
  - then later `candidate_id`
- Repository-loaded templates must flow through `WorkingMemoryRequest.skill_templates`.
- Branch materialization stays single-path:
  - `SkillMemoryTemplate -> ActionSeed -> ActionBranch`
- Do not add a second direct branch-construction path for persisted templates.

#### Merge contract

- Explicit caller-provided `skill_templates` remain authoritative for the same `template_id`.
- Repository-loaded consumed templates merge additively after explicit request templates.
- Repository-loaded templates may append new runtime options, but they must not overwrite explicit caller-provided templates.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Subject has one consumed `skill_template` candidate | Assembly loads it into runtime `skill_templates` and projects it through the ordinary skill path |
| Subject has multiple consumed `skill_template` candidates with the same `template_id` | Runtime loading resolves them through the active read model and keeps only the latest winner by `updated_at`, then `candidate_id` |
| Subject has pending / rejected / archived `skill_template` candidates | They remain inactive and produce no branches |
| Another subject has consumed `skill_template` candidates | They are ignored |
| Request already carries explicit `skill_templates` | Explicit templates remain present and persisted consumed templates merge additively |
| Persisted consumed candidate payload is invalid | Runtime loading fails deterministically through the typed decode boundary |

### 5. Tests Required

- `tests/memory_repository_store.rs`
  - Assert runtime helper filters by `status = consumed` and `subject_ref`
- `tests/skill_memory_projection.rs`
  - Assert consumed duplicate candidates for the same `template_id` collapse to one runtime template
  - Assert equal-`updated_at` duplicates break ties by `candidate_id`
  - Assert runtime merge preserves explicit templates and adds unique persisted templates
- `tests/working_memory_assembly.rs`
  - Assert assembly loads the active winner for duplicate consumed templates, ignores inactive statuses, and still produces ordinary `ActionBranch` values

### 6. Good / Base / Bad Cases

- Good:
  - Load only consumed subject-scoped `skill_template` candidates into runtime assembly.
  - Reconstruct persisted templates before projection and merge them additively with explicit templates.
- Base:
  - Callers that provide only explicit `skill_templates` keep working without repository participation.
- Bad:
  - Activate `Pending` candidates in runtime just because they exist in the substrate.
  - Build persisted branches through a second runtime-only path.

### 7. Wrong vs Correct

#### Wrong

- Treat every persisted `skill_template` candidate as active.
- Let repository-loaded templates overwrite explicit request templates.

#### Correct

- Load only `Consumed` candidates for the current subject.
- Merge persisted templates after explicit request templates and keep the single projection path.

---

## Scenario: Typed Skill-Template Candidate Lifecycle Bridge

### 1. Scope / Trigger

- Trigger: Phase 17 adds an internal-only lifecycle bridge so persisted `skill_template` candidates can move through `Pending -> Consumed/Rejected/Archived` without generic ad hoc row mutation.
- Why this needs code-spec depth: the change crosses repository mutation, typed candidate reconstruction, and runtime activation semantics while reusing the existing `rumination_candidates` substrate.

### 2. Signatures

- `src/memory/repository.rs`
  - `MemoryRepository::consume_skill_template_candidate(candidate_id, transitioned_at) -> Result<PersistedSkillMemoryTemplateCandidate, SkillTemplateCandidateLifecycleError>`
  - `MemoryRepository::reject_skill_template_candidate(candidate_id, transitioned_at) -> Result<PersistedSkillMemoryTemplateCandidate, SkillTemplateCandidateLifecycleError>`
  - `MemoryRepository::archive_skill_template_candidate(candidate_id, transitioned_at) -> Result<PersistedSkillMemoryTemplateCandidate, SkillTemplateCandidateLifecycleError>`
  - `SkillTemplateCandidateLifecycleError::{CandidateNotFound, WrongCandidateKind, InvalidTransition, Repository}`
- `src/cognition/assembly.rs`
  - runtime activation remains indirect through `MemoryRepository::list_consumed_skill_template_candidates_for_subject(...)`

### 3. Contracts

#### Substrate reuse contract

- Lifecycle helpers must reuse the existing `rumination_candidates` row for persisted skill candidates.
- Do not add a new table, mirror record, or external review surface for these transitions.

#### Type boundary contract

- The lifecycle bridge accepts only rows with `candidate_kind = skill_template`.
- Missing candidates must fail with `CandidateNotFound`.
- Non-`skill_template` rows must fail with `WrongCandidateKind` instead of being silently updated through the typed seam.
- Stored `skill_template` payloads must still round-trip through the typed decode boundary before any row mutation occurs.

#### Transition lattice contract

- The skill-template lifecycle bridge is intentionally narrow and allows only:
  - `Pending -> Consumed`
  - `Pending -> Rejected`
  - `Consumed -> Archived`
- Every other attempted status move must fail with `InvalidTransition`.
- Invalid transitions must leave the stored row unchanged, including `status` and `updated_at`.
- `Pending -> Archived` is forbidden in this phase unless a future task expands the contract explicitly.

#### Metadata preservation contract

- A lifecycle transition may change only:
  - `status`
  - `updated_at`
- The transition must preserve:
  - `payload`
  - `evidence_refs`
  - `source_queue_item_id`
  - `subject_ref`
  - `governance_ref_id`
  - `created_at`
- Returned values from the lifecycle helpers must stay typed as `PersistedSkillMemoryTemplateCandidate`.

#### Runtime activation contract

- Runtime assembly continues to observe only `Consumed` skill-template candidates.
- A candidate that remains `Pending`, `Rejected`, or `Archived` must not activate at runtime.
- After a successful consume transition, the existing runtime read model should observe the same candidate without any second activation path.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| `Pending` skill-template candidate is consumed | Candidate status becomes `Consumed`, `updated_at` changes, and typed payload metadata stays intact |
| `Pending` skill-template candidate is rejected | Candidate status becomes `Rejected` and metadata stays intact |
| `Consumed` skill-template candidate is archived | Candidate status becomes `Archived` and metadata stays intact |
| `Pending` skill-template candidate is archived | Lifecycle helper fails with `InvalidTransition`; stored row remains unchanged |
| `Consumed` skill-template candidate is rejected | Lifecycle helper fails with `InvalidTransition`; stored row remains unchanged |
| `Rejected` or `Archived` skill-template candidate is consumed | Lifecycle helper fails with `InvalidTransition`; stored row remains unchanged |
| Candidate id is missing | Lifecycle helper fails with `CandidateNotFound` |
| Candidate id belongs to another candidate kind | Lifecycle helper fails with `WrongCandidateKind` |
| Candidate payload is invalid or legacy | Lifecycle helper fails through the repository typed decode boundary before any status mutation |
| Assembly runs before consume | Pending candidate remains inactive |
| Assembly runs after consume | The same candidate is visible through the consumed runtime read model |

### 5. Good / Base / Bad Cases

- Good:
  - Transition skill-template candidates through explicit typed helpers.
  - Keep lifecycle changes inside the allowed transition lattice and fail closed on every other edge.
  - Reuse the existing repository round-trip so payload and lineage fields are preserved.
  - Verify runtime activation by consuming a pending candidate and re-running assembly.
- Base:
  - Generic repository update methods remain available for other candidate kinds and internal flows.
- Bad:
  - Call `update_rumination_candidate(...)` directly from skill-memory callers for status-only lifecycle changes.
  - Mutate payload or evidence refs as part of a lifecycle transition.
  - Add a second runtime-only flag or activation table for consumed skill templates.

### 6. Tests Required

- `tests/memory_repository_store.rs`
  - Assert `consume_skill_template_candidate(...)` preserves payload, evidence refs, source lineage, and timestamps
  - Assert `reject_skill_template_candidate(...)` and `archive_skill_template_candidate(...)` set the expected status
  - Assert invalid transition attempts fail explicitly without mutating `status` or `updated_at`
  - Assert wrong-kind and missing-candidate failures are explicit
  - Assert invalid or legacy payload rows still fail before mutation
- `tests/working_memory_assembly.rs`
  - Assert a pending persisted skill-template candidate stays inactive before consume
  - Assert the same candidate activates after the consume transition through the existing runtime read model

### 7. Wrong vs Correct

#### Wrong

- Treat lifecycle changes as untyped generic row edits at the skill-memory callsite.
- Archive or reject arbitrary candidate kinds through the skill-template lifecycle seam.
- Let any helper move a candidate to any status just because the status enum exists.
- Assume runtime activation changed just because a test rewrote a generic row manually.

#### Correct

- Use explicit skill-template lifecycle helpers that validate candidate kind and preserve metadata.
- Enforce the narrow transition lattice and reject invalid edges before mutating stored rows.
- Let runtime activation continue to depend only on `Consumed` rows returned by the existing repository read model.
