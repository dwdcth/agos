# Phase 3: Truth Layer Governance - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning
**Source:** Roadmap, requirements, truth-layer theory docs, and Phase 1/2 outputs

<domain>
## Phase Boundary

Phase 3 turns the truth-layer model into real storage and service boundaries. It is responsible for:
- representing T1, T2, and T3 as distinct truth-layer states in storage and service APIs
- preserving T3 provenance, confidence, and revocability as explicit metadata
- defining the governed T3 -> T2 promotion gate and the required evidence/review state
- defining T2 -> T1 candidate handling as proposal/candidate flow rather than direct ontology rewrite
- extending query semantics and repository/service contracts so truth-layer distinctions are actionable, not just labels

This phase is not responsible for:
- ordinary lexical retrieval itself, which is already delivered in Phase 2
- semantic retrieval execution
- Rig-based agent search orchestration
- working-memory assembly, metacognitive action veto, or value scoring
- rumination queue scheduling or automatic write-back execution

</domain>

<decisions>
## Implementation Decisions

### Locked decisions
- Phase 3 must distinguish T1, T2, and T3 in storage and service APIs instead of treating truth-layer as one undifferentiated tag.
- T3 is a private working-hypothesis layer and must preserve explicit provenance, confidence, and revocability fields.
- T3 must never directly overwrite shared truth; promotion toward T2 requires a governed gate with evidence-review and approval state.
- T2 -> T1 must be represented as proposal/candidate handling, not automatic ontology mutation.
- Query semantics must allow callers to filter and interpret records differently by truth layer.
- Existing Phase 2 retrieval and citation behavior must continue to work on top of the new truth metadata, not be broken by this refactor.
- The lexical-first retrieval baseline remains unchanged; Phase 3 should reuse it rather than re-implement it.
- Phase 3 should prepare governance seams that later metacognition/rumination can call, but should not prematurely implement Phase 4/5 behavior.

### the agent's Discretion
- Exact module split for truth-layer metadata, repositories, and promotion-gate services.
- Whether truth governance lives primarily under `memory/`, a new `truth/` module, or a small hybrid split, as long as the boundaries stay clear.
- Exact shape of proposal/candidate records for T2 -> T1, as long as they remain explicit non-automatic proposals.
- Whether promotion reviews are modeled as structured JSON payloads or dedicated typed records, as long as evidence and approval state remain queryable and auditable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` - Phase 3 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` - `TRU-01`, `TRU-02`, `TRU-03`, `TRU-04`
- `.planning/PROJECT.md` - lexical-first baseline, local-first constraints, explainability rule
- `.planning/STATE.md` - current project state and carry-over concerns

### Prior phase outputs
- `.planning/phases/01-foundation-kernel/01-02-SUMMARY.md` - current authority schema and typed memory repository base
- `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md` - status/doctor/readiness surfaces
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-01-SUMMARY.md` - authority-store ingest metadata and validity windows
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md` - lexical sidecar, score breakdown, lexical readiness
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md` - structured retrieval contracts, citations, filters
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-04-SUMMARY.md` - truthful lexical-only status wording

### Domain theory
- `doc/0415-真值层.md` - T1/T2/T3 definitions, promotion logic, revocability, and candidate handling
- `doc/0415-00记忆认知架构.md` - recall vs cognition boundary and memory semantics
- `doc/0415-元认知层.md` - later approval/governance context that Phase 3 should prepare for without implementing fully

### Project research
- `.planning/research/ARCHITECTURE.md` - target module boundaries
- `.planning/research/STACK.md` - stack and extension constraints
- `.planning/research/SUMMARY.md` - phase ordering rationale and pitfalls

</canonical_refs>

<specifics>
## Specific Ideas

- Phase 3 likely needs additive schema work over `memory_records` plus one or more governance tables, rather than a full storage rewrite.
- Distinguish at least:
  - current truth layer on records
  - T3-specific provenance/confidence/revocability
  - promotion review/evidence state for T3 -> T2
  - candidate/proposal tracking for T2 -> T1
- Retrieval filters introduced in Phase 2 should remain valid, but their semantics should now become governance-aware rather than just label-aware.
- Query/service APIs should make it easy for later phases to say:
  - “give me only shared truth”
  - “show me revocable/private hypotheses”
  - “list promotion candidates awaiting review”

</specifics>

<deferred>
## Deferred Ideas

- Rig-based agent search orchestration
- working-memory assembly and metacognitive checks
- automated rumination or write-back loops
- semantic retrieval execution

</deferred>

---

*Phase: 03-truth-layer-governance*
*Context gathered: 2026-04-15 via roadmap, requirements, theory docs, and Phase 1/2 outputs*
