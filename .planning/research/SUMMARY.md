# Project Research Summary

**Project:** Agent Memos
**Domain:** Rust local-first memory cognition system
**Researched:** 2026-04-15
**Confidence:** MEDIUM

## Executive Summary

这个项目不是普通的 RAG 工具，而是一个把“检索”和“认知”明确拆开的记忆认知底座。`doc/` 中的 0415 理论文档决定了它至少要同时满足两件事：一是提供可靠、可解释、可追溯的 ordinary retrieval；二是在此之上提供基于智能体的搜索，将 recall 组织成工作记忆、候选行动与后续写回。

从实现视角看，最稳的做法是采用 `reference/mempal` 那种 Rust 单二进制、本地 SQLite、模块化拆分的工程骨架，但不要直接复制它的领域模型。当前项目的差异化不在于“再做一个 memory search”，而在于 T1/T2/T3、工作记忆、元认知 veto、双队列反刍这些认知结构必须成为一等公民。

## Key Findings

### Recommended Stack

推荐以 Rust 1.85+、SQLite、`rusqlite`、`sqlite-vec`、`libsimple` 和 `rig-core` 作为骨架。`libsimple` 明确覆盖中文 / 拼音 FTS5 tokenizer，`sqlite-vec` 负责本地语义检索，Rig 则更适合作为 agent/tool/provider orchestration 层，而不是核心存储层。核心内存模型仍应自己维护。

**Core technologies:**
- Rust 1.85+：满足 `libsimple 0.9.0` 的版本约束，并适合强边界建模
- SQLite + `rusqlite`：本地优先、便于组合 FTS5 和向量扩展
- `sqlite-vec`：语义检索底座，但官方仍标注 pre-v1，需留兼容缓冲
- `libsimple ~0.9`：中文和拼音全文检索，是普通检索体验的关键
- `rig-core`：agent 搜索、工具编排、模型抽象

### Expected Features

这个领域的 table stakes 不是“能搜”，而是“能搜对、能解释、能约束”。因此，v1 至少要有混合检索、引用和时间正确性、类型化 memory schema、scope filter、agentic search orchestration、working-memory assembly。

**Must have (table stakes):**
- 混合检索
- 引用与 trace
- 类型化 memory schema
- scoped filtering
- 时间有效性

**Should have (competitive):**
- dual search modes
- T1/T2/T3 truth layering
- working-memory assembly
- metacognitive gating

**Defer (v2+):**
- 完整长期反刍后台
- 跨项目 tunnel
- UI-first 产品层

### Architecture Approach

推荐采用四层：接口层、应用服务层、认知核心层、存储基础层。普通检索与 agent 搜索必须是兄弟模块，而不是一个函数的两个 flag。认知核心内部再拆分 truth / world / self / skill / attention / working_memory / value / metacog / rumination，保持 0415 术语与实现一一映射。

**Major components:**
1. `core` — config、db、types、migration
2. `memory` — typed records、truth layers、promotion rules
3. `search` — lexical + semantic + fusion + citation
4. `cognition` — attention、working memory、metacognition、rumination
5. `agent` — Rig orchestration over internal services

### Critical Pitfalls

1. **检索与认知塌缩** — 必须把 deterministic search 和 LLM orchestration 分开
2. **T1/T2/T3 混表** — truth-layer 元数据和 promotion 规则要早建
3. **只做向量搜索** — 中文 exact match 与结构化 cue 会失真
4. **agent 直接写共享真值** — 写回必须经过证据和 gate
5. **把 working memory 当 top-k 结果** — 必须单独装配控制场

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Storage And Hybrid Retrieval Foundation
**Rationale:** 所有上层认知能力都依赖稳定 schema 和混合检索
**Delivers:** 核心 SQLite schema、typed memory records、`libsimple` + `sqlite-vec` 检索链路
**Addresses:** ordinary retrieval, citations, scope, validity
**Avoids:** vector-only search and search/cognition collapse

### Phase 2: Truth-Layered Memory Model
**Rationale:** 没有 truth layers，后续 agent search 和 write-back 都会失真
**Delivers:** T1/T2/T3 metadata、promotion guards、shared/private memory boundaries
**Uses:** Phase 1 schema and search services
**Implements:** truth model and memory repositories

### Phase 3: Agent Search And Working Memory
**Rationale:** 在 deterministic search 可靠后，再接 Rig agent orchestration
**Delivers:** ordinary search API, agent search workflow, working-memory assembly
**Uses:** Rig integration and existing search services
**Implements:** the second core product line

### Phase 4: Metacognition, Rumination, And Write-back
**Rationale:** 先让系统能查、能想，再让它学和纠偏
**Delivers:** metacognitive flags, veto hooks, SPQ/LPQ rumination, bounded write-back

### Phase Ordering Rationale

- 先做存储和检索，再做 truth semantics，再接 agent，最后再做学习回路
- 这样能最早建立可测试面，并避免 LLM 提前绑架底层数据模型
- 也能把高风险坑限制在更小的阶段里

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1:** `libsimple` + `sqlite-vec` + `rusqlite` 初始化与兼容细节
- **Phase 3:** Rig tool / agent / retrieval adapter 的最小可行接法
- **Phase 4:** write-back 协议、promotion gate 和反刍队列策略

Phases with standard patterns (skip research-phase):
- **Phase 2:** 只要 schema 清晰，truth-layer typing 和 repository pattern 相对标准

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | 外部依赖信息足够，但 `sqlite-vec` 和 Rig 仍在快速迭代 |
| Features | HIGH | 用户目标和本地理论文档非常明确 |
| Architecture | MEDIUM | `mempal` 提供了好骨架，但当前项目的 cognition core 仍需自己抽象 |
| Pitfalls | HIGH | 主要风险直接来自本项目理论边界和同类系统常见塌缩模式 |

**Overall confidence:** MEDIUM

### Gaps to Address

- `sqlite-vec` 的实际 pinned version 需要在 Phase 1 结合 `rusqlite` 代码验证
- Rig 是否直接接内置 SQLite adapter，还是先走自定义工具层，需要在 Phase 3 定稿
- T1/T2/T3 的最小 schema 粒度需要在 Phase 2 明确到字段级

## Sources

### Primary (HIGH confidence)
- `doc/0415-00记忆认知架构.md`
- `doc/0415-真值层.md`
- `doc/0415-工作记忆.md`
- `doc/0415-元认知层.md`
- `doc/0415-反刍机制.md`
- `reference/mempal/README_zh.md`
- `reference/mempal/src/core/db.rs`
- `reference/mempal/src/search/mod.rs`

### Secondary (MEDIUM confidence)
- https://docs.rs/crate/libsimple/latest
- https://github.com/asg017/sqlite-vec
- https://github.com/0xPlaygrounds/rig
- https://docs.rs/crate/rig-core/latest

---
*Research completed: 2026-04-15*
*Ready for roadmap: yes*
