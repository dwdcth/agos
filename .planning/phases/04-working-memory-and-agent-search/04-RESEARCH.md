# Phase 4: Working Memory And Agent Search - Research

**Researched:** 2026-04-16  
**Domain:** Working memory, value scoring, metacognitive gating, and thin Rig orchestration for a Rust local-first cognition engine  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Working Memory Structure
- **D-01:** `WorkingMemory` uses a strict typed structure with immutable fields and builder-style assembly.
- **D-02:** The core shape is `WorkingMemory { present: PresentFrame, branches: ActionBranch[] }`.
- **D-03:** Runtime working memory stays in-memory; persistence is only for debug/trace artifacts, not as the primary execution substrate.

### Candidate Actions And Value Scoring
- **D-04:** Candidate actions are fixed to three classes in Phase 4: `epistemic`, `instrumental`, and `regulative`.
- **D-05:** These three action classes coexist in the same `branches` field and compete inside one decision space.
- **D-06:** Value scoring uses five dimensions: goal progress, information gain, risk avoidance, resource efficiency, and agent robustness.
- **D-07:** The initial aggregation strategy is linear weighted combination.
- **D-08:** Weights come from a dynamic `ValueConfig`.
- **D-09:** The scoring design must leave room for later upgrade to multiplicative or more complex aggregation without breaking the typed contract.

### Rig Integration Boundary
- **D-10:** Rig is a thin orchestration adapter only.
- **D-11:** Rig may sequence calls across internal interfaces such as `Retriever`, `Assembler`, `Metacognition`, and `ActionSystem`.
- **D-12:** Cognitive-core logic stays inside `agent_memos`; Rig must not own attention logic, candidate generation, or veto semantics.

### Metacognitive Gate Behavior
- **D-13:** `warning` records diagnostic information and injects risk markers into working memory, but does not block decision flow.
- **D-14:** `veto` has two forms:
  - hard veto blocks output and returns a predefined safe response
  - soft veto forces insertion of a regulating candidate such as clarification or downgrade/reselection
- **D-15:** `escalate` triggers human intervention and pauses the autonomous loop.

### Claude's Discretion
- Exact field names and file/module split for working-memory and action models, as long as the typed structure and decision boundaries above are preserved.
- Exact `ValueConfig` storage shape and default weight values.
- Exact internal interface names for the thin Rig adapter, as long as Rig stays orchestration-only.
- Exact trace/debug persistence format for working-memory snapshots.

### Deferred Ideas (OUT OF SCOPE)
- multiplicative or more advanced value aggregation
- semantic retrieval execution
- autonomous rumination/write-back loops
- broader UI or remote orchestration surfaces
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COG-01 | System can assemble a working-memory object containing `world_fragments`, `self_state`, `active_goal`, `active_risks`, `candidate_actions`, and `metacog_flags`. [VERIFIED: `.planning/REQUIREMENTS.md`; CITED: `doc/0415-工作记忆.md`] | Use immutable `WorkingMemory { present, branches }` plus `WorkingMemoryBuilder`, where `PresentFrame` owns world/self/goal/risk/flag state and `branches` owns candidate-local evidence and scoring. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-工作记忆.md`] |
| COG-02 | Working memory can contain epistemic, operational, and regulatory candidate actions in the same decision field. [VERIFIED: `.planning/REQUIREMENTS.md`] | Normalize Phase 4 enums to locked labels `epistemic` / `instrumental` / `regulative`, and migrate plan/tests/docs away from the older `operational` / `regulatory` wording to avoid split semantics. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-工作记忆.md`] |
| COG-03 | System can score candidate actions with a multi-dimensional value representation before projecting them into a comparable decision score. [VERIFIED: `.planning/REQUIREMENTS.md`; CITED: `doc/0415-价值层.md`] | Model `ValueVector` and `ValueConfig` separately, compute per-dimension subscores first, and only then project with a linear weighted score. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-价值层.md`] |
| COG-04 | Metacognitive logic can inject warnings or veto flags when retrieval or candidate actions are too uncertain, risky, or under-supported. [VERIFIED: `.planning/REQUIREMENTS.md`; CITED: `doc/0415-元认知层.md`] | Use a typed `GateDecision` model with `warning`, `soft_veto`, `hard_veto`, and `escalate`; warnings mutate WM flags only, veto/escalate shape selection/output. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`] |
| AGT-02 | Developer can invoke a Rig-based agent-search workflow that performs multi-step retrieval and evidence gathering over the internal search services. [VERIFIED: `.planning/REQUIREMENTS.md`] | Keep Rig in `src/agent/` as the async adapter that sequences bounded retrieve/assemble/score/gate steps through internal traits and never bypasses `SearchService` or `TruthGovernanceService`. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html] |
| AGT-03 | Agent-search output includes citations and a structured working-memory or decision-support payload instead of a plain freeform answer only. [VERIFIED: `.planning/REQUIREMENTS.md`] | Reuse Phase 2 `SearchResponse` citations and traces inside `WorkingMemory` branches and expose a structured `AgentSearchReport` with selected branch, branch table, citations, and gate outcomes. [VERIFIED: `src/search/mod.rs`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`] |
| AGT-04 | Agent-search orchestration does not bypass ordinary retrieval services or write directly into shared truth without explicit gates. [VERIFIED: `.planning/REQUIREMENTS.md`] | Agent search may read from search/governance services only; all truth mutation remains in Phase 3 governance APIs and write-back remains Phase 5 scope. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] |
</phase_requirements>

## Summary

Phase 4 should land as a new cognitive service seam, not as a retrieval rewrite and not as a Rig-shaped framework pivot. Ordinary retrieval is already typed, cited, and filterable through `SearchService`; truth-layer governance is already typed and synchronous through `TruthGovernanceService`; Phase 4 should assemble those outputs into a front-stage control field, compare action branches, and gate the selected branch before any agent-facing answer is emitted. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md`; CITED: `doc/0415-00记忆认知架构.md`; CITED: `doc/0415-工作记忆.md`]

