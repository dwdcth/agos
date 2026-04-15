# Feature Research

**Domain:** Agent memory and cognition system
**Researched:** 2026-04-15
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Hybrid retrieval | Memory systems are expected to handle exact terms and semantic similarity | MEDIUM | Must combine lexical and vector search instead of choosing one. |
| Source citations and traceability | Users need to trust retrieved memory in downstream decisions | MEDIUM | Results should carry source, timestamp, and retrieval rationale. |
| Structured ingestion | A memory system that cannot normalize docs, notes, and conversations is incomplete | MEDIUM | Ingest needs typed memory units, not just raw blobs. |
| Scoped filtering | Users expect project/topic/type filtering when memory grows | LOW | Wing/room style scoping from `mempal` is a strong reference. |
| Time-aware validity | Memories and facts age; stale recall is worse than no recall | HIGH | Especially important once T1/T2/T3 and truth promotion exist. |
| Search result reranking | Raw vector or BM25 ranking is rarely enough for decision support | MEDIUM | Fusion plus domain-aware rerank is required. |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Dual search modes: ordinary retrieval + agentic search | Cleanly separates recall from cognition and matches the user's stated scope | HIGH | This is the core product shape, not an optional add-on. |
| T1/T2/T3 truth layering | Makes private hypotheses, shared facts, and ontology structurally distinct | HIGH | Directly derived from the 0415 theory set. |
| Working memory assembly | Search outputs become actionable cognitive state, not just snippets | HIGH | Needed for candidate action comparison and agent orchestration. |
| Metacognitive gating | Prevents agents from acting on under-supported or risky recall | HIGH | A major differentiator from typical RAG systems. |
| Rumination / write-back queues | Turns search history and outcomes into evolving memory structures | HIGH | Enables slow-learning behavior instead of static recall. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| “Just chat with the database” | Fast demo path | Collapses retrieval, reasoning, and truth management into one opaque loop | Keep ordinary retrieval and agent search as separate surfaces |
| Fully automatic promotion into shared truth | Feels intelligent and autonomous | Pollutes T2/T1 with hallucinations or single-session bias | Require evidence, verification, and metacognitive checks |
| UI-first dashboards before the core engine | Easier to demo visually | Consumes time before ranking, schema, and search semantics are stable | CLI/library/API-first delivery |
| Overly generic memory blobs | Simplifies ingestion | Makes later truth-layering, working-memory assembly, and write-back much harder | Typed memories: evidence, event, decision, skill, self-state, world-state |

## Feature Dependencies

```text
Hybrid retrieval
    ├──requires──> memory schema
    ├──requires──> lexical index
    └──requires──> vector index

Agentic search
    ├──requires──> hybrid retrieval
    ├──requires──> citation/trace output
    ├──requires──> working memory assembly
    └──requires──> rig integration

Truth layering
    ├──requires──> typed memory schema
    ├──requires──> promotion rules
    └──enhances──> agentic search

Rumination
    ├──requires──> action outcome capture
    ├──requires──> truth layering
    └──enhances──> self/skill/world updates
```

### Dependency Notes

- **Hybrid retrieval requires memory schema:** lexical and vector ranking need a stable row model, filters, and metadata.
- **Agentic search requires citation output:** otherwise the agent cannot explain or verify why a memory was used.
- **Truth layering requires typed schema:** T1/T2/T3 is not a ranking tweak; it is a data-model decision.
- **Rumination requires outcome capture:** without outcomes and corrections, there is nothing meaningful to write back.

## MVP Definition

### Launch With (v1)

- [ ] Local-first memory store with typed memory records and SQLite schema
- [ ] Hybrid search combining `libsimple` lexical recall and `sqlite-vec` semantic recall
- [ ] Search results with source, scope, and timestamp traces
- [ ] Agentic search workflow using Rig to plan multi-step retrieval and assemble working memory
- [ ] Minimal truth-layer separation for private hypotheses vs shared evidence-backed facts

### Add After Validation (v1.x)

- [ ] Candidate action generation and value scoring — once retrieval quality is reliable enough to drive decisions
- [ ] Metacognitive veto and search-policy adjustment — once false-positive and false-confidence patterns are observed
- [ ] Skill-memory extraction from successful traces — once repeated action patterns are available

### Future Consideration (v2+)

- [ ] Full T1/T2/T3 promotion pipelines with approval workflows — high leverage but needs schema maturity
- [ ] Long-cycle rumination queues and background maintenance — defer until the online path is solid
- [ ] MCP / multi-interface exposure — add after the core services are stable
- [ ] Cross-project tunnels or multi-wing routing — useful, but not core to first validation

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Hybrid retrieval | HIGH | MEDIUM | P1 |
| Citation / trace output | HIGH | MEDIUM | P1 |
| Typed memory schema | HIGH | MEDIUM | P1 |
| Agentic search orchestration | HIGH | HIGH | P1 |
| Truth-layer separation | HIGH | HIGH | P1 |
| Working memory assembly | HIGH | HIGH | P1 |
| Rumination queues | MEDIUM | HIGH | P2 |
| Skill extraction | MEDIUM | HIGH | P2 |
| MCP / remote surface | MEDIUM | MEDIUM | P2 |
| Visual UI | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Generic RAG tools | `mempal` | Our Approach |
|---------|-------------------|----------|--------------|
| Hybrid retrieval | Usually yes, but shallow | Yes, with BM25 + vector + RRF | Yes, but extended with cognitive metadata and truth layers |
| Citation / trace | Often partial | Strong | Must be first-class |
| Agentic search | Usually prompt glue | Some agent-facing tooling, not 0415 cognition | Explicit second subsystem with working-memory assembly |
| Truth layering | Rare | Not the primary model | Core differentiator |
| Rumination / write-back | Rare | Diary / ingest centric | Cross-layer write-back from outcomes and corrections |

## Sources

- `doc/0415-00记忆认知架构.md`
- `doc/0415-认知索引.md`
- `doc/0415-真值层.md`
- `doc/0415-工作记忆.md`
- `doc/0415-元认知层.md`
- `doc/0415-反刍机制.md`
- `reference/mempal/README_zh.md`

---
*Feature research for: agent memory and cognition system*
*Researched: 2026-04-15*
