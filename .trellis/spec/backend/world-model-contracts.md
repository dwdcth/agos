# World Model Contracts

> Executable contracts for explicit world-model projection inside cognition and working-memory assembly.

---

## Scenario: World Model Foundation Projection

### 1. Scope / Trigger

- Trigger: Phase 13 introduced an explicit world-model foundation and changed working-memory assembly from assembler-local `EvidenceFragment` construction to a projected world-model seam.
- Why this needs code-spec depth: this is a cross-layer contract change touching retrieval results, truth projection, working-memory compatibility, and downstream agent-search / CLI / rumination consumers.

### 2. Signatures

- `src/cognition/world_model.rs`
  - `ProjectedWorldModel { current: CurrentWorldSlice }`
  - `CurrentWorldSlice { fragments: Vec<WorldFragmentProjection> }`
  - `WorldFragmentProjection`
  - `WorldFragmentProjection::from_search_result(result, truth, repository_dsl) -> WorldFragmentProjection`
  - `ProjectedWorldModel::project_fragments() -> Vec<EvidenceFragment>`
- `src/cognition/assembly.rs`
  - `WorkingMemoryAssembler::assemble(...)`
  - builds `ProjectedWorldModel` from retrieved `SearchResult`s plus truth projection
- `src/cognition/working_memory.rs`
  - outward compatibility stays on `PresentFrame.world_fragments: Vec<EvidenceFragment>`

### 3. Contracts

#### Projection contract

- `WorkingMemoryAssembler` must construct world state through `WorldFragmentProjection` and `ProjectedWorldModel`.
- The assembler must not return to assembler-local `EvidenceFragment` stitching as the primary path.
- `WorkingMemory.present.world_fragments` must remain `Vec<EvidenceFragment>` for downstream compatibility.

#### Metadata preservation contract

- `WorldFragmentProjection` must preserve:
  - `record_id`
  - `snippet`
  - `citation`
  - `provenance`
  - `truth_context`
  - `dsl`
  - `trace`
  - `score`
- Repository DSL may only be used as a fallback when the incoming `SearchResult.dsl` is absent.
- Truth context must continue to derive from the existing `TruthRecord` lookup, not from ad hoc fragment metadata.

#### Adjacent seam contract

- Attention-state scoring and trace data must survive projection unchanged through `trace.attention` and `score.attention_bonus`.
- Self-model projection must remain a separate seam; world-model foundation work must not reintroduce assembler-local `SelfStateSnapshot` stitching.
- Action branches, agent-search reports, CLI rendering, and rumination inputs must continue to consume backward-compatible `EvidenceFragment`s.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Search result already includes DSL | Projection preserves that DSL payload |
| Search result lacks DSL but repository layered record has one | Projection hydrates the repository DSL fallback |
| Search result lacks truth projection | Assembly fails with `MissingTruthProjection` |
| Attention bias is inactive | `trace.attention == null` and `score.attention_bonus == 0.0` survive projection |
| Attention bias is active | Projected fragments preserve the retrieval trace and score bonus |
| Future world-model work needs richer structure | Extend `ProjectedWorldModel` / `CurrentWorldSlice` / `WorldFragmentProjection`, not `PresentFrame.world_fragments` |

### 5. Good / Base / Bad Cases

- Good:
  - Build `WorldFragmentProjection` from existing retrieval + truth records.
  - Use `project_fragments()` as the compatibility bridge back to `EvidenceFragment`.
  - Assert metadata round-trip with focused tests.
- Base:
  - Existing consumers still read `working_memory.present.world_fragments` as before.
- Bad:
  - Rebuilding `EvidenceFragment` directly inside `WorkingMemoryAssembler`.
  - Creating a second citation / truth / provenance source outside retrieval plus repository truth lookup.
  - Mixing world-model foundation work with simulation, prediction, or persistence schema changes.

### 6. Tests Required

- `tests/world_model_projection.rs`
  - Assert world-model projection round-trips citation, provenance, truth context, trace, score, and DSL metadata
  - Assert repository DSL fallback works when search results have no sidecar