The implementation direction should therefore be: add a new `cognition` module for pure typed domain logic, add a small `agent` module for async Rig orchestration, keep retrieval and truth governance as dependencies rather than submodules, and keep runtime working memory entirely in memory with optional trace sinks for debugging only. This matches the locked phase constraints, the 0415 definition of working memory as a “current frame + near-term branches” control field, and the existing repository pattern where SQL stays behind typed services. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `.planning/research/ARCHITECTURE.md`; VERIFIED: `src/memory/repository.rs`; CITED: `doc/0415-工作记忆.md`]

**Primary recommendation:** 采用“同步 typed cognition core + 异步 thin Rig adapter”结构：`search/memory` 只提供证据与治理读接口，`cognition` 负责装配/评分/门控，`agent` 只负责多步编排与结构化输出。 [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Working-memory assembly | API / Backend [ASSUMED] | Database / Storage [ASSUMED] | WM is built from typed retrieval and governance reads, but the live object must remain in memory and only consume storage-backed evidence. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`] |
| Candidate generation and branch comparison | API / Backend [ASSUMED] | — | Candidate classes, value vectors, and branch scoring are cognition-core logic, not DB schema or client UI logic. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-工作记忆.md`; CITED: `doc/0415-价值层.md`] |
| Metacognitive gating | API / Backend [ASSUMED] | — | Warning/veto/escalate semantics supervise decision flow after scoring and before output, so they belong in the decision service. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`] |
| Debug/trace persistence | Database / Storage [ASSUMED] | API / Backend [ASSUMED] | Persistence is explicitly debug-only, so it should sit behind an optional sink and never become the execution substrate. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] |
| Rig orchestration | API / Backend [ASSUMED] | — | Rig should sequence tools/models around internal services and must not own cognitive semantics or durable memory state. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html] |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rig-core` | `0.35.0` [VERIFIED: `cargo search --registry crates-io rig-core`; VERIFIED: `cargo info --registry crates-io rig-core`] | Thin agent/model/tool orchestration | Official `AgentBuilder` supports tool registration, output schema, bounded turns, and context attachment, which is enough for Phase 4 orchestration without moving cognition logic out of this repo. [CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html] |
| `tokio` | `1.52.0` [VERIFIED: `cargo search --registry crates-io tokio`; VERIFIED: `cargo info --registry crates-io tokio`] | Async runtime for Rig-only boundary | Current codebase is synchronous, so Tokio should be introduced only at the `agent` seam to host provider calls and bounded multi-step orchestration. [VERIFIED: `Cargo.toml`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-PATTERNS.md`] |
| `tracing-subscriber` | `0.3.23` [VERIFIED: `cargo search --registry crates-io tracing-subscriber`; VERIFIED: `cargo info --registry crates-io tracing-subscriber`] | Structured trace sink for debug snapshots | Phase 4 needs explainable branch/gate traces; `tracing` already exists in the manifest, and `tracing-subscriber` is the minimal standard complement for JSON or human-readable diagnostics. [VERIFIED: `Cargo.toml`; VERIFIED: `cargo info --registry crates-io tracing-subscriber`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` / `serde_json` | existing `1.x` pins in manifest [VERIFIED: `Cargo.toml`] | Serialize debug snapshots and structured agent reports | Use for `WorkingMemorySnapshot`, `AgentSearchReport`, and optional trace files; do not serialize WM into authority tables for execution. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `Cargo.toml`] |
| `thiserror` / `anyhow` | existing `2.x` / `1.x` pins in manifest [VERIFIED: `Cargo.toml`] | Domain error enums and interface-boundary error wrapping | Keep typed cognition errors in `cognition`, and wrap them only at CLI/agent entrypoints. [VERIFIED: `Cargo.toml`; VERIFIED: existing `src/search/mod.rs`; VERIFIED: existing `src/memory/governance.rs`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rig-core` orchestration [VERIFIED: `.planning/PROJECT.md`] | Hand-rolled provider/tool glue [ASSUMED] | Hand-rolled glue keeps short-term control but duplicates model/tool orchestration the project already decided to standardize through Rig. [VERIFIED: `.planning/PROJECT.md`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html] |
| In-memory `WorkingMemory` + optional trace sink [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] | New SQLite working-memory tables [ASSUMED] | Persisting WM into authority storage would blur execution state with durable truth and create unnecessary migration/debug coupling in Phase 4. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-工作记忆.md`] |

**Installation:**
```bash
cargo add rig-core@0.35
cargo add tokio@1 --features macros,rt-multi-thread
cargo add tracing-subscriber@0.3 --features fmt,env-filter,json
```

