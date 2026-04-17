# Phase 10: Dual-Channel Diagnostics And Service Compatibility - Research

**Researched:** 2026-04-16  
**Domain:** dual-channel operator diagnostics, mode control surfaces, ordinary retrieval / agent-search compatibility  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
- MCP / HTTP exposure of retrieval-mode controls
- user-tunable fusion policies beyond config-derived mode selection
- provider-backed LLM + embedding end-to-end smoke for production environments
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OPS-01 | `status` / `doctor` can report embedding backend and vector-index readiness truthfully alongside lexical readiness. [VERIFIED: `.planning/REQUIREMENTS.md`] | Phase 8 already split lexical/embedding/vector readiness at the status layer, but the operator story still lacks a milestone-level “dual-channel ready / disabled / gated” presentation. [VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/doctor.rs`; VERIFIED: `.planning/phases/08-embedding-backend-and-index-foundation/08-VERIFICATION.md`] |
| OPS-02 | CLI or library search surfaces can enable or disable the embedding second channel through config or request-level behavior without breaking lexical-only operation. [VERIFIED: `.planning/REQUIREMENTS.md`] | Phase 9 added mode-driven retrieval behavior, but the CLI/search surface still defaults to lexical-only and does not expose clean mode controls to operators. [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `src/search/mod.rs`; VERIFIED: `.planning/phases/09-dual-channel-retrieval-fusion/09-VERIFICATION.md`] |
| OPS-03 | Agent-search continues to reuse ordinary retrieval services when the embedding second channel is enabled, instead of introducing a semantic-only bypass path. [VERIFIED: `.planning/REQUIREMENTS.md`] | `AgentSearchOrchestrator` still uses ordinary `SearchRequest`/`SearchResponse` seams; Phase 10 should preserve that contract while validating compatibility under mode selection and enriched traces. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `tests/agent_search.rs`] |
</phase_requirements>

## Summary

Phase 10 is not new retrieval logic. The dual-channel behavior already exists in `SearchService`; what is missing is the operator/control layer that makes that behavior usable and trustworthy. Right now the CLI `search` surface does not expose retrieval-mode control, and `agent-search` still constructs its inner `SearchRequest`s without any explicit mode handoff. The safest approach is to add additive mode-control seams that still flow through the same ordinary retrieval contract. [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/agent/orchestration.rs`]

The diagnostics side is also close but not finished. `StatusReport` now knows about `embedding_dependency_state` and `embedding_index_readiness`, yet it does not summarize how those states interact with the configured retrieval mode in a way that answers a simple operator question: “If I run search now, which channels are active, which are disabled, and which are gated?” Phase 10 should make that explicit without collapsing the detailed capability-state fields. [VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/doctor.rs`]

**Primary recommendation:** finish Phase 10 in two slices: (1) operator surfaces and request-level mode control for search/diagnostics, and (2) agent-search compatibility checks plus minimal mode-plumbing so higher layers continue to consume ordinary retrieval instead of drifting into a semantic bypass. [ASSUMED]

## Current Code Findings

### 1. The CLI still lacks explicit retrieval-mode control

- `SearchCommand` currently exposes filters and trace flags, but no mode override or embedding-toggle flag. [VERIFIED: `src/interfaces/cli.rs`]
- `SearchService::with_variant(...)` exists, but CLI `search` still calls `SearchService::new(...)`, which means it only uses the lexical-only default unless config loading is widened elsewhere. [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `src/search/mod.rs`]

**Implication:** Phase 10 should add an operator-facing way to select or override retrieval mode without forking the search service.

### 2. Status/doctor know capability pieces, but the operator story is still too low-level

- `StatusReport` already distinguishes `lexical_dependency_state`, `embedding_dependency_state`, `index_readiness`, and `embedding_index_readiness`. [VERIFIED: `src/core/status.rs`]
- `DoctorReport` already blocks impossible/gated combinations and can explain why a semantic-primary mode is unavailable. [VERIFIED: `src/core/doctor.rs`]

**Implication:** Phase 10 should layer a clearer “active channels / gated channels” operator story on top of these existing facts rather than replacing them.

### 3. Agent-search still reuses ordinary retrieval, which is the invariant to preserve

- `AgentSearchOrchestrator` uses `SearchRequest` and the `RetrievalPort` abstraction; production wiring still goes through `SearchServicePort`. [VERIFIED: `src/agent/orchestration.rs`]
- Because the ordinary retrieval seam is still shared, Phase 10 can validate compatibility by controlling how that seam is configured rather than modifying agent-search into a separate semantic client. [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `tests/agent_search.rs`]

**Implication:** Mode selection should flow through ordinary retrieval inputs, not around them.

## Recommended Implementation Direction

### Search surface controls

Recommended exact direction:

- Add additive retrieval-mode control to CLI/library search requests.
- Keep lexical-only as the implicit default.
- Allow explicit operator selection of:
  - lexical-only
  - embedding-only
  - hybrid

This can be a CLI flag and/or a richer library-side request field, as long as it resolves into the same ordinary retrieval service path. [ASSUMED]

### Diagnostics

Recommended exact direction:

- Keep existing capability-state lines.
- Add a higher-level summary or note that makes channel activity explicit, for example:
  - lexical active / embedding disabled
  - lexical active / embedding ready but not selected
  - lexical + embedding active
  - embedding configured but gated

This gives operators an actionable answer without hiding the underlying state fields. [ASSUMED]

### Agent-search compatibility

Recommended exact direction:

- Add a minimal mode-plumbing seam so `agent-search` can continue to use ordinary retrieval with explicit retrieval-mode selection when appropriate.
- Preserve default lexical-first behavior if no mode is provided.
- Add regression coverage proving that `AgentSearchReport` still flows from the same ordinary retrieval contract under dual-channel mode selection.

## Testing Direction

### Best regression split

1. **`tests/status_cli.rs`**
   - dual-channel readiness/operator narrative
   - mode-aware status/doctor semantics

2. **`tests/retrieval_cli.rs`**
   - CLI `search` mode control
   - lexical-only stability under explicit/implicit mode selection

3. **`tests/agent_search.rs`**
   - ordinary retrieval reuse under explicit mode selection
   - no semantic-only bypass path

This follows the current repo pattern of operator surface tests + integration tests over shared seams. [VERIFIED: `tests/status_cli.rs`; VERIFIED: `tests/retrieval_cli.rs`; VERIFIED: `tests/agent_search.rs`]

## Anti-Patterns To Avoid

- **Do not add a second semantic-specific CLI command** when `search` already exists and the requirement is about compatibility.
- **Do not hide lexical-only as a “legacy” mode** after hybrid support exists. It is still a supported first-class mode.
- **Do not let agent-search pick a semantic path that ordinary retrieval cannot express.**
- **Do not compress detailed readiness fields into a single opaque summary string** if it removes diagnostic precision.

## Recommended Plan Shape

Phase 10 fits two plans:

1. **Plan 10-01:** dual-channel diagnostics + search surface controls
2. **Plan 10-02:** agent-search compatibility over the same dual-channel service seam

That split maps directly to the roadmap and keeps operator surfaces separate from higher-layer compatibility checks. [VERIFIED: `.planning/ROADMAP.md`]
