---
phase: 07
slug: follow-up-evidence-integration
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-16
---

# Phase 07 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| agent-search retrieval loop -> working-memory assembler | Primary and follow-up retrieval results cross from orchestration into the runtime cognition object. | retrieved records, citations, traces, score breakdowns |
| merged evidence set -> branch materialization | Supporting record IDs are resolved against the integrated fragment set used by working memory. | evidence fragments, record IDs, truth context |
| orchestrator trace data -> report / decision surface | Retrieval trace, working memory, citations, and selected branch must stay consistent after integration. | retrieval steps, top-level citations, branch support, selected decision |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-07-01 (07-01) | T | `src/cognition/assembly.rs` | mitigate | `WorkingMemoryRequest` now carries additive integrated results and `WorkingMemoryAssembler::assemble(...)` merges them through one typed path before fragment materialization, preventing a second hidden evidence channel. Evidence: [assembly.rs](/home/tongyuan/project/agent_memos/src/cognition/assembly.rs#L42), [assembly.rs](/home/tongyuan/project/agent_memos/src/cognition/assembly.rs#L113), [assembly.rs](/home/tongyuan/project/agent_memos/src/cognition/assembly.rs#L248). | closed |
| T-07-02 (07-01) | R | `tests/working_memory_assembly.rs` | mitigate | Assembly regressions prove follow-up-only evidence appears in `present.world_fragments` and can satisfy branch supporting-evidence references. Evidence: [working_memory_assembly.rs](/home/tongyuan/project/agent_memos/tests/working_memory_assembly.rs#L323), [working_memory_assembly.rs](/home/tongyuan/project/agent_memos/tests/working_memory_assembly.rs#L399). | closed |
| T-07-02 (07-02) | T | `src/agent/orchestration.rs` | mitigate | Orchestration now accumulates retrieved results across bounded queries and feeds them back into assembly through `with_integrated_results(...)`, eliminating divergence between report trace and decision input. Evidence: [orchestration.rs](/home/tongyuan/project/agent_memos/src/agent/orchestration.rs#L249), [orchestration.rs](/home/tongyuan/project/agent_memos/src/agent/orchestration.rs#L274). | closed |
| T-07-03 (07-02) | R | `tests/agent_search.rs` | mitigate | Orchestration regressions assert that follow-up-only evidence appears together in `retrieval_steps`, top-level citations, `working_memory`, and selected-branch support, locking report/decision consistency. Evidence: [agent_search.rs](/home/tongyuan/project/agent_memos/tests/agent_search.rs#L432), [agent_search.rs](/home/tongyuan/project/agent_memos/tests/agent_search.rs#L499). | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-16 | 4 | 4 | 0 | Codex |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-16