**Version verification:** `rig-core 0.35.0`, `tokio 1.52.0`, and `tracing-subscriber 0.3.23` were verified against the current crate index on 2026-04-15 via `cargo search --registry crates-io` and `cargo info --registry crates-io`. [VERIFIED: local cargo registry queries]

## Architecture Patterns

### System Architecture Diagram

```text
Task / query / agent intent
    ↓
Phase 2 SearchService
    ↓ cited SearchResponse
Phase 3 TruthGovernanceService / repository reads
    ↓ typed truth snapshots / pending-review context
WorkingMemoryAssembler
    ↓ WorkingMemory { present, branches }
ValueScorer
    ↓ scored branches + ValueConfig
MetacognitionService
    ├── warning → enrich present.active_risks / metacog_flags
    ├── soft veto → inject/force regulative branch reselection
    ├── hard veto → safe response
    └── escalate → pause + request human
DecisionSelector
    ↓ AgentSearchReport
RigAdapter (optional async shell)
    ↓ structured answer with citations
CLI / library / future API surface
```

The key boundary is that Rig sits after the internal decision pipeline, not inside it. Retrieval and truth governance remain upstream evidence sources; metacognition remains the last gate before output. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-00记忆认知架构.md`; CITED: `doc/0415-元认知层.md`]

### Recommended Project Structure

```text
src/
├── cognition/                   # new pure Phase 4 domain layer
│   ├── mod.rs
│   ├── working_memory.rs        # WorkingMemory, PresentFrame, builder
│   ├── action.rs                # ActionKind, ActionCandidate, ActionBranch
│   ├── assembly.rs              # WorkingMemoryAssembler + retrieval integration DTOs
│   ├── value.rs                 # ValueVector, ValueConfig, ValueScorer
│   ├── metacog.rs               # alerts, gate outcomes, veto/escalate logic
│   └── report.rs                # AgentSearchReport / DecisionReport / trace DTOs
├── agent/                       # new async orchestration shell
│   ├── mod.rs
│   ├── rig_adapter.rs           # AgentBuilder wiring only
│   └── orchestration.rs         # bounded multi-step agent search loop
└── interfaces/cli.rs            # add agent-search entrypoint only after library path is stable
```

This split fits the current repository shape: `search` and `memory` are already typed service seams, `interfaces/cli.rs` is already a thin wrapper, and there is currently no `src/cognition` or `src/agent` module to constrain this naming choice. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/mod.rs`; VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `rtk rg --files src tests | sort`] 

### Pattern 1: Immutable Working Memory With Builder-Only Assembly

**What:** Use a mutable builder during assembly and return an immutable `WorkingMemory` value once all present-state and branch-state fields are filled. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-工作记忆.md`]

**When to use:** Every retrieve-score-gate cycle; rebuild the whole WM each turn instead of mutating old branches in place. [CITED: `doc/0415-工作记忆.md`]

**Example:**
```rust
// Source: recommended Phase 4 contract synthesized from locked context + 0415 docs. [ASSUMED]
pub struct WorkingMemory {
    pub present: PresentFrame,
    pub branches: Vec<ActionBranch>,
}

pub struct WorkingMemoryBuilder {
    present: PresentFrameBuilder,
    branches: Vec<ActionBranch>,
}

impl WorkingMemoryBuilder {
    pub fn build(self) -> WorkingMemory {
        WorkingMemory {
            present: self.present.build(),
            branches: self.branches,
        }
    }
}
```

### Pattern 2: Branch-Local Evidence And Scoring

**What:** Each `ActionBranch` should carry candidate kind, evidence references, preconditions, predicted effects, `ValueVector`, projected score, and gate outcome, so comparison stays local to the branch instead of being reconstructed from scattered side arrays. [CITED: `doc/0415-工作记忆.md`; CITED: `doc/0415-价值层.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]

**When to use:** For all three action classes, because the docs define “查 / 做 / 调” as competitors in one decision field rather than separate pipelines. [CITED: `doc/0415-工作记忆.md`; CITED: `doc/0415-价值层.md`]

**Example:**
```rust
// Source: recommended Phase 4 contract synthesized from locked context + 0415 docs. [ASSUMED]
pub enum ActionKind {
    Epistemic,
    Instrumental,
    Regulative,
}

pub struct ActionBranch {
    pub candidate: ActionCandidate,
    pub evidence_ids: Vec<String>,
    pub value: ValueAssessment,
    pub gate: GateDecision,
}
```

### Pattern 3: Thin Rig Adapter Over Internal Services

**What:** Build a Rig agent only to bound turns, attach tools/context, and demand structured output; keep retrieval, assembly, scoring, and gating behind internal Rust traits or services. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html]

**When to use:** Only for `agent-search` flows; ordinary retrieval must keep working without Rig. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `src/search/mod.rs`]

**Example:**
```rust
// Source: AgentBuilder docs + Phase 4 boundary decision. [CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html]
let agent = AgentBuilder::new(model)
    .tool(internal_search_tool)
    .tool(working_memory_tool)
    .default_max_turns(4)
    .output_schema::<AgentSearchReport>()
    .build();
```

### Anti-Patterns to Avoid

