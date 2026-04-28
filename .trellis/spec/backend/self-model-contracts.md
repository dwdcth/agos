# Self Model Contracts

> Executable contracts for explicit self-model projection inside cognition and working-memory assembly.

---

## Scenario: Self Model Foundation Projection

### 1. Scope / Trigger

- Trigger: Phase 12 introduced an explicit self-model foundation and changed the internal `SelfStateProvider` seam from direct snapshot assembly to a projected self-model contract.
- Why this needs code-spec depth: this is a cross-layer contract change touching cognition projection, local adaptation overlays, working-memory output compatibility, and downstream agent-search / rumination consumers.

### 2. Signatures

- `src/cognition/self_model.rs`
  - `StableSelfKnowledge { capability_flags: Vec<String>, facts: Vec<SelfStateFact> }`
  - `RuntimeSelfState { task_context: Option<String>, readiness_flags: Vec<String>, facts: Vec<SelfStateFact> }`
  - `ProjectedSelfModel { stable: StableSelfKnowledge, runtime: RuntimeSelfState }`
  - `ProjectedSelfModel::project_snapshot() -> SelfStateSnapshot`
- `src/cognition/assembly.rs`
  - `trait SelfStateProvider { fn project(&self, request, truths) -> ProjectedSelfModel; fn snapshot(...) -> SelfStateSnapshot }`
  - `MinimalSelfStateProvider`
  - `AdaptiveSelfStateProvider<P>`
- `src/cognition/working_memory.rs`
  - outward compatibility stays on `SelfStateSnapshot`

### 3. Contracts

#### Projection contract

- `SelfStateProvider` now owns an explicit projection step through `ProjectedSelfModel`.
- `snapshot()` is no longer the primary customization point; it is a compatibility helper built on `project()`.
- `WorkingMemory.present.self_state` must remain a `SelfStateSnapshot` for downstream compatibility.

#### Stable vs runtime split

- Stable self-model data currently includes:
  - `capability_flags`
  - truth-derived stable facts
- Runtime self-model data currently includes:
  - `task_context`
  - `readiness_flags`
  - local adaptation facts

#### Overlay contract

- Local adaptation overlay must flow into `runtime.facts`, not into a separate ad hoc `SelfStateSnapshot` patch path.
- Future self-model expansion must extend `ProjectedSelfModel` rather than reintroducing assembler-local snapshot stitching.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Minimal provider receives truth records and request flags | Returns `ProjectedSelfModel` with stable capability/truth data and runtime task/readiness data |
| Adaptive provider receives local adaptation entries | Adds overlay facts into `runtime.facts` through the same projection seam |
| `ProjectedSelfModel::project_snapshot()` runs | Produces backward-compatible `SelfStateSnapshot` without changing the external working-memory contract |
| No local adaptation entries are present | Projection still works; runtime facts may be empty |
| Future code needs richer self-model data | Extend `ProjectedSelfModel`, not assembler-local field stitching |

### 5. Good / Base / Bad Cases

- Good:
  - Providers construct `ProjectedSelfModel` first and only then project to `SelfStateSnapshot`.
  - Overlay facts travel through `runtime.facts`.
  - Tests assert both stable/runtime separation and projected output compatibility.
- Base:
  - Existing consumers still read `working_memory.present.self_state` as `SelfStateSnapshot`.
- Bad:
  - Rebuilding `SelfStateSnapshot` directly inside `WorkingMemoryAssembler`.
  - Adding a second overlay path that mutates snapshots after projection.
  - Mixing self-model foundation work with new persistence schema in the same task.

### 6. Tests Required

- `tests/working_memory_assembly.rs`
  - Assert `MinimalSelfStateProvider` produces projected stable/runtime structure correctly
  - Assert overlay facts still surface after projection
- `tests/rumination_writeback.rs`
  - Assert local adaptation write-back still changes visible self-state through the explicit seam
- `tests/agent_search.rs`
  - Assert downstream agent-search payloads remain compatible with the unchanged `SelfStateSnapshot` output contract
- `tests/attention_state.rs`
  - Assert adjacent attention-state behavior still works when self-state projection is refactored

### 7. Wrong vs Correct

#### Wrong

- Keep `SelfStateProvider` focused on `SelfStateSnapshot` only
- Continue stitching capability/readiness/truth/local-adaptation fields directly in assembler code
- Add richer self-model data later by expanding assembler-local branching

#### Correct

- Introduce an explicit `ProjectedSelfModel`
- Make providers return structured stable/runtime self-model data
- Use `project_snapshot()` as the compatibility bridge back to `SelfStateSnapshot`

