# Phase 10: Dual-Channel Diagnostics And Service Compatibility - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 7
**Analogs found:** 7 / 7

## Revision Notes

- Phase 10 should extend existing surfaces, not add new product lines.
- The repo already has the right seams: `StatusReport`, `DoctorReport`, `SearchService::with_variant(...)`, and `AgentSearchOrchestrator`.
- The main risk is creating a semantic-only bypass instead of flowing dual-channel mode through the shared ordinary retrieval path.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/interfaces/cli.rs` | interface | command/request routing | `src/interfaces/cli.rs` | exact |
| `src/core/status.rs` | diagnostics | inspect/render | `src/core/status.rs` | exact |
| `src/core/doctor.rs` | policy | gate/warn | `src/core/doctor.rs` | exact |
| `src/search/mod.rs` | service | retrieval dispatch | `src/search/mod.rs` | exact |
| `src/agent/orchestration.rs` | orchestration | retrieval reuse | `src/agent/orchestration.rs` | exact |
| `tests/status_cli.rs` | test | operator diagnostics | `tests/status_cli.rs` | exact |
| `tests/retrieval_cli.rs` / `tests/agent_search.rs` | test | compatibility | existing retrieval and agent-search integration tests | exact |

## Pattern Assignments

### `src/interfaces/cli.rs` (interface, command routing)

**Analog:** current `search`, `status`, `doctor`, and `inspect schema` surfaces

**Phase 10 application**

- Add mode controls additively to `search` and possibly `agent-search`.
- Keep lexical-only defaults unchanged.
- Reuse `SearchService::with_variant(...)` instead of branching into a second semantic command path.

### `src/core/status.rs` + `src/core/doctor.rs` (diagnostics/policy)

**Analog:** current capability-state reporting and command-path-specific warnings/failures

**Phase 10 application**

- Keep detailed capability states.
- Add a higher-level mode/channel summary in notes or rendering without removing the detailed fields.
- Preserve the Phase 6/8 operator semantics: truthful, explicit, local-first.

### `src/search/mod.rs` (shared ordinary retrieval seam)

**Analog:** current lexical-only default and `with_variant(...)` support

**Phase 10 application**

- Mode selection for CLI/library/agent flows should all resolve through this seam.
- Avoid any “semantic fast path” not available to ordinary retrieval.

### `src/agent/orchestration.rs` (compatibility seam)

**Analog:** current `RetrievalPort` / `SearchServicePort` usage

**Phase 10 application**

- If retrieval mode becomes explicit for agent-search, pass it through ordinary retrieval wiring.
- Preserve follow-up integration and report consistency from Phase 7/9.

## Anti-Patterns To Avoid

- creating `semantic-search` as a separate CLI surface
- adding mode flags that bypass `SearchService::with_variant(...)`
- making `status` less explicit in order to simplify output
- teaching `agent-search` about semantic retrieval in a way ordinary retrieval cannot express
