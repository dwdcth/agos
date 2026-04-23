# Phase 4: Working Memory And Agent Search - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning
**Source:** Roadmap, requirements, prior phase outputs, and user decisions from discussion

<domain>
## Phase Boundary

Phase 4 delivers the first executable cognitive control layer on top of ordinary retrieval and truth governance. It is responsible for:
- assembling a typed working-memory object from retrieved evidence and current task context
- representing epistemic, instrumental, and regulative candidate actions in one decision field
- scoring candidate actions with an explicit multi-dimensional value model
- integrating Rig as a thin orchestration layer over internal retrieval/assembly/gating/action interfaces
- adding metacognitive warning, veto, and escalate behavior before candidate actions are treated as valid outputs

This phase is not responsible for:
- re-implementing ordinary lexical retrieval or truth-layer storage
- semantic retrieval execution
- automatic learning/rumination loops
- broad product UI or multi-tenant/API platform concerns

</domain>

<decisions>
## Implementation Decisions

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

### the agent's Discretion
- Exact field names and file/module split for working-memory and action models, as long as the typed structure and decision boundaries above are preserved.
- Exact `ValueConfig` storage shape and default weight values.
- Exact internal interface names for the thin Rig adapter, as long as Rig stays orchestration-only.
- Exact trace/debug persistence format for working-memory snapshots.

</decisions>

<specifics>
## Specific Ideas

- Keep the working-memory assembly product explicit and debuggable rather than collapsing it into a loose JSON blob or plain list of retrieved records.
- Treat metacognitive outcomes as first-class structured outputs, not logging side effects only.
- Preserve a clear distinction between:
  - retrieved evidence
  - assembled present-state control field
  - candidate actions
  - metacognitive interventions
- Phase 4 should feel like “ordinary retrieval + control-layer assembly + guarded action selection”, not “chat wrapper over search”.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` — Phase 4 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` — `COG-01`, `COG-02`, `COG-03`, `COG-04`, `AGT-02`, `AGT-03`, `AGT-04`
- `.planning/PROJECT.md` — product constraints, lexical-first baseline, explainability rule, Rig boundary
- `.planning/STATE.md` — current project state and prior-phase decisions

### Prior phase outputs
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md` — lexical sidecar, score breakdown, lexical readiness
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md` — structured retrieval result contracts, citations, filters, CLI/library access
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-04-SUMMARY.md` — truthful lexical status wording
- `.planning/phases/03-truth-layer-governance/03-01-SUMMARY.md` — truth metadata foundation and repository seam
- `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md` — T3 -> T2 promotion governance
- `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md` — T2 -> T1 candidate-only handling and governance queues

### Domain theory
- `doc/0415-工作记忆.md` — working-memory ontology and control-field semantics
- `doc/0415-注意力状态.md` — attention and front-stage selection logic
- `doc/0415-价值层.md` — value dimensions and comparison semantics
- `doc/0415-元认知层.md` — warning / veto / regulation / escalation semantics
- `doc/0415-00记忆认知架构.md` — recall vs cognition boundary

### Project research
- `.planning/research/ARCHITECTURE.md` — module boundary guidance
- `.planning/research/STACK.md` — stack constraints and Rig positioning
- `.planning/research/SUMMARY.md` — phase ordering rationale and major pitfalls

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/search/*` — already provides lexical-first retrieval, filters, citations, and explainable score breakdowns
- `src/memory/repository.rs` / `src/memory/truth.rs` / `src/memory/governance.rs` — already provide truth-layer and governance seams that working memory can consume
- `src/core/status.rs` and existing typed config/readiness contracts — useful precedent for explicit typed service outputs and diagnostic enums

### Established Patterns
- Authority-store + additive side-table evolution
- Thin CLI wrappers over internal services
- Typed model + repository + orchestration service layering
- Explicit phase boundaries that keep semantic retrieval and Rig from swallowing internal domain logic

### Integration Points
- `Retriever` should adapt the existing Phase 2 search services rather than bypass them
- `Assembler` should consume typed retrieval and truth-layer outputs, not raw SQL rows
- `Metacognition` should layer on top of candidate actions and working memory, not directly own retrieval
- Rig adapter should call into internal services, not the other way around

</code_context>

<deferred>
## Deferred Ideas

- multiplicative or more advanced value aggregation
- semantic retrieval execution
- autonomous rumination/write-back loops
- broader UI or remote orchestration surfaces

</deferred>

---

*Phase: 04-working-memory-and-agent-search*
*Context gathered: 2026-04-15*
