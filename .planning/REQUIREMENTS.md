# Requirements: Agent Memos

**Defined:** 2026-04-15
**Core Value:** 当 agent 需要回忆历史决策、证据、模式或当前认知状态时，系统必须能快速给出带出处、带时间性、带状态约束的正确记忆，而不是只返回“看起来相似”的文本片段。

## v1 Requirements

### Foundation

- [ ] **FND-01**: Developer can initialize a local-first Rust application with a SQLite database, schema migrations, and deterministic startup checks for retrieval dependencies.
- [ ] **FND-02**: System can persist typed memory records with source, timestamp, scope, record type, truth-layer metadata, and provenance fields.
- [ ] **FND-03**: Developer can inspect system health and index status from a CLI surface without requiring an LLM.

### Ingest

- [ ] **ING-01**: Developer can ingest notes, documents, and conversation-like text into normalized memory units suitable for indexing.
- [ ] **ING-02**: System can chunk ingested content while preserving source linkage and chunk provenance.
- [ ] **ING-03**: System can store or derive embeddings for indexed memory units without breaking lexical retrieval.

### Retrieval

- [ ] **RET-01**: Agent or developer can run ordinary lexical search over Chinese and PinYin content using `libsimple`-backed SQLite FTS.
- [ ] **RET-02**: Agent or developer can run semantic search over the same corpus using `sqlite-vec`.
- [ ] **RET-03**: System can fuse lexical and semantic candidates into a single ranked result set with stable scoring behavior.
- [ ] **RET-04**: Each retrieval result includes source, scope, timestamp or validity metadata, and enough trace data to explain why it was returned.
- [ ] **RET-05**: Agent or developer can filter retrieval by scope, record type, truth layer, and time validity.

### Truth Layers

- [ ] **TRU-01**: System distinguishes T1, T2, and T3 records in storage and service APIs instead of treating all memory as one undifferentiated blob.
- [ ] **TRU-02**: T3 records carry explicit provenance, confidence, and revocability markers so private working hypotheses remain auditable.
- [ ] **TRU-03**: System can promote a T3 structure toward T2 only through an explicit gate that records evidence review and metacognitive approval state.
- [ ] **TRU-04**: System can create T2-to-T1 ontology candidates without automatically rewriting the shared ontology layer.

### Cognitive Core

- [ ] **COG-01**: System can assemble a working-memory object containing `world_fragments`, `self_state`, `active_goal`, `active_risks`, `candidate_actions`, and `metacog_flags`.
- [ ] **COG-02**: Working memory can contain epistemic, operational, and regulatory candidate actions in the same decision field.
- [ ] **COG-03**: System can score candidate actions with a multi-dimensional value representation before projecting them into a comparable decision score.
- [ ] **COG-04**: Metacognitive logic can inject warnings or veto flags when retrieval or candidate actions are too uncertain, risky, or under-supported.

### Agent Search

- [ ] **AGT-01**: Developer can use ordinary retrieval without invoking a language model or agent runtime.
- [ ] **AGT-02**: Developer can invoke a Rig-based agent-search workflow that performs multi-step retrieval and evidence gathering over the internal search services.
- [ ] **AGT-03**: Agent-search output includes citations and a structured working-memory or decision-support payload instead of a plain freeform answer only.
- [ ] **AGT-04**: Agent-search orchestration does not bypass ordinary retrieval services or write directly into shared truth without explicit gates.

### Learning

- [ ] **LRN-01**: System can route write-back work into short-cycle and long-cycle queues instead of treating all learning as one batch process.
- [ ] **LRN-02**: Short-cycle write-back can update self-model or risk-boundary state from action outcomes and user correction without directly mutating shared truth.
- [ ] **LRN-03**: Long-cycle write-back can produce skill templates, shared-fact promotion candidates, or value-adjustment candidates from accumulated evidence.

## v2 Requirements

### Interfaces

- **INT-01**: Developer can access the system through MCP tools in addition to CLI or library APIs.
- **INT-02**: Developer can expose a stable HTTP API for search and agent-search operations.

### Extended Memory

- **EXT-01**: System can support cross-project tunnel discovery or multi-wing memory routing.
- **EXT-02**: System can support richer visualization or inspection tooling for truth layers and working-memory state.

### Advanced Governance

- **GOV-01**: System can support human review workflows for T2-to-T1 ontology proposals.
- **GOV-02**: System can support richer policy packs or role-specific value overlays.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multi-tenant cloud platform | Local-first single-machine architecture is the v1 target |
| UI-first product shell | Core retrieval and cognition behavior must stabilize first |
| Fully automatic ontology mutation | T2-to-T1 should remain proposal-driven, not self-authorizing |
| Provider-specific hard coupling | `rig` is used to keep the agent layer replaceable |
| “Chat over memories” as the only product surface | The project explicitly separates ordinary retrieval from agentic cognition |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 1 | Pending |
| FND-02 | Phase 1 | Pending |
| FND-03 | Phase 1 | Pending |
| ING-01 | Phase 2 | Pending |
| ING-02 | Phase 2 | Pending |
| ING-03 | Phase 2 | Pending |
| RET-01 | Phase 2 | Pending |
| RET-02 | Phase 2 | Pending |
| RET-03 | Phase 2 | Pending |
| RET-04 | Phase 2 | Pending |
| RET-05 | Phase 2 | Pending |
| TRU-01 | Phase 3 | Pending |
| TRU-02 | Phase 3 | Pending |
| TRU-03 | Phase 3 | Pending |
| TRU-04 | Phase 3 | Pending |
| COG-01 | Phase 4 | Pending |
| COG-02 | Phase 4 | Pending |
| COG-03 | Phase 4 | Pending |
| COG-04 | Phase 4 | Pending |
| AGT-01 | Phase 2 | Pending |
| AGT-02 | Phase 4 | Pending |
| AGT-03 | Phase 4 | Pending |
| AGT-04 | Phase 4 | Pending |
| LRN-01 | Phase 5 | Pending |
| LRN-02 | Phase 5 | Pending |
| LRN-03 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-15*
*Last updated: 2026-04-15 after initial definition*