### Design Decision: Preserve Output Compatibility While Changing the Internal Seam

**Context**: The theory requires self model to become a first-class cognition layer, but current consumers already rely on `WorkingMemory.present.self_state` as `SelfStateSnapshot`.

**Decision**: Change the internal provider seam to `ProjectedSelfModel`, but preserve `SelfStateSnapshot` as the outward working-memory contract for this foundation phase.

**Why**:

- It allows incremental self-model evolution.
- It avoids breaking agent-search, rumination, and decision-report consumers.
- It prevents future code from returning to assembler-local field stitching.

**Related files**:

- `src/cognition/self_model.rs`
- `src/cognition/assembly.rs`
- `src/cognition/working_memory.rs`
- `tests/working_memory_assembly.rs`
- `tests/rumination_writeback.rs`

---

## Scenario: Ledger-First Self-Model Persistence Read Model

### 1. Scope / Trigger

- Trigger: Phase 13 keeps `local_adaptation_entries` as the durable substrate, but stops letting working-memory assembly consume raw ledger rows directly.
- Why this needs code-spec depth: this is a storage-to-cognition contract that changes lifecycle semantics, same-key overwrite behavior, and request-vs-persisted precedence without changing the outward `SelfStateSnapshot` surface.

### 2. Signatures

- `src/cognition/self_model.rs`
  - `ResolvedSelfModelEntry { target_kind, key, value, active, source_queue_item_id, updated_at, entry_id }`
  - `SelfModelReadModel::from_entries(persisted_entries, overlay_entries) -> SelfModelReadModel`
  - `SelfModelReadModel::from_overlay_entries(overlay_entries) -> SelfModelReadModel`
  - `SelfModelReadModel::entries() -> &[ResolvedSelfModelEntry]`
  - `SelfModelReadModel::active_facts() -> Vec<SelfStateFact>`
- `src/cognition/assembly.rs`
  - `WorkingMemoryRequest { ..., local_adaptation_entries, persisted_self_model, ... }`
  - `WorkingMemoryRequest::with_persisted_self_model(...) -> Self`
  - `AdaptiveSelfStateProvider<P>::project(...) -> ProjectedSelfModel`
  - `WorkingMemoryAssembler::assemble(...) -> Result<WorkingMemory, WorkingMemoryAssemblyError>`

### 3. Contracts

#### Read-model seam

- `WorkingMemoryAssembler` must build a `SelfModelReadModel` before projecting self-state when local adaptations are involved.
- Repository rows remain the write/read substrate, but cognition consumes aggregated lifecycle state, not ad hoc raw row order.
- Direct provider usage that skips assembler may fall back to `SelfModelReadModel::from_overlay_entries(...)` so tests and in-memory callers still route through the same lifecycle logic.
- If `WorkingMemoryRequest` already carries `persisted_self_model`, assembler must preserve that explicit read-model seam instead of silently rebuilding it from subject ledger rows.

#### Lifecycle resolution

- Logical-key identity is `target_kind + key`.
- Repeated writes to the same logical key resolve to one authoritative entry.
- Request overlays outrank persisted ledger entries for the same logical key, even if the persisted timestamp is newer.
- Within persisted entries, later `updated_at` wins; if timestamps tie, higher `entry_id` wins.
- Within request overlays, later caller order wins when lifecycle fields otherwise tie.

#### Active vs inactive

- The authoritative entry is active unless one of these explicit tombstone markers is present:
  - `payload.value == null`
  - `payload.value.active == false`
  - `payload.value.status == "inactive"`
  - `payload.value.state == "inactive"`
- Inactive winners stay in the read model, but must not surface in `runtime.facts`.

#### Deterministic surfaced order

- Active facts surfaced from the read model must be ordered by:
  1. `SelfState`
  2. `RiskBoundary`
  3. `PrivateT3`
- Inside each lane, sort by logical `key`.
- `WorkingMemory.present.self_state` remains a `SelfStateSnapshot`; only the internal source of runtime facts changes.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| No persisted entries and no request overlays | Read model is empty; runtime overlay facts stay empty |
| Multiple persisted entries share one logical key | Latest persisted winner surfaces once |
| Persisted and request entries share one logical key | Request winner surfaces once |
| Latest winner is inactive | Logical key is hidden from `runtime.facts` |
| Multiple lanes are present | Facts surface in `self_state`, `risk_boundary`, `private_t3` lane order |
| Rumination writes new local adaptation rows | Existing repository write path remains valid; next assembly reads them through the read model |

