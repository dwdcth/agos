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
