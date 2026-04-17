# Phase 10: Dual-Channel Diagnostics And Service Compatibility - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** v1.1 roadmap scope, Phase 8/9 outputs, and zero-friction discuss defaults derived from the current retrieval and agent-search seams

<domain>
## Phase Boundary

Phase 10 is the stabilization and compatibility layer for dual-channel retrieval. It is responsible for:
- extending `status` / `doctor` / operator surfaces so lexical and embedding/vector readiness are reported together truthfully
- making search surfaces configurable enough to open or close the embedding second channel without breaking lexical-only operation
- ensuring agent-search continues to reuse ordinary retrieval services cleanly after dual-channel retrieval lands

This phase is not responsible for:
- inventing a new retrieval mode beyond lexical-only / embedding-only / hybrid
- redesigning the dual-channel fusion/rerank logic built in Phase 9
- creating new MCP / HTTP interface surfaces
- changing the core cognition/working-memory model

</domain>

<decisions>
## Implementation Decisions

### Diagnostics Contract
- **D-01:** `status` and `doctor` must report lexical readiness and embedding/vector readiness as separate, explicit capability states.
- **D-02:** Operator diagnostics must remain truthful even when the dual-channel substrate is partially configured or intentionally disabled.
- **D-03:** Phase 10 diagnostics should explain *which* channel is unavailable or gated, not just say “search not ready”.

### Surface Compatibility
- **D-04:** Search surfaces should be able to run with the embedding second channel enabled or disabled through config and/or request-level behavior, while lexical-only remains stable.
- **D-05:** Lexical-only behavior must remain a first-class, supported mode after dual-channel retrieval lands; Phase 10 must not treat it as a degraded fallback.
- **D-06:** Any new CLI or library toggles for retrieval mode must align with the generated config matrix and existing root `config.toml` contract from Phase 9.

### Agent Compatibility
- **D-07:** Agent-search must continue to call ordinary retrieval services rather than gaining a semantic-only bypass path.
- **D-08:** If dual-channel retrieval changes ordinary result traces or mode selection, those changes must remain compatible with `AgentSearchReport`, follow-up evidence integration, and metacognitive decision flow.

### the agent's Discretion
- Exact naming of any new request-level toggles or helper structs for retrieval mode control.
- Exact shape of diagnostic text or CLI rendering additions.
- Exact regression split between status/search CLI tests and agent-search integration tests.

</decisions>

<specifics>
## Specific Ideas

- Treat Phase 10 as “make dual-channel retrieval livable for operators and higher layers”, not as “add another retrieval feature”.
- Prefer extending existing operator surfaces and request structs over adding one-off compatibility wrappers.
- Keep the user-visible story simple: developers should be able to tell what retrieval mode ran, what substrate is ready, and whether agent-search is still reusing ordinary retrieval.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope
- `.planning/PROJECT.md` — current v1.1 milestone goal and active requirements
- `.planning/ROADMAP.md` — Phase 10 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` — `OPS-01`, `OPS-02`, `OPS-03`
- `.planning/STATE.md` — current milestone state

### Prior phase outputs
- `.planning/phases/08-embedding-backend-and-index-foundation/08-VERIFICATION.md` — embedding foundation baseline and diagnostic constraints
- `.planning/phases/09-dual-channel-retrieval-fusion/09-VERIFICATION.md` — dual-channel retrieval behavior and explicit channel-trace outcomes
- `.planning/phases/06-runtime-gate-enforcement/06-VERIFICATION.md` — runtime diagnostics and operational gate baseline
- `.planning/phases/07-follow-up-evidence-integration/07-VERIFICATION.md` — agent-search / working-memory consistency baseline

### Runtime/code seams
- `src/core/config.rs` — retrieval mode and config-derived variant contract
- `src/core/status.rs` — lexical/embedding/vector readiness reporting
- `src/core/doctor.rs` — operator-facing failure/warning policy
- `src/interfaces/cli.rs` — current search / status / doctor / inspect schema surfaces
- `src/search/mod.rs` — shared ordinary retrieval service boundary
- `src/agent/orchestration.rs` — agent-search continues to consume ordinary retrieval
- `tests/status_cli.rs` — operator-surface regression harness
- `tests/retrieval_cli.rs` — retrieval CLI compatibility harness
- `tests/agent_search.rs` — agent-search compatibility harness

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `StatusReport` already separates lexical dependency, embedding dependency, and embedding index readiness, which makes it the natural foundation for Phase 10 operator work.
- `SearchService::with_variant(...)` and the config-derived mode matrix from Phase 9 already provide a retrieval-mode seam that operator surfaces can now expose more cleanly.
- `AgentSearchOrchestrator` still delegates through the shared retrieval seam, which is exactly the compatibility invariant Phase 10 should preserve.

### Established Patterns
- typed capability states before rendered diagnostics
- one ordinary retrieval service consumed by higher layers
- lexical-first remains explicit, not implied
- additive compatibility work over existing contracts rather than parallel bypass paths

### Integration Points
- status/doctor/search CLI flags and config parsing must stay aligned
- retrieval CLI output should remain stable and explainable under dual-channel mode selection
- agent-search tests should verify ordinary retrieval reuse after Phase 10 adjustments

</code_context>

<deferred>
## Deferred Ideas

- MCP / HTTP exposure of retrieval-mode controls
- user-tunable fusion policies beyond config-derived mode selection
- provider-backed LLM + embedding end-to-end smoke for production environments

</deferred>

---

*Phase: 10-dual-channel-diagnostics-and-service-compatibility*
*Context gathered: 2026-04-16*
