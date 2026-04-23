# Phase 7: Follow-up Evidence Integration - Research

**Researched:** 2026-04-16  
**Domain:** agent-search follow-up evidence integration, working-memory assembly, decision consistency  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
- semantic follow-up retrieval
- adaptive or learned query-step weighting
- UI visualization of evidence provenance in working memory
- remote/multi-tenant follow-up evidence routing
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COG-01 | System can assemble a working-memory object containing `world_fragments`, `self_state`, `active_goal`, `active_risks`, `candidate_actions`, and `metacog_flags`. [VERIFIED: `.planning/REQUIREMENTS.md`] | Phase 4 already assembles `WorkingMemory`, but the current assembler only searches `request.query`; Phase 7 must merge follow-up retrieval evidence into `present.world_fragments` so the assembled cognitive state matches the full agent-search evidence set. [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`] |
| AGT-02 | Developer can invoke a Rig-based agent-search workflow that performs multi-step retrieval and evidence gathering over the internal search services. [VERIFIED: `.planning/REQUIREMENTS.md`] | The workflow already performs bounded multi-step retrieval, but follow-up steps only populate `retrieval_steps`; they do not affect assembly or decision. Phase 7 should preserve the bounded loop while integrating those results into the cognition path. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md`] |
| AGT-03 | Agent-search output includes citations and a structured working-memory or decision-support payload instead of a plain freeform answer only. [VERIFIED: `.planning/REQUIREMENTS.md`] | The report is already structured and cited, but it can still cite follow-up evidence that never influenced `working_memory` or `decision`; Phase 7 must remove that inconsistency. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`] |
</phase_requirements>

## Summary

The actual gap is narrower than a full “agent-search redesign.” `AgentSearchOrchestrator::execute(...)` already loops over the primary query plus bounded follow-up queries, but it only converts those extra queries into `RetrievalStepReport` and top-level deduplicated citations. It then calls `assembler.assemble(&request.working_memory)` using the original `WorkingMemoryRequest`, which means the assembler performs only one search against the primary query. Scoring and gating therefore never see the follow-up evidence. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`]

The safest design is to keep one working-memory contract and extend the assembler input rather than inventing a second “follow-up evidence” channel. The current `WorkingMemoryRequest` already carries query, filters, action seeds, risks, flags, and local adaptation entries. Phase 7 should add an explicit evidence-merge seam here so `WorkingMemoryAssembler` can combine the primary query search result with externally supplied follow-up `SearchResult` values before materializing `world_fragments` and branch supporting evidence. [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/cognition/working_memory.rs`]

**Primary recommendation:** introduce a typed “retrieved evidence override / extension” path into `WorkingMemoryRequest` and `WorkingMemoryAssembler`, then have `AgentSearchOrchestrator` pass the follow-up search results into that path before scoring and gating. This preserves the existing retrieval loop, keeps Rig thin, keeps one cognition path, and makes `AgentSearchReport.working_memory`, `decision`, and `citations` describe the same evidence set. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `.planning/phases/07-follow-up-evidence-integration/07-CONTEXT.md`; ASSUMED]

## Current Code Findings

### 1. The audit gap is mechanically real in orchestration

- `AgentSearchRequest::bounded_queries()` already returns the primary query plus bounded follow-up queries. [VERIFIED: `src/agent/orchestration.rs`]
- `execute(...)` runs retrieval for each query and builds `retrieval_steps`. [VERIFIED: `src/agent/orchestration.rs`]
- The assembler is then called only with `request.working_memory`, which still contains only the original primary query state. [VERIFIED: `src/agent/orchestration.rs`]

**Implication:** Phase 7 can be closed by changing how orchestration feeds the assembler, not by adding a new outer loop.

### 2. `WorkingMemoryAssembler` currently owns search + materialization together

- `WorkingMemoryAssembler::assemble(...)` internally constructs a `SearchRequest` from `WorkingMemoryRequest.query` and `filters`, runs `SearchService::search(...)`, then turns those results into `EvidenceFragment`s and `TruthRecord`s. [VERIFIED: `src/cognition/assembly.rs`]
- `present.world_fragments` and branch `supporting_evidence` are both sourced from those materialized fragments. [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/cognition/working_memory.rs`]

**Implication:** To make follow-up evidence affect both present-state and branches, the merge has to happen before or during fragment materialization, not later in reporting.

### 3. Scoring and gating already consume one coherent runtime object