- `tests/working_memory_assembly.rs`
  - Assert assembled `world_fragments` remain backward-compatible
  - Assert integrated follow-up evidence and filter traces survive the projection seam
  - Assert adjacent self-model and attention contracts still hold through assembly
- `tests/agent_search.rs`
  - Assert downstream agent-search payloads still expose compatible `world_fragments`
- `tests/retrieval_cli.rs`
  - Assert CLI JSON / text output continues to render fragment trace and source metadata without contract drift

### 7. Wrong vs Correct

#### Wrong

- Keep world-model semantics implicit inside assembler-local fragment construction
- Add richer world-state fields by expanding `EvidenceFragment` first
- Let downstream consumers depend on `ProjectedWorldModel` directly in this foundation phase

#### Correct

- Introduce explicit world-model projection types
- Keep `EvidenceFragment` as the outward compatibility layer
- Route all fragment materialization through the shared world-model seam

### Design Decision: Change the Internal Seam, Preserve the External Fragment Contract

**Context**: The theory requires world model to become a first-class cognition layer, but current downstream consumers already rely on `WorkingMemory.present.world_fragments` as `EvidenceFragment`s.

**Decision**: Introduce `ProjectedWorldModel` and `WorldFragmentProjection` internally, while preserving `PresentFrame.world_fragments` as the outward compatibility contract for this foundation phase.

**Why**:

- It allows future world-model expansion without another broad assembler rewrite.
- It preserves current agent-search, CLI, and rumination consumers.
- It keeps citations, truth context, provenance, DSL, trace, and score data on a single explainable path.

**Related files**:

- `src/cognition/world_model.rs`
- `src/cognition/assembly.rs`
- `src/cognition/working_memory.rs`
- `tests/world_model_projection.rs`
- `tests/working_memory_assembly.rs`
- `tests/agent_search.rs`

---

## Scenario: World-Model Durable Snapshot Persistence

### 1. Scope / Trigger

- Trigger: Phase 10 adds the first durable storage substrate for the explicit internal world-model projection.
- Why this needs code-spec depth: the change crosses SQLite schema, repository DTOs, cognition reconstruction, and test contracts while explicitly preserving the outward `WorkingMemory.present.world_fragments` shape.

### 2. Signatures

- `migrations/0010_world_model_snapshots.sql`
  - `world_model_snapshots(subject_ref, world_key, snapshot_id, fragments_json, created_at, updated_at)`
- `src/core/migrations.rs`
  - migration registration for `0010_world_model_snapshots.sql`
- `src/memory/repository.rs`
  - `PersistedWorldModelSnapshot`
  - `PersistedWorldModelSnapshotFragment`
  - `MemoryRepository::load_world_model_snapshot(subject_ref, world_key) -> Result<Option<PersistedWorldModelSnapshot>, RepositoryError>`
  - `MemoryRepository::replace_world_model_snapshot(snapshot) -> Result<(), RepositoryError>`
- `src/cognition/world_model.rs`
  - `ProjectedWorldModel::to_snapshot(subject_ref, world_key, snapshot_id, created_at, updated_at) -> PersistedWorldModelSnapshot`
  - `ProjectedWorldModel::from_snapshot(snapshot) -> ProjectedWorldModel`
  - `WorldFragmentProjection::to_snapshot_fragment() -> PersistedWorldModelSnapshotFragment`
  - `WorldFragmentProjection::from_snapshot_fragment(fragment) -> WorldFragmentProjection`

### 3. Contracts

#### Snapshot identity contract

- World-model snapshots are keyed by `subject_ref + world_key`.
- `snapshot_id` is required metadata on each stored snapshot, but it does not widen the uniqueness boundary beyond `subject_ref + world_key`.
- `world_key` scopes the persisted slice; the current-world slice should use a stable internal key such as `"current"`.

#### Metadata preservation contract