### 5. Good / Base / Bad Cases

- Good:
  - Assemble persisted ledger rows and request overlays into one `SelfModelReadModel`.
  - Keep lifecycle semantics in `self_model.rs`, not scattered across assembler and tests.
  - Preserve `ProjectedSelfModel` and `SelfStateSnapshot` seams while changing internal runtime fact sourcing.
- Base:
  - Distinct logical keys across lanes still appear as runtime facts.
  - Short-cycle rumination continues writing the same `local_adaptation_entries` substrate.
- Bad:
  - Re-extending `request.local_adaptation_entries` with repository rows and projecting them directly.
  - Letting duplicate same-key facts leak into `runtime.facts`.
  - Introducing new tables or inspect surfaces in the same phase.

### 6. Tests Required

- `tests/working_memory_assembly.rs`
  - Assert same-key request overlays overwrite earlier request values.
  - Assert request overlays override persisted values for the same logical key.
  - Assert persisted same-key rows resolve by `updated_at`, then `entry_id`.
  - Assert inactive winners suppress the surfaced fact.
  - Assert surfaced fact order is deterministic across `self_state`, `risk_boundary`, and `private_t3`.
- `tests/rumination_writeback.rs`
  - Assert rumination write-back still becomes visible through the provider seam without mutating shared-truth tables.

### 7. Wrong vs Correct

#### Wrong

- Read `local_adaptation_entries` in assembler and push every row straight into `runtime.facts`
- Let same-key duplicates survive because "ledger order is already deterministic"
- Encode request precedence by mutating unrelated outward contracts

#### Correct

- Build a `SelfModelReadModel` over persisted rows plus request overlays
- Resolve overwrite, inactive, and tie-break rules before projection
- Surface only active authoritative facts through the existing self-model seam

---

## Scenario: Self-Model Persistence Compaction Snapshot

### 1. Scope / Trigger

- Trigger: Phase 14 adds self-model-only snapshot/compaction on top of the existing ledger-first substrate so persisted self-state can be pruned and reconstructed efficiently without changing the outward projection contract.
- Why this needs code-spec depth: the change introduces a new SQLite table, new repository read/write seams, and a storage-to-cognition reconstruction path that must preserve lifecycle-core semantics exactly.

### 2. Signatures

- `migrations/0009_self_model_snapshots.sql`
  - `self_model_snapshots(subject_ref PRIMARY KEY, snapshot_id, entries_json, compacted_through_updated_at, compacted_through_entry_id, created_at, updated_at)`
- `src/memory/repository.rs`
  - `PersistedSelfModelSnapshotEntry`
  - `PersistedSelfModelSnapshot`
  - `PersistedSelfModelState { snapshot, tail_entries }`
  - `MemoryRepository::get_self_model_snapshot(subject_ref) -> Result<Option<PersistedSelfModelSnapshot>, RepositoryError>`
  - `MemoryRepository::load_self_model_state(subject_ref) -> Result<PersistedSelfModelState, RepositoryError>`
  - `MemoryRepository::replace_self_model_snapshot(snapshot) -> Result<(), RepositoryError>`
  - `MemoryRepository::prune_local_adaptation_entries_through(subject_ref, updated_at, entry_id) -> Result<usize, RepositoryError>`
- `src/cognition/self_model.rs`
  - `SelfModelReadModel::from_persisted_state(snapshot, persisted_entries, overlay_entries) -> SelfModelReadModel`
  - `SelfModelReadModel::to_snapshot(subject_ref, snapshot_id, compacted_at, cursor) -> SelfModelSnapshot`
  - `compact_self_model_subject(repository, subject_ref, snapshot_id, compacted_at) -> Result<Option<SelfModelSnapshot>, RepositoryError>`
- `src/cognition/assembly.rs`
  - `WorkingMemoryAssembler::assemble(...)` loads persisted self-model through `MemoryRepository::load_self_model_state(...)`

### 3. Contracts

#### Snapshot seam

- `self_model_snapshots` stores exactly one latest snapshot row per `subject_ref`.
- Snapshot payload stores resolved self-model entries, not raw ledger rows.
- Snapshot entries must preserve:
  - `target_kind`
  - `key`
  - `value`
  - `active`
  - `source_queue_item_id`
  - `updated_at`
  - `entry_id`

#### Reconstruction contract

- Persisted reconstruction order is:
  1. snapshot entries up to the compaction cursor
  2. ledger tail rows with `(updated_at, entry_id)` strictly after the cursor
  3. request overlays from `WorkingMemoryRequest.local_adaptation_entries`
