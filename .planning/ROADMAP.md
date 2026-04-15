# Roadmap: Agent Memos

## Overview

Agent Memos 的首个里程碑按五个阶段推进：先建立 Rust + SQLite 的本地认知底座，再交付 ingest 与 lexical-first 普通检索链路，然后把 T1/T2/T3 真值分层固化进数据模型，之后接入 working memory 与 Rig agent search，最后再补上元认知写回与双队列反刍。这样可以先把 deterministic retrieval 做对，再把 cognition 和 learning 叠上去，避免项目过早退化成“高级聊天检索”。

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation Kernel** - 建立 Rust 应用骨架、SQLite 底座与基础状态检查
- [ ] **Phase 2: Ingest And Lightweight Retrieval** - 交付普通检索主线，打通 ingest、中文词法检索、轻量关键词权重与可解释返回
- [ ] **Phase 3: Truth Layer Governance** - 把 T1/T2/T3 分层和受控晋升规则落到数据模型与服务边界
- [ ] **Phase 4: Working Memory And Agent Search** - 接入 Rig 智能体搜索、工作记忆装配、价值评分与元认知监督
- [ ] **Phase 5: Rumination And Adaptive Write-back** - 实现短周期/长周期反刍队列与受控写回

## Phase Details

### Phase 1: Foundation Kernel
**Goal**: 建立本地优先 Rust 代码骨架、SQLite schema/migration 入口、typed memory base model 与基础状态检查能力。
**Depends on**: Nothing (first phase)
**Requirements**: [FND-01, FND-02, FND-03]
**Success Criteria** (what must be TRUE):
  1. Developer can start the Rust application and initialize the local SQLite store with deterministic setup steps.
  2. Developer can inspect schema version, dependency loading state, and index readiness from a CLI status command.
  3. Typed memory records with source, timestamp, scope, and truth metadata exist as first-class storage structures.
**Plans**: 3 plans

Plans:
- [ ] 01-01: Bootstrap the Rust project structure and shared `core` module
- [ ] 01-02: Implement SQLite schema, migrations, and typed memory base entities
- [ ] 01-03: Add startup checks, status reporting, and developer inspection commands

### Phase 2: Ingest And Lightweight Retrieval
**Goal**: 打通普通检索主线，使系统可以在不依赖 LLM 和模型文件的前提下 ingest 资料并执行中文词法检索 + 轻量关键词加权排序。
**Depends on**: Phase 1
**Requirements**: [ING-01, ING-02, ING-03, RET-01, RET-02, RET-03, RET-04, RET-05, AGT-01]
**Success Criteria** (what must be TRUE):
  1. Developer can ingest notes, documents, or conversation text into normalized and chunked memory units with source linkage intact.
  2. Agent or developer can run ordinary lexical search over the corpus and see Rust-side lightweight keyword weighting affect final ranking.
  3. Retrieval results include source, scope, and validity metadata that explain why each memory was returned.
  4. Ordinary retrieval is fully usable from CLI or library APIs without invoking Rig or any LLM.
**Plans**: 3 plans

Plans:
- [ ] 02-01: Implement source normalization, chunking, and ingest persistence
- [ ] 02-02: Implement `libsimple` lexical recall and Rust lightweight scoring rules
- [ ] 02-03: Implement explainable rerank, filtering, citations, and ordinary retrieval APIs

### Phase 3: Truth Layer Governance
**Goal**: 将 T1/T2/T3 真值分层、私有假设边界和共享晋升规则固化到系统模型中。
**Depends on**: Phase 2
**Requirements**: [TRU-01, TRU-02, TRU-03, TRU-04]
**Success Criteria** (what must be TRUE):
  1. System stores T1, T2, and T3 memory structures as distinct truth-layer states instead of one undifferentiated memory class.
  2. T3 records preserve provenance, confidence, and revocability so private hypotheses remain auditable.
  3. T3-to-T2 promotion is blocked unless evidence review and metacognitive approval data are present.
  4. T2-to-T1 changes are represented as proposals or candidates, not automatic ontology rewrites.
**Plans**: 3 plans

Plans:
- [ ] 03-01: Add truth-layer metadata, repositories, and query semantics
- [ ] 03-02: Implement T3 provenance/revocability rules and promotion gate models
- [ ] 03-03: Add T2-to-T1 candidate handling and governance-oriented service APIs

### Phase 4: Working Memory And Agent Search
**Goal**: 在 ordinary retrieval 之上接入 Rig 智能体搜索，并把 working memory、value、metacognition 变成可执行服务。
**Depends on**: Phase 3
**Requirements**: [COG-01, COG-02, COG-03, COG-04, AGT-02, AGT-03, AGT-04]
**Success Criteria** (what must be TRUE):
  1. System can assemble a working-memory object containing world fragments, self state, active goal, risks, candidate actions, and metacognitive flags.
  2. Candidate actions from epistemic, operational, and regulatory modes can be compared inside the same decision field.
  3. Developer can invoke Rig-based agent search that performs multi-step retrieval while preserving citations and internal service boundaries.
  4. Metacognitive checks can flag or veto unsafe or under-supported candidate actions before they are treated as valid outputs.
**Plans**: 3 plans

Plans:
- [ ] 04-01: Implement attention-to-working-memory assembly services
- [ ] 04-02: Implement value scoring and metacognitive gating over candidate actions
- [ ] 04-03: Integrate Rig-based agent-search orchestration on top of ordinary retrieval

### Phase 5: Rumination And Adaptive Write-back
**Goal**: 让系统具备短周期/长周期反刍与受控写回能力，使搜索结果和行动结果能逐步沉淀为长期结构。
**Depends on**: Phase 4
**Requirements**: [LRN-01, LRN-02, LRN-03]
**Success Criteria** (what must be TRUE):
  1. System routes learning work into short-cycle and long-cycle queues with distinct triggers and write targets.
  2. Short-cycle write-back can update self-model or risk-boundary state from outcomes and user correction without mutating shared truth directly.
  3. Long-cycle processing can emit skill templates, promotion candidates, or value-adjustment candidates from accumulated evidence.
**Plans**: 3 plans

Plans:
- [ ] 05-01: Implement SPQ and LPQ scheduling with bounded triggers
- [ ] 05-02: Implement short-cycle write-back into self/risk state
- [ ] 05-03: Implement long-cycle write-back for skill extraction and promotion candidates

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation Kernel | 0/3 | Not started | - |
| 2. Ingest And Lightweight Retrieval | 0/3 | Not started | - |
| 3. Truth Layer Governance | 0/3 | Not started | - |
| 4. Working Memory And Agent Search | 0/3 | Not started | - |
| 5. Rumination And Adaptive Write-back | 0/3 | Not started | - |