- Snapshot fragments must preserve enough metadata to reconstruct `ProjectedWorldModel` without loss:
  - `record_id`
  - `snippet`
  - `citation`
  - `provenance`
  - `truth_context`
  - `dsl`
  - `trace`
  - `score`
- `ProjectedWorldModel::from_snapshot(...)` must rebuild `WorldFragmentProjection` values that are equal to the original pre-persisted projection.

#### Internal-only persistence contract

- The persisted representation is internal-only.
- Do not make `SearchResult` or outward CLI/HTTP structs the primary durable contract for this phase.
- Prefer dedicated persisted DTOs for trace/citation/filter metadata rather than broadening output-facing serde contracts without need.
- Do not mix skill-memory persistence or prediction/simulation fields into the same snapshot.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| No snapshot exists for `subject_ref + world_key` | `load_world_model_snapshot(...)` returns `Ok(None)` |
| A snapshot is rewritten for the same `subject_ref + world_key` | Repository upsert replaces the stored row rather than creating a second logical snapshot |
| Snapshot JSON stores fragment metadata | `ProjectedWorldModel::from_snapshot(...)` reconstructs the same projection metadata and `project_fragments()` output |
| Existing working-memory assembly reads live retrieval results | No external assembly behavior changes are required in this phase |
| Future world-model work needs richer persistent structure | Extend the dedicated persisted snapshot DTOs, not `EvidenceFragment` or public response shapes |

### 5. Good / Base / Bad Cases

- Good:
  - Persist `ProjectedWorldModel` through dedicated snapshot DTOs.
  - Keep repository APIs scoped to `subject_ref + world_key`.
  - Assert equality on both snapshot round-trip and reconstructed projected fragments.
- Base:
  - Existing working-memory assembly still projects live retrieval results through `ProjectedWorldModel` and remains backward-compatible.
- Bad:
  - Persist raw `SearchResult` or CLI JSON as the storage contract.
  - Expose world-model snapshots through CLI/HTTP/MCP in the same phase.
  - Reuse the same schema for skill-memory persistence or prediction state.

### 6. Tests Required

- `tests/foundation_schema.rs`
  - Assert schema version advances additively and `world_model_snapshots` exposes `subject_ref`, `world_key`, `snapshot_id`, `fragments_json`, `created_at`, and `updated_at`
  - Assert a uniqueness index exists for `subject_ref + world_key`
- `tests/memory_repository_store.rs`
  - Assert repository round-trips a `PersistedWorldModelSnapshot`
  - Assert lookup uses `subject_ref + world_key`
- `tests/world_model_projection.rs`
  - Assert `ProjectedWorldModel::to_snapshot(...)` and `from_snapshot(...)` preserve fragment metadata and reconstructed `EvidenceFragment`s
- Regression suites:
  - `tests/working_memory_assembly.rs`
  - `tests/agent_search.rs`
  - `tests/retrieval_cli.rs`

### 7. Wrong vs Correct

#### Wrong

- Add `Deserialize` broadly to outward search/CLI payloads just to satisfy an internal snapshot need
- Store only `record_id`s and re-fetch the rest later, losing the explicit explainability snapshot
- Widen the scope to skill-memory persistence or simulation state

#### Correct

- Store dedicated world-model snapshot DTOs behind the repository seam
- Preserve citation, truth, DSL, trace, and score metadata inside `fragments_json`
- Keep the change internal to cognition/repository/schema layers while preserving outward working-memory compatibility

---

## Scenario: World-Model Runtime Read Model From Snapshot

### 1. Scope / Trigger

- Trigger: Phase 10 added durable snapshot persistence, but runtime working-memory assembly still always reconstructed world state from live retrieval. This phase bridges persisted `"current"` snapshots back into the assembly path.
- Why this needs code-spec depth: the change introduces a runtime read-model helper, a precedence-resolving dispatch inside `WorkingMemoryAssembler`, truth rehydration from snapshot fragments, and explicit precedence rules that must remain deterministic across sessions.

### 2. Signatures

