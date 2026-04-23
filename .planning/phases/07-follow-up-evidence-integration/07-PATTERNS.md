# Phase 7: Follow-up Evidence Integration - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 6
**Analogs found:** 6 / 6

## Revision Notes

- Phase 7 is an integration-repair phase: reuse existing retrieval, assembly, scoring, and report contracts.
- The code already has bounded follow-up retrieval; the missing seam is how those results enter working-memory assembly.
- The safest pattern is additive request/assembler extension plus orchestrator wiring and regression tests.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/cognition/assembly.rs` | service | transform | `src/cognition/assembly.rs` | exact |
| `src/cognition/working_memory.rs` | model | transform | `src/cognition/working_memory.rs` | exact |
| `src/agent/orchestration.rs` | orchestration | event-driven | `src/agent/orchestration.rs` | exact |
| `tests/working_memory_assembly.rs` | test | integration | `tests/working_memory_assembly.rs` | exact |
| `tests/agent_search.rs` | test | integration | `tests/agent_search.rs` | exact |
| `src/cognition/metacog.rs` / `src/cognition/value.rs` | consumer | read-only | current scorer/gate consumers | supporting |

## Pattern Assignments

### `src/cognition/assembly.rs` (service, transform)

**Analog:** `src/cognition/assembly.rs:1-220`

**Pattern to copy**

- request struct grows additively with builder helpers
- assembler keeps one `assemble(...)` entrypoint
- internal search/materialization logic converts results into `EvidenceFragment` + `TruthRecord`

**Phase 7 application**

- Extend `WorkingMemoryRequest` with an additive evidence input field instead of creating a parallel assembly API.
- Keep the current “search if no override, integrate if supplied” shape inside `assemble(...)`.
- Reuse `materialize_branch(...)` and `world_fragments` generation so branch evidence and present-state evidence come from the same fragment list.

### `src/cognition/working_memory.rs` (model, transform)

**Analog:** `src/cognition/working_memory.rs:1-140`

**Pattern to copy**

- small typed structs
- provenance stays on `EvidenceFragment`
- `PresentFrame.world_fragments` is the runtime evidence carrier

**Phase 7 application**

- Preserve the top-level `WorkingMemory { present, branches }` contract.
- If new metadata is needed to distinguish integrated follow-up evidence, add it to `EvidenceFragment` additively and keep serde/report compatibility in mind.
- Do not add a second top-level follow-up evidence collection to `WorkingMemory`.

### `src/agent/orchestration.rs` (orchestration, event-driven)

**Analog:** `src/agent/orchestration.rs:145-290`

**Pattern to copy**

- bounded query list
- `retrieval_steps` as trace data
- orchestrator owns sequencing, not cognition semantics

**Phase 7 application**

- Keep the retrieval loop intact.
- Build integrated assembler input from the already retrieved per-query results before calling `assemble(...)`.
- Keep `collect_unique_citations(...)`, but make sure report citations and `working_memory.present.world_fragments` now reflect the same deduplicated evidence set.

### `tests/working_memory_assembly.rs` (test, integration)

**Analog:** `tests/working_memory_assembly.rs:1-220`

**Pattern to copy**

- temp DB + real ingest + real assembler
- assert on `world_fragments`, truth context, citations, and branch evidence

**Phase 7 application**

- Add one regression proving externally supplied follow-up results appear in `present.world_fragments`.
- Add one regression proving integrated fragments can also satisfy branch supporting-evidence references.

### `tests/agent_search.rs` (test, integration)

**Analog:** `tests/agent_search.rs:1-480`

**Pattern to copy**

- scripted retriever / assembler / scorer / gate ports for narrow orchestration tests
- typed report assertions over `retrieval_steps`, `citations`, and `working_memory`

**Phase 7 application**

- Update orchestration tests so follow-up query evidence is present in the assembled `working_memory`, not only in `retrieval_steps`.
- Add assertions that branch evidence / gate decisions align with integrated citations.

## Anti-Patterns To Avoid

- Extending only `AgentSearchReport` while leaving assembler inputs unchanged.
- Duplicating integrated evidence in a second working-memory field instead of `present.world_fragments`.
- Requiring scorer or gate callers to know which evidence came from follow-up queries through a separate API.
- Losing `matched_query` provenance during evidence dedupe/merge.
