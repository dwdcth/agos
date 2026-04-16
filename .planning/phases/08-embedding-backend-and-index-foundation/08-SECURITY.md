---
phase: 08
slug: embedding-backend-and-index-foundation
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-16
---

# Phase 08 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| config file -> runtime readiness | Embedding backend/model choices now affect readiness notes and operator-visible state. | config values, backend/model metadata |
| ingest completion -> embedding sidecar persistence | Chunked authority records may optionally produce embedding side-table rows. | chunk text, record IDs, embedding vectors |
| vector-sidecar bootstrap state -> operator diagnostics | Missing or incomplete embedding substrate must be visible without being confused with lexical readiness. | schema/index state, readiness reports |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-08-01 (08-01) | R | `src/core/status.rs` / `src/core/doctor.rs` | mitigate | Typed embedding backend/readiness states plus `status_cli` coverage prevent diagnostics from silently lying about semantic capability. Evidence: [status_cli.rs](/home/tongyuan/project/agent_memos/tests/status_cli.rs#L429), [status_cli.rs](/home/tongyuan/project/agent_memos/tests/status_cli.rs#L488), [status.rs](/home/tongyuan/project/agent_memos/src/core/status.rs#L97). | closed |
| T-08-02 (08-02) | T | `src/ingest/mod.rs` / `src/memory/repository.rs` | mitigate | Embeddings are persisted only as additive side-table rows keyed to `memory_records(id)`, leaving authority ownership and lexical-only ingest intact. Evidence: [0006_embedding_foundation.sql](/home/tongyuan/project/agent_memos/migrations/0006_embedding_foundation.sql), [repository.rs](/home/tongyuan/project/agent_memos/src/memory/repository.rs#L402), [ingest_pipeline.rs](/home/tongyuan/project/agent_memos/tests/ingest_pipeline.rs#L236). | closed |
| T-08-03 (08-03) | R | `src/core/status.rs` / `src/interfaces/cli.rs` | mitigate | `embedding_index_readiness` is reported separately from lexical `index_readiness`, and operator surfaces expose the difference directly. Evidence: [status.rs](/home/tongyuan/project/agent_memos/src/core/status.rs#L102), [status.rs](/home/tongyuan/project/agent_memos/src/core/status.rs#L183), [cli.rs](/home/tongyuan/project/agent_memos/src/interfaces/cli.rs#L437), [status_cli.rs](/home/tongyuan/project/agent_memos/tests/status_cli.rs#L550). | closed |

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
