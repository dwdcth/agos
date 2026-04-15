---
phase: 06
slug: runtime-gate-enforcement
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-16
---

# Phase 06 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| config + local runtime state -> operational command gate | Retrieval mode, embedding backend, schema state, and lexical index readiness determine whether a CLI command may execute. | local config values, SQLite capability state, readiness notes |
| CLI entrypoint -> DB/service layers | `ingest`, `search`, and `agent-search` cross from user-invoked command dispatch into DB and core service execution. | command args, DB open, retrieval requests, agent-search requests |
| diagnostic command surface -> operator decision | `status`, `doctor`, and `inspect schema` expose local runtime facts to operators without mutating state. | diagnostic text, readiness/failure reasons |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-06-01 (06-01) | T | `src/interfaces/cli.rs` | mitigate | Shared `operational_gate(...)` now runs before `Database::open(...)` in `ingest_command`, `search_command`, and `agent_search_command`, eliminating per-command bypass seams. Evidence: [cli.rs](/home/tongyuan/project/agent_memos/src/interfaces/cli.rs#L275), [cli.rs](/home/tongyuan/project/agent_memos/src/interfaces/cli.rs#L313), [cli.rs](/home/tongyuan/project/agent_memos/src/interfaces/cli.rs#L380), [cli.rs](/home/tongyuan/project/agent_memos/src/interfaces/cli.rs#L406). | closed |
| T-06-02 (06-01) | R | `src/core/doctor.rs` | mitigate | Blocked operational paths are rendered through `DoctorReport::render_text()` with explicit failure lists, so denials stay explainable and reproducible instead of degrading into lower-level runtime errors. Evidence: [doctor.rs](/home/tongyuan/project/agent_memos/src/core/doctor.rs#L22), [doctor.rs](/home/tongyuan/project/agent_memos/src/core/doctor.rs#L86), [runtime_gate_cli.rs](/home/tongyuan/project/agent_memos/tests/runtime_gate_cli.rs#L134). | closed |
| T-06-03 (06-01) | E | `src/core/doctor.rs` / `src/core/status.rs` | mitigate | Reserved semantic modes remain explicit failures on operational paths, and no lexical fallback is introduced for `embedding_only` or `hybrid`. Evidence: [doctor.rs](/home/tongyuan/project/agent_memos/src/core/doctor.rs#L41), [runtime_gate_cli.rs](/home/tongyuan/project/agent_memos/tests/runtime_gate_cli.rs#L135). | closed |
| T-06-01 (06-02) | T | `tests/runtime_gate_cli.rs` | mitigate | Regression coverage freezes lexical not-ready cases (`missing init`, `bad db file`, `missing lexical sidecars`) so future refactors cannot silently re-open the bypass below the CLI gate. Evidence: [runtime_gate_cli.rs](/home/tongyuan/project/agent_memos/tests/runtime_gate_cli.rs#L224). | closed |
| T-06-02 (06-02) | D | `src/interfaces/cli.rs` | mitigate | Operational commands stop before DB/service execution when local runtime state is not executable, reducing confusing downstream failures and denial-of-service-by-bad-state behavior. Evidence: [cli.rs](/home/tongyuan/project/agent_memos/src/interfaces/cli.rs#L406), [doctor.rs](/home/tongyuan/project/agent_memos/src/core/doctor.rs#L121), [runtime_gate_cli.rs](/home/tongyuan/project/agent_memos/tests/runtime_gate_cli.rs#L225). | closed |
| T-06-03 (06-02) | R | `tests/status_cli.rs` / `src/core/doctor.rs` | mitigate | Informational diagnostics and operational gate output are locked to the same structured contract where applicable, so operators can compare `status`/`doctor` with blocked commands deterministically. Evidence: [status_cli.rs](/home/tongyuan/project/agent_memos/tests/status_cli.rs#L332), [doctor.rs](/home/tongyuan/project/agent_memos/src/core/doctor.rs#L86). | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-16 | 6 | 6 | 0 | Codex |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-16
