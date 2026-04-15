# Roadmap: Agent Memos

## Milestones

- ✅ **v1.0 milestone** — Phases 1-7 (shipped 2026-04-16)
- 🚧 **v1.1 Embedding Second-Channel Retrieval** — Phases 8-10 (planned)

## Phases

<details>
<summary>✅ v1.0 milestone (Phases 1-7) — SHIPPED 2026-04-16</summary>

- [x] Phase 1: Foundation Kernel (4/4 plans) — completed 2026-04-15
- [x] Phase 2: Ingest And Lightweight Retrieval (4/4 plans) — completed 2026-04-15
- [x] Phase 3: Truth Layer Governance (3/3 plans) — completed 2026-04-15
- [x] Phase 4: Working Memory And Agent Search (3/3 plans) — completed 2026-04-16
- [x] Phase 5: Rumination And Adaptive Write-back (3/3 plans) — completed 2026-04-16
- [x] Phase 6: Runtime Gate Enforcement (2/2 plans) — completed 2026-04-16
- [x] Phase 7: Follow-up Evidence Integration (2/2 plans) — completed 2026-04-16

</details>

### 🚧 v1.1 Embedding Second-Channel Retrieval

- [ ] **Phase 8: Embedding Backend And Index Foundation** - 增加可选 embedding backend、embedding 持久化与 vector sidecar/index 基础能力
- [ ] **Phase 9: Dual-Channel Retrieval Fusion** - 实现 lexical-first + embedding second-channel 的 recall / fusion / rerank
- [ ] **Phase 10: Dual-Channel Diagnostics And Service Compatibility** - 对 dual-channel search 的 trace、状态诊断与 agent-search 兼容性做收尾

## Phase Details

### Phase 8: Embedding Backend And Index Foundation
**Goal**: 为 optional embedding second-channel 建立最小可用底座，包括 backend config/readiness、embedding 持久化，以及 additive vector sidecar/index。
**Depends on**: Phase 7
**Requirements**: [EMB-01, EMB-02, EMB-03]
**Success Criteria** (what must be TRUE):
  1. Developer can configure an optional embedding backend/model while lexical-only baseline remains intact.
  2. System can persist embeddings additively for ingested records/chunks without replacing the authority store.
  3. Optional vector sidecar/index exists behind the same local-first SQLite deployment model.
**Plans**: 3 plans

Plans:
- [ ] 08-01: Add embedding backend config, readiness, and storage contracts
- [ ] 08-02: Persist embeddings for ingested records or chunks
- [ ] 08-03: Add optional vector sidecar/index bootstrap and maintenance path

### Phase 9: Dual-Channel Retrieval Fusion
**Goal**: 在 ordinary retrieval 中引入 lexical-first + embedding second-channel 的 recall / fusion / rerank，使 dual-channel 结果仍然保持 explainable。
**Depends on**: Phase 8
**Requirements**: [DCR-01, DCR-02, DCR-03, DCR-04]
**Success Criteria** (what must be TRUE):
  1. Ordinary retrieval can execute lexical and embedding channels in one request flow.
  2. Lexical remains the primary explanation channel even when embedding contributes recall or rerank signal.
  3. Final result contract can explain channel contribution, dedupe behavior, and combined ranking.
**Plans**: 3 plans

Plans:
- [ ] 09-01: Add embedding second-channel recall on top of the existing search contract
- [ ] 09-02: Implement lexical-first fusion, dedupe, and rerank logic
- [ ] 09-03: Extend result traces to explain dual-channel contribution

### Phase 10: Dual-Channel Diagnostics And Service Compatibility
**Goal**: 把 dual-channel retrieval 的状态诊断、CLI/library surface 和 agent-search 兼容性补齐，确保新通道不会破坏现有 ordinary retrieval / agent-search 边界。
**Depends on**: Phase 9
**Requirements**: [OPS-01, OPS-02, OPS-03]
**Success Criteria** (what must be TRUE):
  1. `status` / `doctor` 能同时报告 lexical 与 embedding / vector readiness，并保持 truthful diagnostics。
  2. Search surfaces can open or close the embedding second channel without破坏 lexical-only 使用方式。
  3. Agent-search continues to consume ordinary retrieval services when dual-channel retrieval is enabled.
**Plans**: 2 plans

Plans:
- [ ] 10-01: Extend dual-channel diagnostics and operator surfaces
- [ ] 10-02: Ensure ordinary retrieval and agent-search compatibility over the same dual-channel service

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation Kernel | v1.0 | 4/4 | Complete | 2026-04-15 |
| 2. Ingest And Lightweight Retrieval | v1.0 | 4/4 | Complete | 2026-04-15 |
| 3. Truth Layer Governance | v1.0 | 3/3 | Complete | 2026-04-15 |
| 4. Working Memory And Agent Search | v1.0 | 3/3 | Complete | 2026-04-16 |
| 5. Rumination And Adaptive Write-back | v1.0 | 3/3 | Complete | 2026-04-16 |
| 6. Runtime Gate Enforcement | v1.0 | 2/2 | Complete | 2026-04-16 |
| 7. Follow-up Evidence Integration | v1.0 | 2/2 | Complete | 2026-04-16 |
| 8. Embedding Backend And Index Foundation | v1.1 | 0/3 | Not started | - |
| 9. Dual-Channel Retrieval Fusion | v1.1 | 0/3 | Not started | - |
| 10. Dual-Channel Diagnostics And Service Compatibility | v1.1 | 0/2 | Not started | - |
