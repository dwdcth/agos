# Phase 7: Follow-up Evidence Integration - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** Milestone audit gaps, Phase 4 agent-search artifacts, and zero-friction discuss defaults derived from the current cognition pipeline

<domain>
## Phase Boundary

Phase 7 closes the remaining cognition integration gap in agent-search. It is responsible for:
- folding follow-up retrieval evidence back into the assembled working-memory state
- making branch scoring and metacognitive gating observe follow-up evidence instead of only the primary query context
- keeping the structured `AgentSearchReport` aligned with the evidence that actually influenced decision selection

This phase is not responsible for:
- introducing semantic retrieval or changing the lexical-first baseline
- redesigning the runtime gate work completed in Phase 6
- changing the overall Rig boundary or moving cognition logic out of `agent_memos`
- expanding into rumination, UI, remote orchestration, or broader product surfaces

</domain>

<decisions>
## Implementation Decisions

### Evidence Integration Boundary
- **D-01:** Follow-up retrieval evidence must be merged into the same working-memory execution object that downstream scoring and gate evaluation consume.
- **D-02:** Follow-up evidence must not remain report-only data in `retrieval_steps` / top-level `citations` after this phase.
- **D-03:** The primary query and follow-up queries may remain distinguishable in trace/report metadata, but their returned evidence must share one cognition path before decision selection.

### Working Memory Semantics
- **D-04:** The existing `WorkingMemory { present, branches }` contract remains the runtime container; Phase 7 extends assembly inputs, not the high-level cognitive boundary itself.
- **D-05:** Follow-up evidence should enter `present.world_fragments` or an equivalent already-consumed evidence field inside the assembled runtime state, not a second side channel visible only to reporting.
- **D-06:** Evidence provenance and citation detail must stay attached when follow-up evidence is merged, so explainability is not reduced by the integration.

### Scoring And Gate Visibility
- **D-07:** Value scoring must see the merged follow-up evidence when comparing branches.
- **D-08:** Metacognitive gating must also evaluate against the merged evidence set, so veto/warning decisions cannot diverge from the evidence shown in the final report.
- **D-09:** Phase 7 must not create a special-case “follow-up-only” scoring or gate path; the point is one coherent decision surface.

### Report Contract
- **D-10:** `AgentSearchReport` should continue to expose `retrieval_steps` and citations for traceability, but the report must no longer imply that evidence was used if it never affected working memory or decision logic.
- **D-11:** Structured output should make it possible to tell that follow-up evidence was integrated, without flattening away query-step traceability.

### the agent's Discretion
- Exact field additions or helper structs used to annotate integrated evidence provenance.
- Exact merge policy for deduplicating overlapping primary and follow-up citations, as long as evidence is not silently lost.
- Exact test decomposition across working-memory assembly tests and agent-search integration tests.

</decisions>

<specifics>
## Specific Ideas

- Treat this phase as a cognition-path repair, not as “more retrieval”.
- Prefer extending the existing assembler/orchestrator seam over inventing a parallel follow-up evidence store.
- Keep the user-visible invariant simple: if evidence appears in `AgentSearchReport` as materially relevant, it should have actually participated in working-memory assembly and decision selection.
- Phase 7 should feel like “reported evidence and used evidence are finally the same thing”.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and audit target
- `.planning/ROADMAP.md` — Phase 7 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` — `COG-01`, `AGT-02`, and `AGT-03` are pending here
- `.planning/v1.0-MILESTONE-AUDIT.md` — exact follow-up-evidence integration gap this phase must close
- `.planning/PROJECT.md` — lexical-first baseline, local-first constraint, and explainability contract
- `.planning/STATE.md` — current milestone position and prior decisions

### Prior phase outputs
- `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md` — locked working-memory, value, Rig, and gate decisions that Phase 7 must preserve
- `.planning/phases/04-working-memory-and-agent-search/04-01-SUMMARY.md` — initial working-memory assembly seam
- `.planning/phases/04-working-memory-and-agent-search/04-02-SUMMARY.md` — scoring and metacognitive gate contracts
- `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md` — bounded agent-search orchestration and structured report behavior
- `.planning/phases/06-runtime-gate-enforcement/06-CONTEXT.md` — recent command-boundary tightening that should remain orthogonal to cognition repair

### Runtime code seams
- `src/agent/orchestration.rs` — current follow-up query execution, `retrieval_steps`, and `AgentSearchReport`
- `src/cognition/assembly.rs` — working-memory assembly seam
- `src/cognition/working_memory.rs` — runtime evidence container and present-state fields
- `src/cognition/value.rs` — branch scoring over assembled working memory
- `src/cognition/metacog.rs` — gate evaluation over assembled working memory and scored branches
- `tests/agent_search.rs` — existing orchestration/report regression surface
- `tests/working_memory_assembly.rs` — existing assembly regression surface

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/agent/orchestration.rs` already executes bounded follow-up queries and preserves per-step citations, so Phase 7 should reuse that retrieval sequencing rather than inventing another loop.
- `src/cognition/assembly.rs` already owns the typed transition from retrieved evidence into `WorkingMemory`, making it the natural integration seam.
- `src/cognition/value.rs` and `src/cognition/metacog.rs` already consume one assembled working-memory object, which is exactly why follow-up evidence must be merged before they run.

### Established Patterns
- Typed runtime objects first, report rendering second.
- One cognition path, not duplicated “report path” vs “decision path”.
- Rig remains a thin outer boundary over internal services.
- Lexical-first retrieval and explicit citations remain non-negotiable.

### Integration Points
- `AgentSearchRequest.follow_up_queries` and the resulting `retrieval_steps` are already present in orchestration; the missing seam is how those results feed `WorkingMemoryRequest` / assembler inputs.
- `collect_unique_citations(...)` and report rendering should remain trace surfaces, but they should now reflect evidence that also entered cognition.
- Regression coverage should prove consistency across:
  - merged working-memory evidence
  - branch scoring / gate decisions
  - final structured report

</code_context>

<deferred>
## Deferred Ideas

- semantic follow-up retrieval
- adaptive or learned query-step weighting
- UI visualization of evidence provenance in working memory
- remote/multi-tenant follow-up evidence routing

</deferred>

---

*Phase: 07-follow-up-evidence-integration*
*Context gathered: 2026-04-16*
