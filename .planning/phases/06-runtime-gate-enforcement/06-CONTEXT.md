# Phase 6: Runtime Gate Enforcement - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** Roadmap, milestone audit gaps, prior phase outputs, and zero-friction discuss defaults derived from the current CLI/runtime behavior

<domain>
## Phase Boundary

Phase 6 closes the cross-phase readiness gap by making the existing runtime health contract actually govern operational command entrypoints. It is responsible for:
- propagating `doctor/init` readiness semantics into `ingest`, `search`, and `agent-search`
- blocking impossible or reserved retrieval mode / embedding backend combinations before downstream command logic proceeds
- keeping operational failure messages aligned with the existing status/doctor explanation model
- adding regression coverage so later phases cannot silently reopen the bypass path

This phase is not responsible for:
- implementing semantic retrieval or making `embedding_only` / `hybrid` actually runnable
- changing the three retrieval modes or collapsing them into boolean feature flags
- redesigning ordinary retrieval, working memory, or Rig orchestration behavior
- adding new product surfaces beyond the existing CLI/runtime seams

</domain>

<decisions>
## Implementation Decisions

### Gate Scope
- **D-01:** Phase 6 must gate all current operational CLI entrypoints that execute core memory logic: `ingest`, `search`, and `agent-search`.
- **D-02:** The gate applies before downstream command execution and must prevent invalid runtime combinations from reaching ingestion, retrieval, or cognition services.
- **D-03:** Informational commands such as `status`, `doctor`, and schema inspection remain non-blocking diagnostic surfaces rather than being converted into hard-fail paths.

### Gate Source Of Truth
- **D-04:** Readiness policy remains centralized in the existing status/doctor contract; operational commands must reuse that contract rather than introducing a second rule set.
- **D-05:** Invalid or impossible mode/backend combinations must fail consistently everywhere instead of being blocked in `init/doctor` but tolerated elsewhere.
- **D-06:** Reserved semantic modes stay explicit. Phase 6 must not silently downgrade `embedding_only` or `hybrid` requests into `lexical_only`.

### Operator Semantics
- **D-07:** Gate failures should be rendered in the same explanatory style as `doctor`: structured readiness result plus concrete failure reasons, not opaque runtime exceptions.
- **D-08:** Operational blocking should distinguish between hard failures and informational warnings the same way the existing diagnostics do; warnings may be shown, but only hard failures block execution.
- **D-09:** Lexical-first remains the only executable retrieval baseline in v1, so operational commands should continue when lexical readiness is truly satisfied and stop when it is not.

### Test And Regression Boundary
- **D-10:** Regression coverage must prove that the same blocked configurations are rejected across `ingest`, `search`, and `agent-search`, not only in `init/doctor`.
- **D-11:** Regression coverage must also preserve the informational contract for `status`/`doctor`, so Phase 6 does not accidentally turn diagnostics into side-effecting or brittle command paths.

### the agent's Discretion
- Exact helper names, enum expansion, and whether the shared gate wrapper lives at CLI dispatch or a thinner operational boundary.
- Exact text formatting for reused gate output, as long as it stays aligned with the existing doctor/status contract.
- Exact test decomposition between CLI integration tests and narrower unit coverage.

</decisions>

<specifics>
## Specific Ideas

- Treat this phase as propagation of an already-agreed contract, not invention of a new readiness model.
- Prefer one reusable operational gate path over command-specific ad hoc conditionals.
- Keep the user-visible outcome explicit: if a mode is reserved or impossible, the command should say so immediately and stop before doing real work.
- Phase 6 should feel like “runtime commands finally obey the declared operating envelope”, not “another diagnostics refactor”.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and audit target
- `.planning/ROADMAP.md` — Phase 6 goal, success criteria, and gap-closure plan slots
- `.planning/REQUIREMENTS.md` — `FND-01`, `FND-03`, and `AGT-01` are remapped here as pending
- `.planning/v1.0-MILESTONE-AUDIT.md` — exact integration failures and broken operational flow this phase must close
- `.planning/PROJECT.md` — lexical-first baseline, local-first constraint, and explainability contract
- `.planning/STATE.md` — current milestone state and prior-phase decisions

### Prior phase outputs
- `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md` — startup diagnostics, status, and initial doctor/init rule shaping
- `.planning/phases/01-foundation-kernel/01-04-SUMMARY.md` — truthful status degradation and post-init warning behavior
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md` — ordinary retrieval CLI/library behavior that now needs runtime gating
- `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md` — `agent-search` entrypoint and thin Rig boundary that must now obey the same readiness contract

### Runtime code seams
- `src/core/app.rs` — current runtime readiness semantics by retrieval mode
- `src/core/status.rs` — readiness snapshot, capability states, and operator-facing notes
- `src/core/doctor.rs` — command-path-sensitive failure vs warning policy
- `src/interfaces/cli.rs` — current operational entrypoints and the main gate propagation target

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/core/app.rs` — already materializes typed `RuntimeReadiness` from config and preserves lexical-only / embedding-only / hybrid intent.
- `src/core/status.rs` — already computes schema, dependency, and index readiness with explainable notes.
- `src/core/doctor.rs` — already distinguishes failures from warnings and has path-sensitive logic for `Init` vs `Doctor`.

### Established Patterns
- Diagnostics are typed first and rendered second; CLI should not invent new readiness semantics.
- Command-path-sensitive gating already exists conceptually, but Phase 6 must extend it beyond bootstrap and explicit diagnostics.
- Lexical-first behavior is explicit and mode-aware; reserved semantic paths stay visible rather than being hidden behind fallback behavior.

### Integration Points
- `src/interfaces/cli.rs` is the immediate enforcement seam for `ingest`, `search`, and `agent-search`.
- `src/ingest/mod.rs`, `src/search/mod.rs`, and `src/agent/orchestration.rs` should remain focused on core behavior rather than each re-implementing gate policy.
- Existing CLI integration tests around status/retrieval/agent-search provide the natural regression surface for this phase.

</code_context>

<deferred>
## Deferred Ideas

- Making `embedding_only` executable with a real embedding backend
- Hybrid recall / rerank runtime behavior beyond lexical-first v1
- Non-CLI operational surfaces such as MCP or HTTP readiness gating

</deferred>

---

*Phase: 06-runtime-gate-enforcement*
*Context gathered: 2026-04-16*