- `WorkingMemoryAssembler` must rebuild `SelfModelReadModel` through `from_persisted_state(...)` when `subject_ref` is present and no explicit `persisted_self_model` was provided.
- An explicit `WorkingMemoryRequest.persisted_self_model` still outranks repository reconstruction and remains the projection seam override for tests/in-memory callers.

#### Compaction contract

- `compact_self_model_subject(...)` is self-model-only and must not touch world-model or skill-memory persistence.
- Compaction writes the new snapshot first, then prunes `local_adaptation_entries` at or before the snapshot cursor.
- The compaction cursor is the max `(updated_at, entry_id)` across the prior snapshot cursor and newly compacted ledger tail.
- Inactive winners stay in snapshot entries and remain suppressive after compaction; only `runtime.facts` omit them.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| No snapshot and no ledger rows for a subject | `load_self_model_state()` returns `snapshot = None`, `tail_entries = []`; compaction returns `None` |
| Snapshot exists and no newer ledger tail rows exist | Assembly rebuilds from snapshot only; compaction returns `None` because there is nothing new to compact |
| Snapshot exists and newer ledger tail rows exist | Assembly merges snapshot winners with tail rows before applying request overlays |
| Snapshot contains an inactive winner | Logical key remains absent from surfaced `runtime.facts` after rebuild |
| Snapshot upsert succeeds but prune fails | Read correctness still prefers snapshot + tail cursor filtering; only storage reduction is incomplete |
| Caller passes explicit `persisted_self_model` | Repository snapshot/tail reconstruction is skipped entirely |

### 5. Good / Base / Bad Cases

- Good:
  - Read persisted self-model through `load_self_model_state()` and `from_persisted_state(...)`.
  - Compact only `local_adaptation_entries` into `self_model_snapshots`.
  - Keep `SelfStateSnapshot` as the outward working-memory contract.
- Base:
  - Subjects without snapshots still use the existing ledger-only path.
  - New ledger rows written after compaction remain visible through the tail query.
- Bad:
  - Reading both snapshot and the full ledger, which would double-count compacted rows.
  - Dropping inactive winners from snapshot storage, which would resurrect suppressed facts.
  - Reusing the snapshot table for world-model or skill-memory persistence in this phase.

### 6. Tests Required

- `tests/working_memory_assembly.rs`
  - Assert assembly rebuilds from compacted snapshot plus newer ledger tail rows.
  - Assert inactive snapshot winners remain suppressive after compaction.
  - Assert explicit `persisted_self_model` still overrides repository reconstruction.
- `tests/foundation_schema.rs`
  - Assert schema version includes `self_model_snapshots` and its compaction-cursor index.
- `tests/memory_repository_store.rs`
  - Keep repository-level migration smoke coverage green after the additive schema bump.

### 7. Wrong vs Correct

#### Wrong

- Treat snapshot rows as an external product surface or a replacement for the self-model projection seam
- Compact raw ledger rows into ad hoc JSON without preserving lifecycle ordering fields
- Rebuild working memory by concatenating snapshot facts directly into `SelfStateSnapshot`

#### Correct

- Store resolved self-model entries plus a compaction cursor in `self_model_snapshots`
- Reconstruct persisted state through `SelfModelReadModel::from_persisted_state(...)`
- Preserve the explicit `ProjectedSelfModel` -> `SelfStateSnapshot` bridge unchanged while storage compacts underneath it

---

## Scenario: Self-Model Governance Conflict Review

### 1. Scope / Trigger

- Trigger: Phase 15 adds self-model-only governance on top of the existing ledger + snapshot + read-model substrate.
- Why this needs code-spec depth: the change crosses durable storage (`local_adaptation_entries`, `self_model_snapshots`), read-model reconstruction, and runtime projection while explicitly preserving the outward `ProjectedSelfModel` / `SelfStateSnapshot` contracts.

### 2. Signatures

- `src/memory/repository.rs`
  - `SelfModelResolutionState::{Unresolved, Accepted, Rejected}`
  - `SelfModelGovernanceMetadata { resolution, conflicting_entry_ids, review_reason }`
  - `PersistedSelfModelSnapshotEntry { ..., governance: Option<SelfModelGovernanceMetadata>, ... }`
- `src/cognition/self_model.rs`
  - `ResolvedSelfModelEntry { ..., governance: Option<SelfModelGovernanceMetadata>, ... }`
  - `ResolvedSelfModelEntry::governance_resolution() -> Option<SelfModelResolutionState>`
  - `SelfModelReadModel::from_persisted_state(...) -> SelfModelReadModel`
  - `SelfModelReadModel::active_facts() -> Vec<SelfStateFact>`

