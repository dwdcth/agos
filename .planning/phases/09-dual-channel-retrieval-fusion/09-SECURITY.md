---
phase: 09
slug: dual-channel-retrieval-fusion
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-16
---

# Phase 09 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| root config contract -> retrieval mode dispatch | Parsed config drives whether lexical-only, embedding-only, or hybrid retrieval behavior is active. | retrieval mode, embedding config, vector backend config |
| lexical + embedding candidate sets -> merged result set | Candidate overlap and dedupe now happen across two channels before final ranking. | record IDs, scores, query strategies, channel provenance |
| final result trace -> higher-layer consumers | Dual-channel trace fields are consumed by retrieval CLI and higher-level systems such as agent-search. | channel contribution, citations, result traces |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-09-01 (09-01) | R | `src/core/config.rs` / `tests/dual_channel_retrieval.rs` | mitigate | The real root `config.toml` is parsed into a shared retrieval config base, and three explicit runtime variants are generated/tested so mode selection cannot drift silently. Evidence: [config.rs](/home/tongyuan/project/agent_memos/src/core/config.rs), [dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L8). | closed |
| T-09-02 (09-02) | T | `src/search/mod.rs` / `src/search/rerank.rs` | mitigate | Lexical and embedding candidates are merged through one `SearchService` path and deduped by `record_id` before final result shaping, preserving authority identity and lexical-first explanation. Evidence: [search/mod.rs](/home/tongyuan/project/agent_memos/src/search/mod.rs), [dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L147). | closed |
| T-09-03 (09-03) | R | `src/search/rerank.rs` / `src/search/mod.rs` | mitigate | Final results now expose explicit `ChannelContribution` while preserving lexical citations/provenance and compatibility with existing retrieval consumers. Evidence: [search/rerank.rs](/home/tongyuan/project/agent_memos/src/search/rerank.rs), [retrieval_cli.rs](/home/tongyuan/project/agent_memos/tests/retrieval_cli.rs). | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-16 | 3 | 3 | 0 | Codex |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-16
