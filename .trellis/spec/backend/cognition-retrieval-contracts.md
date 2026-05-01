# Cognition-Retrieval Contracts

> Executable contracts for features that cross cognition, ordinary retrieval, and agent-search seams.

---

## Scenario: Attention State Retrieval Bias

### 1. Scope / Trigger

- Trigger: Phase 11 added an explicit attention-state layer that changes shared request contracts and result-trace shape across cognition, retrieval, working-memory assembly, and agent-search.
- Why this needs code-spec depth: this is a cross-layer contract change, not a local scoring tweak.

### 2. Signatures

- `src/cognition/attention.rs`
  - `AttentionLane { Goal, Risk, Metacog, Readiness, Capability }`
  - `AttentionCue { lane, source, cue, weight }`
  - `AttentionBaseline`
  - `AttentionContribution { lane, source, cue, matched_fields, bonus }`
  - `AttentionDelta { total_bonus, contributions }`
  - `AttentionState { baseline: AttentionBaseline, delta: AttentionDelta }`
  - `AttentionTrace { total_bonus, contributions: Vec<AttentionContribution> }`
  - `ATTENTION_BONUS_CAP: f32 = 0.15`
  - Lane weight constants: Goal=0.06, Risk=0.04, Metacog=0.03, Readiness=0.02, Capability=0.02
  - `AttentionState::derive_delta(goal, risks, metacog_flags, readiness_flags, capability_flags) -> AttentionDelta`
- `src/search/mod.rs`
  - `SearchRequest { query, limit, filters, attention_state: Option<AttentionState> }`
  - `SearchRequest::with_attention_state(attention_state)`
- `src/cognition/assembly.rs`
  - `WorkingMemoryRequest { ..., attention_state: Option<AttentionState>, ... }`
  - `WorkingMemoryRequest::with_attention_state(attention_state)`
  - `WorkingMemoryRequest::resolved_attention_state() -> Option<AttentionState>`
- `src/agent/orchestration.rs`
  - `AgentSearchRequest::with_attention_state(attention_state)` forwards into `working_memory`
- `src/search/score.rs`
  - `ScoreBreakdown { ..., attention_bonus: f32, final_score: f32 }`
- `src/search/rerank.rs`
  - `ResultTrace { matched_query, query_strategies, channel_contribution, applied_filters, attention }`

### 3. Contracts

#### Request contract

- `SearchRequest.attention_state` is optional.
- No attention state means retrieval must preserve the existing lexical-first baseline.
- `WorkingMemoryRequest.attention_state` is optional.
- `WorkingMemoryRequest::resolved_attention_state()` uses the explicit state first.
- An explicit but empty `AttentionState` on `WorkingMemoryRequest` suppresses derived bias and is treated as “no attention”.
- If no explicit state is present, `resolved_attention_state()` may derive a narrow `AttentionDelta` from:
  - `active_goal`
  - `active_risks`
  - `metacog_flags`
  - `readiness_flags`
  - `capability_flags`

#### Scoring contract

- Attention influence is additive and Rust-side.
- Attention must not create a parallel retrieval path outside `SearchService`.
- Attention must not replace lexical explanation.
- `attention_bonus` contributes to `final_score`, but lexical scoring and citations remain the primary explanation surface.
- Total attention bonus is capped at `ATTENTION_BONUS_CAP` (0.15).
- Per-lane weights are: Goal 0.06, Risk 0.04, Metacog 0.03, Readiness 0.02, Capability 0.02.
- Matching: each cue is split into terms and checked against candidate label, content, and DSL fields. A match adds the lane weight as bonus.

#### Trace contract

- When no attention bias is active, `score.attention_bonus == 0.0` and `trace.attention == null`.
- When attention bias is active, `trace.attention` must expose:
  - `total_bonus`
  - per-contribution `lane`
  - `source`
  - `cue`
  - matched fields and bonus contribution

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| `attention_state` omitted on `SearchRequest` | Retrieval stays backward-compatible; no synthetic attention trace |
| Explicit `AttentionState` provided but empty | Treat as absent; do not emit attention bonus |
| Explicit empty `AttentionState` provided on `WorkingMemoryRequest` | Do not derive fallback goal/risk/metacog/readiness/capability bias |
| `WorkingMemoryRequest` has no explicit attention but has goal/risk/metacog/readiness/capability metadata | Derive a narrow delta through `resolved_attention_state()` |
| Agent-search runs multiple bounded queries | The same resolved attention state is forwarded into each retrieval step |
| Attention bias matches candidate fields | Add a capped positive `attention_bonus` and emit `trace.attention` |
| Attention does not match any candidate fields | `attention_bonus == 0.0`; `trace.attention == null` |