- **Working-memory-as-top-k:** Do not rename `SearchResponse.results` to WM; working memory must add self state, goal, risks, flags, and actionable branches. [VERIFIED: `src/search/mod.rs`; CITED: `doc/0415-工作记忆.md`]
- **Rig-owned cognition:** Do not move candidate generation, value scoring, or veto semantics into prompts or Rig tools; keep them in Rust domain code. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] 
- **Persistence-first WM:** Do not create durable WM state as the main execution substrate; Phase 4 runtime WM is in-memory only. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]
- **Read/write bypass:** Do not let agent search read raw SQL rows or write truth tables directly; use `SearchService` and `TruthGovernanceService` only. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`]

## Recommended Module / Model Direction

### Working-memory schema

- `PresentFrame` should own `world_fragments`, `self_state`, `active_goal`, `active_risks`, and `metacog_flags`, because those are the locked and documented front-stage fields; `branches` then carries the near-term futures. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `.planning/REQUIREMENTS.md`; CITED: `doc/0415-工作记忆.md`]  
- `world_fragments` should store trimmed retrieval evidence as typed fragments referencing Phase 2 citation IDs, not full copied records, so branch explanations stay linked to ordinary retrieval instead of duplicating memory content. [VERIFIED: `src/search/mod.rs`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`; ASSUMED]  
- `branches` should be `Vec<ActionBranch>` with one shared comparison surface regardless of action kind, because the 0415 docs explicitly define epistemic / instrumental / regulative competition inside one field. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-工作记忆.md`; CITED: `doc/0415-价值层.md`]  
- Keep `WorkingMemory` immutable after build; if retrieval or metacog modifiers change the scene, rebuild a fresh WM frame instead of mutating the old frame in place. [CITED: `doc/0415-工作记忆.md`]  

### Candidate-action model

- Model `ActionCandidate` separately from `ActionBranch`: candidate is “what to do”, branch is “what competes now”. This keeps generation decoupled from value/gate annotations. [CITED: `doc/0415-工作记忆.md`; ASSUMED]  
- Each candidate should minimally include `kind`, `intent`, `parameters`, `preconditions`, `expected_effects`, and `evidence_refs`, because the docs define candidate granularity as a parameter-filled, consequence-previewed semi-structured action. [CITED: `doc/0415-工作记忆.md`]  
- Standardize enum labels to `epistemic`, `instrumental`, `regulative` in code and explicitly migrate any remaining `operational` / `regulatory` wording in docs/tests during planning, otherwise planners and tests will fork on synonymous-but-not-identical variants. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  

### Value-scoring architecture

- Represent value in two layers: `ValueVector { goal_progress, information_gain, risk_avoidance, resource_efficiency, agent_robustness }` and `ProjectedScore { weighted_sum, weight_snapshot }`, so later aggregation upgrades do not break branch contracts. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-价值层.md`]  
- Make `ValueConfig` a runtime input to `ValueScorer`, not an enum baked into candidates, because the 0415 docs define weights as dynamic outputs of task/self/metacog state. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-价值层.md`]  
- Compute per-dimension scores before aggregation, keep them normalized, and store the exact weight vector used per decision report for replay/debugging. The docs treat value as vector-first and projection-second; the existing codebase also favors typed trace data over opaque ranking. [CITED: `doc/0415-价值层.md`; VERIFIED: `src/search/mod.rs`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`]  
- Do not add threshold-based hard exclusions to `ValueScorer` yet; threshold-like blocking belongs in `MetacognitionService` for Phase 4 because the locked gate semantics already define the blocking layer. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-价值层.md`; CITED: `doc/0415-元认知层.md`]  

### Metacognitive gate architecture

- `MetacognitionService` should evaluate `(WM, branch_scores, system_state)` and emit a typed result such as `GateDecision::Warning`, `SoftVeto`, `HardVeto`, or `Escalate`, mirroring the documented `MetaCheck(WM, V, S)` role. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`]  
- `warning` should append `active_risks` and `metacog_flags` but leave the branch list comparable. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  
- `soft_veto` should either inject a new regulative branch such as “clarify”, “downgrade”, or “pause and inspect”, or force reselection among existing branches while recording the rejected branch ID. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`; ASSUMED]  
- `hard_veto` should short-circuit selection and return a predefined safe response DTO with citations to the blocking risks or missing evidence. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED]  
- `escalate` should produce a paused report that explicitly asks for human input; it should not silently downgrade into a warning. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  

### Retrieval / truth-governance integration points

- Retrieval integration should consume existing `SearchService::search(&SearchRequest)` responses, because they already provide citations, filters, snippets, and score traces. [VERIFIED: `src/search/mod.rs`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`]  
- Truth integration should use `TruthGovernanceService` read APIs or `MemoryRepository` typed projections for layer-aware evidence, never raw SQL joins from the new cognition layer. [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`]  
- Phase 4 may read governance queue context to inform metacognitive risk or branch generation, but it must not perform promotion approval, ontology mutation, or write-back. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LLM/provider orchestration | Custom provider glue and ad hoc tool loop [ASSUMED] | `rig-core::agent::AgentBuilder` and internal tool adapters. [CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html] | Official Rig already provides tools, context injection, output schema, and bounded turns; custom glue would duplicate unstable integration code while the project already locked Rig as the orchestration layer. [VERIFIED: `.planning/PROJECT.md`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html] |
| Working-memory persistence | Durable WM authority tables [ASSUMED] | In-memory WM plus optional trace sink (`tracing` / JSON snapshot). [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED] | WM is explicitly runtime-only in this phase; durable tables would create schema churn for debug-only data and blur truth vs execution state. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] |
| Veto semantics | Boolean `blocked: bool` flags sprinkled across services [ASSUMED] | Typed gate outcomes with warning / soft veto / hard veto / escalate. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`] | Phase 4 semantics are richer than allow/deny; typed outcomes keep planner/test coverage aligned with the locked behavior matrix. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] |
| Retrieval reuse | Raw SQL or new search pipeline inside agent code [ASSUMED] | Existing `SearchService` and truth-governance reads. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`] | Reimplementing retrieval here would break the phase boundary and duplicate already-tested explainability/citation behavior. [VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] |

**Key insight:** Phase 4 complexity is in typed decision assembly and gating, not in inventing another search stack or another agent framework. The fastest safe path is to reuse Phase 2 and Phase 3 as immutable dependencies and only add the cognition/control layer above them. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`]

