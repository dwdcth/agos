# Cognition-Retrieval Contracts

> Executable contracts for features that cross cognition, ordinary retrieval, and agent-search seams.

---

## Scenario: Attention State Retrieval Bias

### 1. Scope / Trigger

- Trigger: Phase 11 added an explicit attention-state layer that changes shared request contracts and result-trace shape across cognition, retrieval, working-memory assembly, and agent-search.
- Why this needs code-spec depth: this is a cross-layer contract change, not a local scoring tweak.

### 2. Signatures

- `src/cognition/attention.rs`
  - `AttentionCue { cue: String, weight: f32 }`
  - `AttentionBaseline`
  - `AttentionDelta`
  - `AttentionState { baseline: AttentionBaseline, delta: AttentionDelta }`
  - `AttentionTrace { total_bonus: f32, contributions: Vec<AttentionContribution> }`
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