- `src/cognition/world_model.rs`
  - `CURRENT_WORLD_KEY: &str = "current"`
  - `load_runtime_current_world_model(repository, subject_ref) -> Result<Option<ProjectedWorldModel>, RepositoryError>`
- `src/cognition/assembly.rs`
  - `WorkingMemoryAssembler::resolve_world_model(&self, request) -> Result<(ProjectedWorldModel, Vec<TruthRecord>), WorkingMemoryAssemblyError>`
  - `WorkingMemoryAssembler::project_live_world_model(&self, request) -> Result<(ProjectedWorldModel, Vec<TruthRecord>), WorkingMemoryAssemblyError>`
  - `WorkingMemoryAssembler::load_truths_for_world_model(&self, world_model) -> Result<Vec<TruthRecord>, WorkingMemoryAssemblyError>`

### 3. Contracts

#### Precedence contract

- When `integrated_results` is non-empty, assembly uses live retrieval (the snapshot path is skipped entirely).
- When `integrated_results` is empty and `subject_ref` is present and a `"current"` snapshot exists for that subject, assembly uses the snapshot-backed `ProjectedWorldModel`.
- When `integrated_results` is empty and no snapshot exists, assembly falls back to live retrieval unchanged.
- This precedence is three-tier: explicit integrated results > snapshot-backed current world model > live retrieval.

#### Snapshot read-model contract

- `load_runtime_current_world_model` loads the snapshot keyed by `subject_ref + "current"` and reconstructs `ProjectedWorldModel` via `from_snapshot`.
- The reconstructed model must produce identical `project_fragments()` output to the original pre-persisted projection.

#### Truth rehydration contract

- Snapshot-backed assembly rehydrates `TruthRecord`s from the repository using each fragment's `record_id`.
- Deduplication is by `record_id` using a `BTreeSet`.
- If a truth record is missing, assembly fails with `MissingTruthProjection` — the same error as the live path.

#### Outward compatibility

- `WorkingMemory.present.world_fragments` remains `Vec<EvidenceFragment>` regardless of which world-model source was used.
- Self-model projection, action branches, skill seeds, and downstream consumers are unaffected.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| `integrated_results` non-empty | Live retrieval path used; snapshot ignored |
| `integrated_results` empty, `subject_ref` present, `"current"` snapshot exists | Snapshot-backed assembly |
| `integrated_results` empty, no `subject_ref` | Live retrieval |
| `integrated_results` empty, `subject_ref` present, no snapshot | Live retrieval fallback |
| Snapshot fragment references a missing truth record | `MissingTruthProjection` error |
| Snapshot fragment has duplicate `record_id`s | Deduplicated; each truth loaded once |

### 5. Good / Base / Bad Cases

- Good:
  - Three-tier precedence keeps explicit caller data authoritative.
  - Truth rehydration reuses the same repository seam as live assembly.
  - `project_fragments()` produces identical output regardless of source.
- Base:
  - No `subject_ref` or no snapshot means zero behavior change from the live path.
- Bad:
  - Loading snapshot fragments directly into `EvidenceFragment` without truth rehydration.
  - Letting snapshot-backed fragments skip the `project_fragments()` bridge.
  - Adding a second world-model source without documenting precedence rules.

### 6. Tests Required

- `tests/world_model_projection.rs`
  - Assert `load_runtime_current_world_model` round-trips through snapshot persistence and reconstructs an equal `ProjectedWorldModel`.
- `tests/working_memory_assembly.rs`
  - Assert snapshot-backed assembly produces correct `world_fragments`.
  - Assert live retrieval fallback when no snapshot exists.
  - Assert explicit integrated results outrank snapshot-backed world state.
  - Assert snapshot-backed fragments remain compatible with branch materialization and self-state projection.

### 7. Wrong vs Correct

#### Wrong

- Bypass `resolve_world_model` and load the snapshot inside the main `assemble` body without precedence checks.
- Skip truth rehydration for snapshot-backed fragments, leaving `self_state` incomplete.
- Add a new outward field on `PresentFrame` to distinguish snapshot vs live sources.

