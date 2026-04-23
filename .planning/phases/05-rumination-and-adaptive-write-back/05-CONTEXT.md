# Phase 5: Rumination And Adaptive Write-back - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** Roadmap, requirements, prior phase outputs, and default-zero-friction discuss decisions derived from theory and current implementation boundaries

<domain>
## Phase Boundary

Phase 5 delivers the first controlled learning/write-back loop on top of ordinary retrieval, truth governance, and agent-search outputs. It is responsible for:
- introducing the dual-queue rumination model (`SPQ` and `LPQ`)
- defining bounded triggers, throttling, deduplication, and scheduling semantics
- implementing short-cycle write-back into self/risk state
- implementing long-cycle generation of skill templates, promotion candidates, and value-adjustment candidates
- preserving the governance rule that shared truth mutation remains proposal-driven and controlled

This phase is not responsible for:
- re-implementing retrieval, working memory, value scoring, or Rig orchestration
- semantic retrieval execution
- automatic direct mutation of shared T1 truth
- product UI or cloud/platform concerns

</domain>

<decisions>
## Implementation Decisions

### Queue Model And Triggering
- **D-01:** Phase 5 uses a dual-queue learning model: `SPQ` for short-cycle synchronous rumination and `LPQ` for long-cycle asynchronous rumination.
- **D-02:** `SPQ` is triggered by action failure, user correction, and metacognitive veto events.
- **D-03:** `LPQ` is triggered by session boundaries, evidence accumulation, idle windows, and abnormal-pattern accumulation.
- **D-04:** Queue triggering must support deduplication, minimum interval / cooldown, and bounded budget so rumination does not loop uncontrollably.
- **D-05:** `SPQ` always has higher priority than `LPQ`.

### Short-Cycle Write-back Boundaries
- **D-06:** Short-cycle write-back may update `self_state`, `risk_boundary`, and local/private T3-adjacent adaptation state.
- **D-07:** Short-cycle write-back must not directly mutate shared T2/T1 truth.
- **D-08:** Short-cycle write-back is primarily corrective and should optimize for immediate next-step safety and self-model correction.

### Long-Cycle Output Shapes
- **D-09:** Long-cycle rumination outputs must share a unified queue-item schema.
- **D-10:** The initial long-cycle output classes are:
  - `skill_template`
  - `promotion_candidate`
  - `value_adjustment_candidate`
- **D-11:** Long-cycle processing is allowed to synthesize candidates and proposals, but not to auto-apply them into shared truth.

### Write-back Safety And Approval
- **D-12:** Default write-back policy is candidate/proposal-first, not direct mutation.
- **D-13:** Shared-truth-facing changes remain proposal-driven and require explicit governance handling rather than automatic approval.
- **D-14:** The system should distinguish between:
  - local adaptive updates that are allowed automatically
  - governance candidates that must be reviewed or consumed by later services

### the agent's Discretion
- Exact queue item struct names and storage split, as long as `SPQ`/`LPQ` remain explicit and distinct.
- Exact cooldown / dedupe / budget field names and default values.
- Exact decomposition of `self_state` and `risk_boundary` write-back targets.
- Exact long-cycle candidate payload shape, as long as the three output classes above stay first-class.

</decisions>

<specifics>
## Specific Ideas

- Keep rumination outputs explicit and auditable instead of hiding them in log text or ad hoc background jobs.
- Preserve a clean distinction between:
  - trigger detection
  - queue scheduling
  - candidate generation
  - actual write-back application
- Phase 5 should feel like “bounded learning control plane”, not “background self-mutating magic”.
- Any write-back touching truth governance should remain visible as proposal/candidate objects that later review or service layers can inspect.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` — Phase 5 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` — `LRN-01`, `LRN-02`, `LRN-03`
- `.planning/PROJECT.md` — lexical-first baseline, local-first constraints, explainability rule
- `.planning/STATE.md` — current project state and prior-phase decisions

### Prior phase outputs
- `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md` — T3 -> T2 governed promotion
- `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md` — T2 -> T1 candidate-only handling and governance queues
- `.planning/phases/04-working-memory-and-agent-search/04-01-SUMMARY.md` — working-memory assembly
- `.planning/phases/04-working-memory-and-agent-search/04-02-SUMMARY.md` — value scoring and metacognitive gates
- `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md` — bounded agent-search orchestration and thin Rig adapter

### Domain theory
- `doc/0415-反刍机制.md` — SPQ / LPQ, trigger logic, write-back semantics
- `doc/0415-元认知层.md` — veto and warning signals as learning triggers
- `doc/0415-真值层.md` — promotion/candidate governance boundaries
- `doc/0415-自我模型.md` — self-model correction and risk-boundary implications
- `doc/0415-价值层.md` — value-layer adjustments and slow-variable implications

### Project research
- `.planning/research/ARCHITECTURE.md` — module boundary guidance
- `.planning/research/STACK.md` — stack and extension constraints
- `.planning/research/SUMMARY.md` — ordering rationale and systemic pitfalls

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/memory/governance.rs` — already provides proposal/candidate-first governance patterns
- `src/cognition/metacog.rs` — already emits warning/veto/escalate outcomes suitable as SPQ triggers
- `src/agent/orchestration.rs` and `DecisionReport` / `AgentSearchReport` — already provide bounded, cited outcome artifacts that rumination can consume

### Established Patterns
- additive side-table evolution over authority data
- typed model + repository + orchestration layering
- explicit governance candidates instead of automatic shared-truth mutation
- bounded and inspectable control flow over hidden heuristics

### Integration Points
- `SPQ` should consume metacognitive and action-outcome signals from Phase 4 reports
- `LPQ` should emit governance-compatible candidates that Phase 3 seams already know how to represent
- write-back services should consume existing `self_state`, truth, and value structures rather than invent parallel stores

</code_context>

<deferred>
## Deferred Ideas

- fully autonomous shared-truth mutation
- semantic retrieval driven rumination
- UI-driven review consoles
- cross-process distributed learning queues

</deferred>

---

*Phase: 05-rumination-and-adaptive-write-back*
*Context gathered: 2026-04-16*
