# Attention State Design Notes

## Why this phase exists

The current system already has retrieval, working memory, value scoring, metacognitive gating, and rumination. What it does not yet have is an explicit attention layer that explains why the same query should recall different memories under different cognitive states.

## Theory constraints from `doc/`

- Attention is not just a query string; it is a control state that decides what deserves foreground recall.
- The theory distinguishes a slow-changing baseline from a request-local delta.
- Self-model and metacognition should influence attention through modifiers rather than by rewriting truth.
- Working memory is downstream of attention, so attention should alter what gets assembled into `world_fragments`.

## Current codebase constraints

- `SearchRequest` is the shared ordinary retrieval contract and is already reused by agent-search.
- Retrieval explainability currently lives in the lexical/rerank/trace path.
- `WorkingMemoryRequest` already carries goal, risk, readiness, and metacognitive fields that can seed an attention delta.
- The codebase strongly prefers additive typed seams over rewriting existing contracts.

## Recommended design for Phase 11

### 1. Add an explicit attention module

Introduce a new module under `src/cognition/` for:

- `AttentionBaseline`
- `AttentionDelta`
- `AttentionState`
- a derived lightweight bias summary that retrieval can consume deterministically

### 2. Extend the ordinary retrieval contract

Add optional attention-state input to `SearchRequest` so all retrieval callers can opt in without a parallel API.

### 3. Apply attention in scoring/rerank, not in raw SQL recall

Do not hide attention inside SQL filtering first. The better seam is Rust-side scoring/rerank because:

- it is easier to explain in traces
- it avoids corrupting lexical recall semantics
- it remains compatible with lexical-only, embedding-only, and hybrid modes

### 4. Seed attention from working-memory/agent-search requests

`WorkingMemoryRequest` and `AgentSearchRequest` already carry:

- active goal
- active risks
- metacognitive flags
- readiness/capability hints

These should feed an attention delta when callers do not provide one explicitly.

## Non-goals for this phase

- No full emotion modulation engine
- No persistent session baseline manager
- No attention-aware MCP/API surface
- No self-model or world-model redesign in this phase

## Risks to avoid

- Do not create a second retrieval path outside `SearchService`.
- Do not let attention bypass lexical-first explainability.
- Do not encode complex policy into opaque heuristics with no trace output.
- Do not make attention required for normal retrieval.