#### Correct

- Dispatch through `resolve_world_model` with three-tier precedence.
- Rehydrate truths from the repository using snapshot fragment `record_id`s.
- Keep the outward `world_fragments` contract identical regardless of source.

---

## Scenario: World-Model Prediction / Simulation

### 1. Scope / Trigger

- Trigger: The theory requires `Simulate(s_t, a) → ŝ_{t+1}, r̂, û` — the world model must predict consequences of candidate actions.
- Why this needs code-spec depth: prediction crosses the world model, action system, and LLM backend; it produces ephemeral structured output that must not mutate the current world model.

### 2. Signatures

- `src/cognition/world_model.rs`
  - `ChangeDirection { Strengthened, Weakened, Invalidated, Unchanged, NewRisk }`
  - `PredictedFragmentChange { record_id, change_description, change_direction }`
  - `PredictedSeverity { Low, Medium, High }`
  - `PredictedRisk { description, severity }`
  - `PredictedWorldSlice { affected_fragments, new_risks, uncertainty_delta, overall_assessment }`
  - `SimulationResult { predicted: PredictedWorldSlice, confidence, action_summary }`
  - `SimulationError { LlmUnconfigured, LlmRequestFailed }`
  - `SimulationStructuredOutput` (JsonSchema for rig TypedPrompt)
  - `WorldSimulator<B>` with `simulate_async()` and `simulate()`
  - `build_simulation_prompt(world_fragments, action) -> String`

### 3. Contracts

#### Prediction contract

- Prediction takes a `CurrentWorldSlice` + `ActionCandidate` and returns a `SimulationResult`.
- Prediction is ephemeral: not persisted, not stored in any snapshot.
- Prediction must NOT mutate the current world model or its fragments.
- Prediction is opt-in: assembly and orchestration do not automatically trigger prediction.

#### LLM backend contract

- Follows the same `TypedPrompt<T>` + `JsonSchema` pattern as `src/memory/summary.rs`.
- Graceful degradation: returns `SimulationError` when LLM is unavailable, never panics.
- Both async (`simulate_async`) and sync (`simulate`) entry points.

#### Prompt contract

- Prompt includes: world fragment record_ids, snippets, optional DSL claims, and full action details (kind, summary, intent, expected_effects).
- Prompt is self-contained and deterministic given the same inputs.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| LLM config missing or incomplete | `SimulationError::LlmUnconfigured` |
| LLM request fails (network, timeout) | `SimulationError::LlmRequestFailed` |
| World fragments empty | Prompt still valid, prediction covers only action consequences |
| Action has no expected_effects | Prompt omits expected effects section |
| Action has no intent | Prompt uses fallback "no explicit intent provided" |

### 5. Good / Base / Bad Cases

- Good:
  - Structured prediction with confidence score and traceable fragment changes.
  - Prompt is self-contained and reproducible.
- Base:
  - Without calling `simulate()`, system behaves identically to before.
- Bad:
  - Persisting predictions into the current world snapshot.
  - Auto-triggering prediction on every assembly call.
  - Letting prediction output bypass the evidence fragment contract.

### 6. Tests Required

- `src/cognition/world_model.rs` (inline tests)
  - Assert prompt includes all world fragments and action fields
  - Assert prompt handles empty expected_effects and missing intent
  - Assert SimulationStructuredOutput converts to SimulationResult
  - Assert PredictedWorldSlice serialization round-trip
  - Assert end-to-end simulation with stub backend

### 7. Wrong vs Correct

#### Wrong

- Auto-predict on every assembly call without explicit opt-in.
- Persist predictions into world_model_snapshots.
- Return unstructured text instead of structured prediction types.

#### Correct

- Prediction is opt-in, ephemeral, and returns structured types.
- Follow the same rig TypedPrompt pattern as summary generation.
- Gracefully degrade when LLM is unavailable.