### 5. Good / Base / Bad Cases

- Good:
  - Explicit `AttentionState` is passed through `SearchRequest::with_attention_state(...)`.
  - Derived attention is reused through `WorkingMemoryRequest::resolved_attention_state()`.
  - Tests assert both scoring (`attention_bonus`) and trace shape (`trace.attention`).
- Base:
  - Existing lexical retrieval code calls `SearchRequest::new(query)` with no attention state and remains unchanged.
- Bad:
  - Creating a second “attention-aware search” API instead of extending `SearchRequest`.
  - Applying attention only inside CLI formatting or only inside agent-search reporting.
  - Emitting a hidden bonus with no trace output.

### 6. Tests Required

- Unit / focused integration:
  - `tests/attention_state.rs`
  - Assert no-attention requests keep `attention_bonus == 0.0`
  - Assert explicit goal/risk/metacog cues can deterministically rerank results
- Working-memory integration:
  - `tests/working_memory_assembly.rs`
  - Assert derived attention from working-memory metadata reaches retrieval and survives in fragment trace
  - Assert an explicit empty `AttentionState` disables derived fallback bias
- Agent-search integration:
  - `tests/agent_search.rs`
  - Assert explicit attention and derived attention both forward into retrieval requests
- CLI contract:
  - `tests/retrieval_cli.rs`
  - Assert JSON trace keeps `attention: null` when inactive
  - Assert text `--trace` output renders `attention_bonus`

### 7. Wrong vs Correct

#### Wrong

- Add a new retrieval entrypoint such as `AttentionSearchRequest`
- Thread it only through agent-search
- Leave ordinary retrieval and CLI trace output unaware of the new contract

#### Correct

- Extend the shared `SearchRequest` contract with optional `attention_state`
- Reuse the same seam from ordinary retrieval, working-memory assembly, and agent-search
- Keep attention additive in scoring/rerank and expose it in `ScoreBreakdown` plus `ResultTrace`

### Design Decision: Attention Is Additive, Not Authoritative

**Context**: The theory requires attention to influence what enters the foreground, but the project also requires lexical-first explainability and backward-compatible ordinary retrieval.

**Decision**: Apply attention as an additive, deterministic bonus inside Rust-side scoring/rerank instead of changing the raw recall engine contract or introducing a separate attention-aware retrieval pipeline.

**Why**:

- It keeps retrieval explainable.
- It keeps lexical citations and source metadata as the primary explanation surface.
- It avoids a semantic-only or cognition-only bypass path.

**Related files**:

- `src/cognition/attention.rs`
- `src/search/mod.rs`
- `src/search/score.rs`
- `src/search/rerank.rs`
- `src/cognition/assembly.rs`
- `src/agent/orchestration.rs`

---

## Scenario: Semantic Retrieval Runtime Readiness

### 1. Scope / Trigger

- Trigger: runtime status and doctor output now treat `embedding_only` / `hybrid` as real supported modes, but only when the semantic runtime substrate is actually configured and present.
- Why this needs code-spec depth: this is a cross-layer command/runtime contract affecting config parsing, status rendering, doctor gating, CLI behavior, and retrieval-mode tests.

### 2. Signatures

- `src/core/app.rs`
  - `RuntimeReadiness { configured_mode, effective_mode, ready, notes }`
  - `RuntimeReadiness::from_config(config)`
- `src/core/status.rs`
  - `StatusReport { ..., embedding_backend, vector_backend, lexical_dependency_state, embedding_dependency_state, index_readiness, embedding_index_readiness, ready, ... }`
  - `StatusReport::collect(app)`
  - `embedding_dependency_state(mode, backend, model, vector_backend) -> CapabilityState`
  - `embedding_backend_label(backend) -> &'static str`
  - `vector_backend_label(backend) -> &'static str`
- `src/core/doctor.rs`
  - `DoctorReport::evaluate(status, command_path)`
  - `operational_readiness_failures(status) -> Vec<String>`
  - `doctor_mode_readiness_failures(status) -> Vec<String>`
- `src/search/mod.rs`
  - `SearchService::search(request)` with `RetrievalMode::{LexicalOnly, EmbeddingOnly, Hybrid}`

### 3. Contracts

#### Runtime-readiness contract