- `WorkingMemoryScoringPort::score(...)` reads only `working_memory.branches`. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `src/cognition/value.rs`]
- `MetacognitionService::evaluate(...)` reads only `working_memory` plus `scored_branches`. [VERIFIED: `src/cognition/metacog.rs`]

**Implication:** Once follow-up evidence is truly merged into the assembled runtime object and branch evidence, scoring and gating will naturally see it without their own special-case API changes.

## Recommended Integration Direction

### Working-memory request shape

Recommended exact direction:

- Add a typed field to `WorkingMemoryRequest` for externally supplied retrieved results or pre-materialized evidence input.
- Keep the original `query` field because the primary query remains part of the assembly identity and trace.
- Add a small builder helper such as:
  - `with_follow_up_results(...)`
  - or `with_retrieved_results(...)`

Avoid raw JSON or report-only annotations here; the assembler needs typed search/result data with citations, traces, and scores intact. [ASSUMED; VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/search/mod.rs`]

### Assembler behavior

Recommended exact behavior:

- If no external results are supplied, preserve current behavior: perform one search from `query` + `filters`.
- If external results are supplied, merge primary-query and follow-up results into one result set before truth projection and fragment materialization.
- Deduplicate by `record_id` / citation record identity so overlapping primary and follow-up recall does not duplicate fragments.
- Preserve per-result `trace.matched_query` so downstream report/debug can still tell which query surfaced each fragment.

This keeps Phase 7 additive and backward-compatible for ordinary assembly callers. [ASSUMED]

### Agent-search report alignment

Recommended exact target state:

- `retrieval_steps` remain per-query trace reports.
- `citations` remain deduplicated across all retrieval steps.
- `working_memory.present.world_fragments` becomes the deduplicated, integrated evidence set from primary + follow-up retrieval.
- `decision.selected_branch` and gate diagnostics are therefore based on the same evidence set exposed in the report.

This satisfies the audit’s “reported evidence and used evidence must match” requirement. [VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`]

## Testing Direction

### Best regression split

Use two layers:

1. **Assembly-level regression** in `tests/working_memory_assembly.rs`
   - prove integrated follow-up evidence ends up in `present.world_fragments`
   - prove supporting branch evidence can reference integrated fragments

2. **Agent-search integration regression** in `tests/agent_search.rs`
   - prove follow-up query citations also appear in `working_memory`
   - prove scoring/gate output and structured report now align on the same evidence set

This follows the repo’s current testing pattern: narrow seam tests plus end-to-end orchestrator tests. [VERIFIED: `tests/working_memory_assembly.rs`; VERIFIED: `tests/agent_search.rs`]

## Patterns To Reuse

### Pattern 1: Typed runtime object first

- The repo consistently builds typed runtime objects and only then renders reports. [VERIFIED: `src/cognition/working_memory.rs`; VERIFIED: `src/agent/orchestration.rs`]
- Phase 7 should keep that pattern: integrate evidence before report rendering.

### Pattern 2: Thin outer orchestration, thick inner seams

- Orchestration sequences work; assembly/scoring/gating own domain semantics. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md`]
- Phase 7 should keep follow-up integration as an assembler/orchestrator handoff, not a Rig-facing concern.

### Pattern 3: Deduplicated citation contracts

- `collect_unique_citations(...)` already deduplicates report citations by `record_id`. [VERIFIED: `src/agent/orchestration.rs`]
- A matching dedupe rule should govern integrated evidence to avoid report/runtime drift.

## Anti-Patterns To Avoid

- **Do not keep follow-up evidence only in `AgentSearchReport.retrieval_steps`.** That is the exact bug this phase exists to close. [VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`]
- **Do not add a second scoring or gate API that consumes follow-up evidence separately.** One decision surface is a locked decision. [VERIFIED: `.planning/phases/07-follow-up-evidence-integration/07-CONTEXT.md`]
- **Do not flatten away query provenance when merging evidence.** Phase 7 must improve cognition consistency without reducing explainability. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `src/search/mod.rs`]
- **Do not move cognition ownership into Rig or report rendering.** The thin-boundary rule from Phase 4 remains in force. [VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-CONTEXT.md`]

## Recommended Plan Shape

Phase 7 cleanly fits two plans:

1. **Plan 07-01:** extend working-memory request + assembler so integrated follow-up results become real `world_fragments` / branch evidence.
2. **Plan 07-02:** update orchestration/report/scoring regression coverage so `AgentSearchReport`, branch scoring, and metacognitive decisions align on the merged evidence set.

This split maps directly to the roadmap and to the audit’s two broken seams: assembly first, then decision/report alignment. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`]