## Test Strategy

- Keep cognition-core tests library-first and deterministic: the main test surface should be pure Rust unit/integration tests over typed inputs, not provider-backed end-to-end flows. [VERIFIED: existing test style in `tests/*.rs`; VERIFIED: `cargo test -- --list`; ASSUMED]  
- Split tests by the current three-plan seams: `tests/working_memory_assembly.rs` for working-memory contracts and assembly, `tests/value_metacog.rs` for value projection plus gate behavior, and `tests/agent_search.rs` for bounded orchestration over fake services. [RESOLVED: aligned with current plan files]  
- Add fixture helpers that convert Phase 2 `SearchResponse` and Phase 3 `TruthRecord` values into assembly inputs, rather than inventing JSON fixtures divorced from the codebase contracts. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/truth.rs`; ASSUMED]  
- Treat provider-backed Rig smoke tests as optional and off by default because the environment currently has no `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`. [VERIFIED: local `printenv` probes]  

## Anti-Goals

- Do not re-implement ordinary retrieval, lexical rerank, or citation construction in Phase 4. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `src/search/mod.rs`]  
- Do not introduce semantic retrieval execution or `sqlite-vec` in this phase. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `.planning/PROJECT.md`]  
- Do not write back into shared truth, self-model, or rumination queues from agent search output. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  
- Do not collapse metacognition into “warning logs only”; `soft_veto`, `hard_veto`, and `escalate` are mandatory typed behaviors. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`]  
- Do not let Rig own the memory schema, working-memory structure, or value semantics. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  

## Migration Risks

### Risk 1: Action-class naming drift

**What goes wrong:** Existing roadmap/requirements still say `operational` / `regulatory`, while the locked Phase 4 context uses `instrumental` / `regulative`; code/tests/docs can diverge if planners implement both. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]  
**How to avoid:** Normalize code and plan artifacts to the locked `instrumental` / `regulative` naming at Phase 4 start, and explicitly update requirement wording in downstream docs/tests. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED]  

### Risk 2: Async boundary leak into stable synchronous services

**What goes wrong:** The current codebase is synchronous in `search` and `memory`; adding Tokio too low in the stack would force broad signature churn. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; VERIFIED: `Cargo.toml`]  
**How to avoid:** Keep `search`, `memory`, and `cognition` synchronous; add async only inside `src/agent` and interface entrypoints that actually talk to Rig/provider APIs. [VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-PATTERNS.md`; ASSUMED]  

### Risk 3: WM trace persistence turning into schema debt

**What goes wrong:** A debug-only trace requirement can tempt a migration for a new table that later becomes sticky production state. [VERIFIED: locked debug-only persistence in `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED]  
**How to avoid:** Start with `tracing` or JSON snapshot sinks behind a trait, and only add SQLite trace storage if later verification proves file traces insufficient. [ASSUMED]  

### Risk 4: Rig bypassing existing evidence contracts

**What goes wrong:** If Rig tools fetch raw records or freeform prompt context directly, Phase 2 citations and Phase 3 governance semantics stop being authoritative. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`; ASSUMED]  
**How to avoid:** Make the only Rig-visible tools wrap internal service methods that already return typed, cited structures. [VERIFIED: `.planning/PROJECT.md`; ASSUMED]  

## Common Pitfalls

### Pitfall 1: 把工作记忆当检索结果列表
**What goes wrong:** 系统能“找出来”，但不能稳定比较行动、携带自我状态或传播风险。 [CITED: `doc/0415-工作记忆.md`]  
**Why it happens:** 当前仓库已有成熟 `SearchResponse`，最容易偷懒把它直接升级成 WM。 [VERIFIED: `src/search/mod.rs`]  
**How to avoid:** 强制引入 `PresentFrame` 与 `ActionBranch` 两级模型，并让 assembly 输出与检索输出是不同类型。 [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED]  
**Warning signs:** 新模块里只有 `Vec<SearchResult>`，没有 `self_state`、`active_risks`、`metacog_flags`。 [VERIFIED: `.planning/REQUIREMENTS.md`; ASSUMED]  

### Pitfall 2: 把价值层提前压成一个裸分数
**What goes wrong:** 三类异质行动失去可解释比较基础，后续聚合器升级会破坏 API。 [CITED: `doc/0415-价值层.md`]  
**Why it happens:** 标量排序实现最省事。 [ASSUMED]  
**How to avoid:** 先存 `ValueVector`，再存投影后的 `ProjectedScore` 与权重快照。 [CITED: `doc/0415-价值层.md`; ASSUMED]  
**Warning signs:** 候选行动模型只有 `score: f32`，没有维度分量。 [ASSUMED]  

### Pitfall 3: 把元认知退化成日志
**What goes wrong:** 高风险或证据不足分支仍然会因为高分直接出线。 [CITED: `doc/0415-元认知层.md`]  
**Why it happens:** logging 比 typed gate 更容易接现有流程。 [ASSUMED]  
**How to avoid:** 用 `GateDecision` 和 `DecisionSelector` 明确编码 warning / soft veto / hard veto / escalate。 [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED]  
**Warning signs:** 所有 metacog 结果都只是 `Vec<String>` 或 `warn!()`。 [ASSUMED]  

### Pitfall 4: 让 Rig 成为认知内核
**What goes wrong:** 模型提示词决定领域语义，Rust 代码只剩壳层，测试会迅速失真。 [VERIFIED: `.planning/PROJECT.md`; ASSUMED]  
**Why it happens:** Phase 4 很容易被实现成“LLM chat over search”。 [VERIFIED: `.planning/research/PITFALLS.md`; ASSUMED]  
**How to avoid:** Rig 只拿到内部 typed 工具和 structured output schema；认知规则仍在本仓库服务层。 [VERIFIED: `.planning/PROJECT.md`; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html]  
**Warning signs:** `agent/` 目录里直接出现候选行动分类、打分公式或 veto 逻辑。 [ASSUMED]  

## Code Examples

Verified patterns from official and in-repo sources:

### Reuse ordinary retrieval as a typed upstream dependency
```rust
// Source: src/search/mod.rs [VERIFIED: repo file]
pub struct SearchService<'db> {
    lexical: lexical::LexicalSearch<'db>,
}

