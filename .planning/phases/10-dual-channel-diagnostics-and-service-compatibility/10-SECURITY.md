---
phase: 10
slug: dual-channel-diagnostics-and-service-compatibility
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-17
---

# Phase 10 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| operator mode override -> runtime readiness gate | CLI mode selection changes which retrieval channels must be ready before search or agent-search may run. | retrieval mode, embedding backend state, vector readiness |
| runtime-configured retrieval -> ordinary search / agent-search | Higher-level consumers must inherit the same retrieval behavior and explanation surface without silently dropping config. | channel contribution, citations, retrieval traces |
| operator diagnostics -> operational decisions | Misleading active/gated channel reporting could cause operators to run an unsafe or nonfunctional mode. | active channel labels, gated channel labels, doctor failures |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-10-01 (10-01) | R | `src/interfaces/cli.rs` / `src/core/status.rs` / `src/core/doctor.rs` | mitigate | Ordinary `search` mode selection is explicit and dual-channel diagnostics now expose `active_channels`, `gated_channels`, and mode-aware doctor failures so operators can tell what is actually runnable. Evidence: `src/interfaces/cli.rs`, `src/core/status.rs`, `src/core/doctor.rs`, `tests/status_cli.rs`, `tests/retrieval_cli.rs`. | closed |
| T-10-02 (10-02) | T | `src/agent/orchestration.rs` / `src/interfaces/cli.rs` | mitigate | Agent-search now consumes the full runtime config through the same `SearchService` seam, preserving embedding/vector config and preventing a semantic-only bypass path. Evidence: `src/agent/orchestration.rs`, `src/interfaces/cli.rs`, `tests/agent_search.rs`. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-17 | 2 | 2 | 0 | Codex |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-17