- `lexical_only` remains the default ready mode after schema/index initialization.
- `embedding_only` is config-ready only when all of the following hold:
  - `embedding.backend = builtin`
  - `embedding.model` is present
  - `vector.backend = sqlite_vec`
- `hybrid` is config-ready only when all of the following hold:
  - `embedding.backend = builtin`
  - `embedding.model` is present
  - `vector.backend = sqlite_vec`
- Config readiness is not the same thing as operational readiness.
- Operational readiness for semantic modes additionally depends on:
  - embedding sidecar/index presence in the database
  - lexical index presence for `hybrid`

#### Status contract

- `status` is informational and must still render even when `ready: false`.
- `status` must expose:
  - `embedding_backend`
  - `vector_backend`
  - `lexical_dependency_state`
  - `embedding_dependency_state`
  - `index_readiness`
  - `embedding_index_readiness`
  - `active_channels`
  - `gated_channels`
- `embedding_dependency_state` must reflect vector backend reality:
  - builtin + model + `sqlite_vec` -> semantic dependency can be `ready`
  - builtin + model + `none` -> semantic dependency is not ready for semantic modes

#### Doctor / operational gate contract

- `doctor`, `search`, `ingest`, and `agent-search` must fail closed when semantic runtime requirements are missing.
- If `embedding.backend = reserved`:
  - `embedding_only` failure text must say it requires a builtin embedding backend
  - `hybrid` failure text must say it requires a builtin embedding backend for the secondary path
- If `vector.backend = none` under semantic modes:
  - `embedding_only` must fail with `vector backend is not ready for embedding_only retrieval`
  - `hybrid` must fail with `vector backend is not ready for hybrid retrieval`
- `hybrid` must not silently degrade to lexical-only in operational commands when the semantic path is explicitly requested but unavailable.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| `lexical_only` + `embedding.backend=disabled` | `status ready: true`; lexical path remains usable |
| `embedding_only` + `embedding.backend=reserved` | `doctor/search/agent-search` fail closed with builtin-backend requirement |
| `hybrid` + `embedding.backend=reserved` | `doctor/search/agent-search` fail closed with builtin-backend requirement |
| `embedding_only` + builtin model + `vector.backend=none` | `status ready: false`; doctor/search fail closed with vector-backend error |
| `hybrid` + builtin model + `vector.backend=none` | `status ready: false`; `active_channels` may still show lexical, but doctor/search fail closed with vector-backend error |
| `embedding_only` + builtin model + `sqlite_vec` + ready sidecar/index | `status ready: true`; semantic operational commands succeed |
| `hybrid` + builtin model + `sqlite_vec` + ready lexical + embedding indexes | `status ready: true`; semantic operational commands succeed |

### 5. Good / Base / Bad Cases

- Good:
  - `status` reports semantic modes truthfully without overclaiming readiness.
  - `doctor` distinguishes missing embedding backend from missing vector backend.
  - Tests cover reserved backend, missing vector backend, and fully ready semantic modes.
- Base:
  - `lexical_only` remains informative even when embedding config exists as optional foundation.
- Bad:
  - Reporting semantic mode as ready from config alone while vector backend is absent.
  - Allowing `hybrid` CLI commands to silently fall back to lexical when semantic mode was explicitly selected but unavailable.
  - Keeping stale “Phase 1 reserved” messaging after semantic code paths exist.

### 6. Tests Required

- `tests/status_cli.rs`
  - Assert `status` remains informational for reserved or missing semantic substrates
  - Assert ready semantic modes require `vector_backend: sqlite_vec`
  - Assert missing vector backend produces `ready: false` and truthful active channel reporting
- `tests/runtime_gate_cli.rs`
  - Assert `ingest`, `search`, and `agent-search` succeed for ready semantic modes
  - Assert reserved backends still fail closed with updated error text
- `tests/retrieval_cli.rs`
  - Assert semantic modes fail closed when vector backend is missing
  - Assert ready semantic modes preserve dual-channel retrieval behavior

### 7. Wrong vs Correct

#### Wrong

- Mark `embedding_only` / `hybrid` as supported in retrieval code
- But leave `status` and `doctor` hardcoded to “reserved in Phase 1”
- Or mark them `ready` without checking `vector.backend`

#### Correct

- Treat semantic modes as supported runtime variants
- Gate operational readiness on backend, model, vector backend, and DB sidecar/index state
- Keep `status`, `doctor`, and CLI tests aligned with the actual retrieval/runtime contract