impl<'db> SearchService<'db> {
    pub fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SearchError> {
        let candidates = self.lexical.recall(request)?;
        let scored = score::score_candidates(request, candidates);
        Ok(rerank::rerank_results(request, scored)?)
    }
}
```

### Keep governance orchestration above repository CRUD
```rust
// Source: src/memory/governance.rs [VERIFIED: repo file]
pub struct TruthGovernanceService<'db> {
    repository: MemoryRepository<'db>,
}

impl<'db> TruthGovernanceService<'db> {
    pub fn create_promotion_review(
        &self,
        request: CreatePromotionReviewRequest,
    ) -> Result<PromotionReviewReport, TruthGovernanceError> {
        self.require_active_t3_source(&request.source_record_id)?;
        // ...
        self.review_report(&review.review_id)
    }
}
```

### Use Rig for bounded structured orchestration
```rust
// Source: AgentBuilder docs [CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html]
let agent = AgentBuilder::new(model)
    .context("phase-4-agent-search")
    .default_max_turns(4)
    .tool(tool)
    .output_schema::<AgentSearchReport>()
    .build();
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| “Agent search” as freeform chat over search results [ASSUMED] | Typed WM assembly, branch scoring, and metacognitive gating over cited retrieval results. [CITED: `doc/0415-工作记忆.md`; CITED: `doc/0415-元认知层.md`] | Locked for Phase 4 on 2026-04-15. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] | Keeps decision semantics testable and explainable. [ASSUMED] |
| Framework-owned memory model [ASSUMED] | Repo-owned memory/truth/search model with Rig as orchestration adapter. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] | Locked since project init and reaffirmed for Phase 4. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/STATE.md`] | Preserves local domain control and prevents retrieval/cognition collapse. [ASSUMED] |
| Single scalar action score first [ASSUMED] | Multi-dimensional `ValueVector` first, linear weighted projection second. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-价值层.md`] | Locked for initial Phase 4 aggregation. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] | Allows later aggregation upgrades without changing branch shape. [ASSUMED] |

**Deprecated/outdated:**
- Treating `operational` / `regulatory` as Phase 4 canonical enum labels is outdated relative to the locked context and should be migrated before implementation starts. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | JSON/tracing-based WM trace sinks are sufficient for Phase 4, so no SQLite migration is needed initially. [ASSUMED] | Standard Stack / Migration Risks | If trace volume or replay needs are higher than expected, planner must add a trace-storage task. |
| A2 | `self_state` will be assembled through a minimal `SelfStateProvider` seam that combines runtime task context, capability/readiness flags, and selected truth-layer facts, without adding a dedicated persistent self-model subsystem in Phase 4. [RESOLVED: planner revision 1, 2026-04-16] | Recommended Module / Model Direction | If later phases need richer self-model persistence, add it as separate scoped work instead of widening Phase 4. |
| A3 | Provider-backed Rig smoke tests remain optional because Phase 4 correctness is primarily in internal orchestration, citations, and typed outputs; the required gate is deterministic local coverage. [RESOLVED: planner revision 1, 2026-04-16] | Test Strategy / Environment Availability | If the user later wants a real provider demo, add opt-in env setup and ignored smoke coverage without making it a blocker. |

## Open Questions (RESOLVED)