### 3. Contracts

#### Scope guard

- Governance in this phase is self-model-only.
- Do not widen the same metadata path into world-model persistence, skill-memory persistence, CLI inspection, MCP, or HTTP surfaces.

#### Ledger payload envelope

- Durable `local_adaptation_entries` may carry governed self-model writes by storing an envelope in `payload.value`:
  - `{"value": <actual self-model value>, "governance": {...}}`
- `governance` must deserialize to `SelfModelGovernanceMetadata`.
- When the envelope is present:
  - surfaced fact value comes from nested `value`
  - lifecycle `active` detection runs against nested `value`
  - governance metadata is attached to the resolved self-model entry

#### Snapshot persistence

- `self_model_snapshots.entries_json` must preserve governance metadata through serde round-trips.
- Compaction must carry `ResolvedSelfModelEntry.governance` into `PersistedSelfModelSnapshotEntry.governance`.

#### Deterministic read behavior

- Mechanical precedence still picks the authoritative same-key candidate first.
- Governance metadata does not change outward types; it changes whether the authoritative fact is allowed to surface.
- `Accepted`:
  - the authoritative entry may surface if `active == true`
- `Unresolved`:
  - fail closed; do not surface the authoritative fact
- `Rejected`:
  - fail closed; do not surface the authoritative fact
- Material conflict detection is same-logical-key disagreement on resolved `value` or `active` state.
- If a self-model payload carries a `governance` field that does not deserialize to `SelfModelGovernanceMetadata`, treat it as `Unresolved` and fail closed rather than silently dropping governance.
- If material conflict exists, governance metadata is already present on a lower-precedence candidate, and the authoritative entry has no governance metadata, read-model reconstruction should synthesize `SelfModelGovernanceMetadata { resolution = Unresolved }` on the authoritative entry and attach the lower-precedence conflicting ids so review cannot be bypassed by plain overwrites.
- If governance metadata exists and `conflicting_entry_ids` is empty, read-model reconstruction should populate it from materially conflicting lower-precedence candidates when available.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Governed write has `resolution = unresolved` and conflicts with an older same-key value | `active_facts()` excludes the key |
| Governed write has `resolution = accepted` and conflicts materially | `active_facts()` surfaces the authoritative value |
| Snapshot entry has `resolution = rejected` | `active_facts()` excludes the compacted winner |
| Payload carries malformed `governance` metadata | Treat as `Unresolved`; `active_facts()` excludes the key |
| A lower-precedence same-key candidate carries governance metadata, but the authoritative winner does not | Read-model synthesizes `Unresolved` governance metadata on the winner and `active_facts()` excludes the key |
| Governance metadata is present but `conflicting_entry_ids` is empty | Read-model fills it from lower-precedence conflicting candidates when they still exist |
| No governance metadata is present anywhere for the logical key | Existing lifecycle / overwrite / inactive semantics remain unchanged |

### 5. Good / Base / Bad Cases

- Good:
  - keep governance interpretation inside `self_model.rs`
  - preserve `ProjectedSelfModel` / `SelfStateSnapshot` unchanged
  - persist governance only via self-model ledger/snapshot structures
- Base:
  - ungoverned request overlays and persisted writes still resolve through the existing read-model precedence rules
- Bad:
  - adding a second assembler-local suppression path outside `SelfModelReadModel`
  - exposing governance-specific CLI or API inspection in the same phase
  - copying the same governance logic into world-model or skill-memory storage

### 6. Tests Required

- `tests/working_memory_assembly.rs`
  - assert unresolved governed conflicts fail closed
  - assert governed lower-precedence conflicts cannot be bypassed by later plain overwrites
  - assert accepted governed conflicts surface deterministically
  - assert malformed governance metadata fails closed
  - assert rejected snapshot governance suppresses compacted winners
- `tests/memory_repository_store.rs`
  - assert snapshot governance metadata round-trips through repository serde

### 7. Wrong vs Correct

#### Wrong

- Treat governance as an assembler concern and post-filter `SelfStateSnapshot`
- Persist reviewed state only in memory while leaving snapshots unaware of it
- Let unresolved same-key governed writes surface because precedence already chose a winner

#### Correct

- Keep governance on resolved self-model entries and snapshot entries
- Fail closed for synthesized unresolved, explicit unresolved, or rejected governed winners
- Preserve existing outward cognition contracts while making internal read behavior deterministic
