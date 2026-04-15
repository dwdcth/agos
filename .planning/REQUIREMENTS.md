# Requirements: Agent Memos

**Defined:** 2026-04-16
**Core Value:** 当 agent 需要回忆历史决策、证据、模式或当前认知状态时，系统必须能快速给出带出处、带时间性、带状态约束的正确记忆，而不是只返回“看起来相似”的文本片段。

## v1.1 Requirements

### Embedding Foundation

- [ ] **EMB-01**: Developer can configure an optional embedding backend and model without weakening the existing lexical-only baseline.
- [ ] **EMB-02**: System can generate and persist embeddings for ingested records or chunks in additive storage structures.
- [ ] **EMB-03**: System can maintain an optional vector sidecar or index for embedding-backed retrieval without redefining the authority store.

### Dual-Channel Retrieval

- [ ] **DCR-01**: Ordinary retrieval can run a lexical-first search path plus an embedding second channel within one request flow.
- [ ] **DCR-02**: Lexical retrieval remains the primary explanation source even when embedding recall contributes candidates or rerank signal.
- [ ] **DCR-03**: Retrieval fusion can dedupe and rank results from lexical and embedding channels into one stable result contract.
- [ ] **DCR-04**: Search results expose enough trace data to tell whether lexical recall, embedding recall, or both contributed to the final ranking.

### Operational Surfaces

- [ ] **OPS-01**: `status` / `doctor` can report embedding backend and vector-index readiness truthfully alongside lexical readiness.
- [ ] **OPS-02**: CLI or library search surfaces can enable or disable the embedding second channel through config or request-level behavior without breaking lexical-only operation.
- [ ] **OPS-03**: Agent-search continues to reuse ordinary retrieval services when the embedding second channel is enabled, instead of introducing a semantic-only bypass path.

## v2 Requirements

### Interfaces

- **INT-01**: Developer can access the system through MCP tools in addition to CLI or library APIs.
- **INT-02**: Developer can expose a stable HTTP API for search and agent-search operations.

### Extended Retrieval

- **EXT-01**: System can support richer embedding-model lifecycle management, background re-embedding, and index rebuild tooling.
- **EXT-02**: System can expose human-tunable fusion policies, rerank overlays, or role-specific retrieval profiles.
- **EXT-03**: System can support cross-project tunnel discovery or multi-wing memory routing.
- **EXT-04**: System can support richer visualization or inspection tooling for truth layers and working-memory state.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Replacing lexical-first with embedding-first retrieval | Violates the project’s explainability and baseline-stability constraints |
| New MCP / HTTP interface surface in this milestone | Interface expansion is useful, but not required to validate second-channel retrieval |
| New human-review governance workflow | Governance UX is a separate milestone concern from retrieval-channel expansion |
| Provider-specific deep coupling to one embedding runtime | v1.1 should keep the embedding path optional and replaceable |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| EMB-01 | Phase 8 | Pending |
| EMB-02 | Phase 8 | Pending |
| EMB-03 | Phase 8 | Pending |
| DCR-01 | Phase 9 | Pending |
| DCR-02 | Phase 9 | Pending |
| DCR-03 | Phase 9 | Pending |
| DCR-04 | Phase 9 | Pending |
| OPS-01 | Phase 10 | Pending |
| OPS-02 | Phase 10 | Pending |
| OPS-03 | Phase 10 | Pending |

**Coverage:**
- v1.1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-16*
*Last updated: 2026-04-16 after v1.1 milestone start*