1. **最小 `self_state` provider 是否在 Phase 4 落地？**  
Decision: 是。Phase 4 明确采用一个最小 `SelfStateProvider` seam，并把它放进 working-memory assembly 计划。  
Chosen shape: provider 产出一个 typed runtime snapshot，最少包含当前任务/goal 上下文、agent capability/readiness flags、以及来自现有 truth/governance 读取面的选定事实；它不引入新的 SQLite 表、不引入独立 `self_model` 子系统，也不把 `self_state` 退化成无类型 JSON。 [RESOLVED: aligned with `04-01-PLAN.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `.planning/REQUIREMENTS.md`]  
Planning consequence: `04-01` 必须在 `src/cognition/assembly.rs` 明确实现 `SelfStateProvider` 并通过测试证明 `self_state` 来自 runtime-plus-truth assembly，而不是持久化自我模型。 [RESOLVED]

2. **Phase 4 是否要求真实 provider 的 Rig smoke test？**  
Decision: 不要求。Live Rig smoke 在 Phase 4 是可选信心检查，不属于必过执行门槛。  
Why: 当前环境没有 provider API keys，而且 Phase 4 的核心正确性是检验内部 retrieve -> assemble -> score -> gate 编排、citation 保留、以及边界不越权，这些都可以由本地 deterministic tests 覆盖。 [VERIFIED: local `printenv` probes; VERIFIED: `.planning/REQUIREMENTS.md`]  
Planning consequence: `04-03` 的 required verification 只包含本地 deterministic `cargo test` / `clippy`；如果未来配置了 provider credentials，可以追加 ignored/manual smoke，但不能把 live smoke 设成计划阻塞项。 [RESOLVED]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `rustc` | Phase 4 code/test build | ✓ [VERIFIED: local probe] | `1.94.1` [VERIFIED: local probe] | — |
| `cargo` | Build/test/dependency install | ✓ [VERIFIED: local probe] | `1.94.1` [VERIFIED: local probe] | — |
| `sqlite3` CLI | Manual DB inspection only | ✗ [VERIFIED: local probe] | — | Use bundled `rusqlite` tests and repository/service assertions. [VERIFIED: `Cargo.toml`] |
| crate registry access | Add `rig-core` / `tokio` / `tracing-subscriber` | ✓ [VERIFIED: `cargo search --registry crates-io`; VERIFIED: `cargo info --registry crates-io`] | current index reachable on 2026-04-15 [VERIFIED: local cargo registry queries] | — |
| LLM provider API keys | Optional live Rig smoke tests only | ✗ [VERIFIED: local `printenv` probes] | — | Use fake/mocked orchestration tests as the required Phase 4 gate. [RESOLVED] |

**Missing dependencies with no fallback:**
- None for library-first Phase 4 implementation. [ASSUMED]

**Missing dependencies with fallback:**
- `sqlite3` CLI is absent, but this does not block implementation because the project already uses bundled `rusqlite`. [VERIFIED: local probe; VERIFIED: `Cargo.toml`]
- Provider API keys are absent, but mocked agent-search tests remain viable. [VERIFIED: local `printenv` probes; ASSUMED]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test`. [VERIFIED: `cargo test -- --list`] |
| Config file | `Cargo.toml`. [VERIFIED: repository root listing] |
| Quick run command | `rtk cargo test --test working_memory_assembly -- --nocapture` after Phase 4 tests exist. [RESOLVED: aligned with `04-VALIDATION.md`] |
| Full suite command | `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings`. [VERIFIED: prior phase summaries; RESOLVED for continued Phase 4 use] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COG-01 | Assemble immutable WM with present frame + branches | integration | `rtk cargo test --test working_memory_assembly working_memory_builder_requires_present_frame_and_uses_phase4_action_labels -- --nocapture` | ❌ Wave 0 [RESOLVED] |
| COG-02 | Compare `epistemic` / `instrumental` / `regulative` branches in one field | integration | `rtk cargo test --test working_memory_assembly working_memory_builder_requires_present_frame_and_uses_phase4_action_labels -- --nocapture` | ❌ Wave 0 [RESOLVED] |
| COG-03 | Compute vector-first scoring, then project with `ValueConfig` | integration | `rtk cargo test --test value_metacog value_scorer_projects_five_dimensions_with_dynamic_weights -- --nocapture` | ❌ Wave 0 [RESOLVED] |
| COG-04 | warning / soft veto / hard veto / escalate behavior | integration | `rtk cargo test --test value_metacog metacog_gates_warn_veto_and_escalate_with_typed_reports -- --nocapture` | ❌ Wave 0 [RESOLVED] |
| AGT-02 | Rig adapter performs bounded multi-step retrieval over internal services | integration | `rtk cargo test --test agent_search orchestrator_reuses_internal_services_and_returns_structured_report -- --nocapture` | ❌ Wave 0 [RESOLVED] |
| AGT-03 | Agent-search returns structured report with citations and WM payload | integration | `rtk cargo test --test agent_search rig_adapter_stays_thin_and_never_bypasses_search_or_truth_gates -- --nocapture` | ❌ Wave 0 [RESOLVED] |
| AGT-04 | Agent-search never bypasses retrieval/governance or mutates truth directly | integration | `rtk cargo test --test agent_search rig_adapter_stays_thin_and_never_bypasses_search_or_truth_gates -- --nocapture` | ❌ Wave 0 [RESOLVED] |

### Sampling Rate

- **Per task commit:** targeted Phase 4 test file plus any touched existing regression file. [ASSUMED]
- **Per wave merge:** `rtk cargo test --tests`. [VERIFIED: existing project practice in prior summaries; ASSUMED]
- **Phase gate:** `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings` green before `/gsd-verify-work`. [VERIFIED: existing project practice in prior summaries; ASSUMED]

### Wave 0 Gaps

- [ ] `tests/working_memory_assembly.rs` — covers COG-01 and COG-02 plus the minimal `self_state` provider seam. [RESOLVED]
- [ ] `tests/value_metacog.rs` — covers COG-03 and COG-04. [RESOLVED]
- [ ] `tests/agent_search.rs` — covers AGT-02/03/04 with fake retriever/assembler/gate services. [RESOLVED]
- [ ] `src/cognition/` and `src/agent/` module exports in `src/lib.rs`. [VERIFIED: `src/lib.rs`; ASSUMED]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [ASSUMED] | Local CLI/library phase has no auth surface yet. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `src/interfaces/cli.rs`] |
| V3 Session Management | no [ASSUMED] | No session/cookie/web auth surface exists in the current project. [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `.planning/ROADMAP.md`] |
| V4 Access Control | yes [ASSUMED] | Treat metacognitive veto and no-direct-writeback boundaries as internal authorization rules over unsafe actions and truth mutation. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `.planning/ROADMAP.md`] |
| V5 Input Validation | yes [ASSUMED] | Validate action kind enums, branch parameters, and requested tool inputs with typed Rust models and explicit error enums. [VERIFIED: existing enum/error style in `src/memory/truth.rs`; VERIFIED: `src/memory/governance.rs`] |
| V6 Cryptography | no [ASSUMED] | Phase 4 introduces no new crypto requirement if live provider auth remains external env configuration. [VERIFIED: local `printenv` probes; ASSUMED] |

### Known Threat Patterns for this phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Retrieved text tries to steer agent behavior beyond evidence scope. [ASSUMED] | Tampering | Keep retrieval as cited evidence only, require branch generation/scoring/gating in Rust, and never let retrieved text directly define final action classes. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED] |
| High-score branch hides missing preconditions or under-supported evidence. [CITED: `doc/0415-元认知层.md`] | Elevation of Privilege | Run metacognitive checks after scoring and before output, with soft/hard veto support. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; CITED: `doc/0415-元认知层.md`] |
| Agent orchestration writes truth directly or bypasses governance. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`] | Tampering | Route all truth writes through Phase 3 governance only and keep Phase 4 output read-only with respect to durable truth. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`] |
| Debug traces leak more content than necessary. [ASSUMED] | Information Disclosure | Persist trace snapshots only behind an optional sink and prefer citation IDs / record IDs over duplicating full corpus text where possible. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; ASSUMED] |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md` - locked Phase 4 decisions, discretion, and out-of-scope boundaries. [VERIFIED: repo file]
- `.planning/ROADMAP.md` - Phase 4 goal, requirements list, and plan slots. [VERIFIED: repo file]
- `.planning/REQUIREMENTS.md` - `COG-*` and `AGT-*` acceptance requirements. [VERIFIED: repo file]
- `src/search/mod.rs` - current ordinary retrieval boundary and result contract. [VERIFIED: repo file]
- `src/memory/governance.rs` / `src/memory/repository.rs` / `src/memory/truth.rs` - current truth-governance service/repository/type seams. [VERIFIED: repo files]
- `doc/0415-工作记忆.md` - working-memory ontology and current-frame/near-term-branch semantics. [VERIFIED: repo file]
- `doc/0415-价值层.md` - five-dimensional value vector and dynamic weight model. [VERIFIED: repo file]
- `doc/0415-元认知层.md` - warning/veto/escalate supervision semantics. [VERIFIED: repo file]
- `Cargo.toml` - current manifest shows no existing `tokio`, `rig-core`, or `tracing-subscriber`. [VERIFIED: repo file]
- `cargo search --registry crates-io rig-core`, `tokio`, `tracing-subscriber` - current published versions. [VERIFIED: local registry queries]
- `cargo info --registry crates-io rig-core`, `libsimple`, `tokio`, `tracing-subscriber` - crate metadata and official documentation links. [VERIFIED: local registry queries]
- https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html - official Rig builder API for tools, structured output, and bounded turns. [CITED]

### Secondary (MEDIUM confidence)
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md` and `02-03-SUMMARY.md` - confirm Phase 2 retrieval/service boundary and structured citation contract. [VERIFIED: repo files]
- `.planning/phases/03-truth-layer-governance/03-01-SUMMARY.md`, `03-02-SUMMARY.md`, `03-03-SUMMARY.md` - confirm additive governance evolution and queue/service patterns. [VERIFIED: repo files]
- `.planning/research/ARCHITECTURE.md` and `.planning/research/PITFALLS.md` - prior project-level architectural and pitfall framing. [VERIFIED: repo files]

### Tertiary (LOW confidence)
- None. [VERIFIED: this research avoided unverified external ecosystem claims beyond official crate metadata/docs]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - current versions and docs were verified with local cargo registry queries and official Rig docs. [VERIFIED: local cargo registry queries; CITED: https://docs.rs/rig-core/latest/rig/agent/struct.AgentBuilder.html]
- Architecture: HIGH - the recommendation aligns with locked Phase 4 decisions and the current codebase seams already present in `search` and `memory`. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/memory/governance.rs`]
- Pitfalls: HIGH - major risks are directly evidenced by terminology drift, current sync boundaries, and explicit phase scope locks. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `Cargo.toml`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]

**Research date:** 2026-04-15  
**Valid until:** 2026-05-15 for repo-internal findings; re-verify crate versions and Rig docs if planning starts after that date. [VERIFIED: research date; ASSUMED]

## RESEARCH COMPLETE
