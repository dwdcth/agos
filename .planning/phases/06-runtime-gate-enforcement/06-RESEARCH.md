# Phase 6: Runtime Gate Enforcement - Research

**Researched:** 2026-04-16  
**Domain:** CLI runtime gate propagation, readiness enforcement, operator-facing diagnostics  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Gate Scope
- **D-01:** Phase 6 must gate all current operational CLI entrypoints that execute core memory logic: `ingest`, `search`, and `agent-search`.
- **D-02:** The gate applies before downstream command execution and must prevent invalid runtime combinations from reaching ingestion, retrieval, or cognition services.
- **D-03:** Informational commands such as `status`, `doctor`, and schema inspection remain non-blocking diagnostic surfaces rather than being converted into hard-fail paths.

### Gate Source Of Truth
- **D-04:** Readiness policy remains centralized in the existing status/doctor contract; operational commands must reuse that contract rather than introducing a second rule set.
- **D-05:** Invalid or impossible mode/backend combinations must fail consistently everywhere instead of being blocked in `init/doctor` but tolerated elsewhere.
- **D-06:** Reserved semantic modes stay explicit. Phase 6 must not silently downgrade `embedding_only` or `hybrid` requests into `lexical_only`.

### Operator Semantics
- **D-07:** Gate failures should be rendered in the same explanatory style as `doctor`: structured readiness result plus concrete failure reasons, not opaque runtime exceptions.
- **D-08:** Operational blocking should distinguish between hard failures and informational warnings the same way the existing diagnostics do; warnings may be shown, but only hard failures block execution.
- **D-09:** Lexical-first remains the only executable retrieval baseline in v1, so operational commands should continue when lexical readiness is truly satisfied and stop when it is not.

### Test And Regression Boundary
- **D-10:** Regression coverage must prove that the same blocked configurations are rejected across `ingest`, `search`, and `agent-search`, not only in `init/doctor`.
- **D-11:** Regression coverage must also preserve the informational contract for `status`/`doctor`, so Phase 6 does not accidentally turn diagnostics into side-effecting or brittle command paths.

### the agent's Discretion
- Exact helper names, enum expansion, and whether the shared gate wrapper lives at CLI dispatch or a thinner operational boundary.
- Exact text formatting for reused gate output, as long as it stays aligned with the existing doctor/status contract.
- Exact test decomposition between CLI integration tests and narrower unit coverage.

### Deferred Ideas (OUT OF SCOPE)
- Making `embedding_only` executable with a real embedding backend
- Hybrid recall / rerank runtime behavior beyond lexical-first v1
- Non-CLI operational surfaces such as MCP or HTTP readiness gating
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FND-01 | Developer can initialize a local-first Rust application with a SQLite database, schema migrations, and deterministic startup checks for retrieval dependencies. [VERIFIED: `.planning/REQUIREMENTS.md`] | Runtime commands must respect the same startup/readiness contract after initialization, not only during `init`; otherwise deterministic startup checks are bypassable in practice. [VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`; VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `src/core/status.rs`] |
| FND-03 | Developer can inspect system health and index status from a CLI surface without requiring an LLM. [VERIFIED: `.planning/REQUIREMENTS.md`] | `status` / `doctor` / `inspect schema` must stay informational and explainable while operational commands consume the same typed readiness data for blocking decisions. [VERIFIED: `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md`; VERIFIED: `.planning/phases/01-foundation-kernel/01-04-SUMMARY.md`; VERIFIED: `src/core/doctor.rs`; VERIFIED: `src/core/status.rs`] |
| AGT-01 | Developer can use ordinary retrieval without invoking a language model or agent runtime. [VERIFIED: `.planning/REQUIREMENTS.md`] | `ingest` and `search` should remain thin local wrappers over ordinary services, but only when lexical runtime readiness is actually satisfied; otherwise the CLI lies about operability. [VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`; VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `tests/retrieval_cli.rs`] |
</phase_requirements>

## Summary

Phase 6 is not a new diagnostics subsystem. It is a propagation phase: the repository already has typed runtime state in `RuntimeReadiness`, an explainable snapshot in `StatusReport`, and command-path-sensitive failure/warning classification in `DoctorReport`, but `ingest`, `search`, and `agent-search` currently bypass all of that and go straight to `Database::open(...)` plus downstream services. [VERIFIED: `src/core/app.rs`; VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/doctor.rs`; VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`]

The key implementation insight is that reusing the current `doctor` behavior verbatim is not enough. Today `DoctorReport::evaluate` only hard-fails invalid mode/backend combinations everywhere, and reserved semantic modes only fail on the explicit `Doctor` path. It does not turn missing schema, missing base tables, bad local database files, or missing lexical sidecar indexes into operational failures for `ingest`, `search`, and `agent-search`, even though `StatusReport` already computes those states and `StatusReport::ready` already collapses them into the executable lexical baseline. [VERIFIED: `src/core/doctor.rs`; VERIFIED: `src/core/status.rs`; VERIFIED: `src/interfaces/cli.rs`]

**Primary recommendation:** keep `StatusReport` as the source of readiness facts, extend `DoctorReport` so operational command paths consume those facts as blocking failures, and add one shared CLI gate helper that runs before `Database::open(...)` for `ingest`, `search`, and `agent-search`. This preserves the existing typed diagnostics model, keeps service layers free of duplicated gate logic, and closes the exact audit gap without scope creep into semantic retrieval or new interfaces. [VERIFIED: `.planning/phases/06-runtime-gate-enforcement/06-CONTEXT.md`; VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/doctor.rs`; VERIFIED: `src/interfaces/cli.rs`]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Readiness fact collection | `src/core/status.rs` | `src/core/app.rs` | Status already owns schema/index/dependency state and renders operator-facing notes. [VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/app.rs`] |
| Failure vs warning policy | `src/core/doctor.rs` | `src/interfaces/cli.rs` | Doctor already classifies invalid combinations and reserved semantic modes by command path; Phase 6 should extend this contract rather than fork it. [VERIFIED: `src/core/doctor.rs`; VERIFIED: `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md`] |
| Operational gate invocation | `src/interfaces/cli.rs` | — | Current bypass exists at CLI entrypoints; that is the narrowest seam that can stop bad execution before `Database::open` and service invocation. [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`] |
| Ordinary retrieval / cognition behavior | `src/ingest/mod.rs`, `src/search/mod.rs`, `src/agent/orchestration.rs` | — | These modules should stay focused on core logic; they should not each re-implement mode/backend validation. [VERIFIED: `src/ingest/mod.rs`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/agent/orchestration.rs`] |

## Key Findings From Current Code

### 1. The bypass is entirely in CLI dispatch

- `init_command` explicitly collects `StatusReport`, evaluates `DoctorReport`, and stops on failure before touching the DB. [VERIFIED: `src/interfaces/cli.rs`]
- `status_command`, `doctor_command`, and `inspect_schema_command` are diagnostic-only paths. [VERIFIED: `src/interfaces/cli.rs`]
- `ingest_command`, `search_command`, and `agent_search_command` currently call `Database::open(app.db_path())?` directly, with no readiness gate at all. [VERIFIED: `src/interfaces/cli.rs`]

**Implication:** Phase 6 does not need a deep architectural rewrite. A shared preflight helper at the CLI boundary is sufficient if it delegates to the typed diagnostics layer instead of inventing new booleans or ad hoc string checks.

### 2. `StatusReport` already computes the facts operational commands need

- `StatusReport::collect` inspects whether the DB exists, whether schema/base tables exist, whether lexical sidecar tables and triggers exist, and whether the configured retrieval mode is executable. [VERIFIED: `src/core/status.rs`]
- `StatusReport::ready` is only `true` when the lexical baseline is truly runnable; it is `false` for `embedding_only`, `hybrid`, missing schema, missing base tables, bad DB files, and missing lexical sidecars. [VERIFIED: `src/core/status.rs`]  
- Non-SQLite files are downgraded into explicit `schema_state: missing` and explanatory notes instead of crashing the `status` command. [VERIFIED: `.planning/phases/01-foundation-kernel/01-04-SUMMARY.md`; VERIFIED: `src/core/status.rs`]

**Implication:** Operational gate failures should be synthesized from `StatusReport` capability states and notes, not from repeated SQL probes in the CLI layer.

### 3. `DoctorReport` needs path expansion, not replacement

- `DoctorReport::evaluate` already handles invalid combinations everywhere and reserved semantic modes on `CommandPath::Doctor`. [VERIFIED: `src/core/doctor.rs`]
- It currently has only `Init` and `Doctor` variants. [VERIFIED: `src/core/doctor.rs`]
- It does not upgrade `status.ready == false` into failures for operational paths. [VERIFIED: `src/core/doctor.rs`; VERIFIED: `src/core/status.rs`]

**Implication:** Phase 6 should extend `CommandPath` with operational variants and make them use a stricter readiness gate than `Init`, while preserving the current distinction that `status` and schema inspection remain informational.

## Recommended Runtime Gate Direction

### Command path model

Recommended exact target state:

```rust
pub enum CommandPath {
    Init,
    Doctor,
    Ingest,
    Search,
    AgentSearch,
}
```

Operational variants should behave like `Doctor` for reserved semantic-mode failures and additionally treat non-ready lexical/runtime states as hard failures. `Init` remains a narrower bootstrap gate so the existing post-init warning behavior is preserved. [VERIFIED: `.planning/phases/01-foundation-kernel/01-04-SUMMARY.md`; VERIFIED: `src/core/doctor.rs`; ASSUMED]

### Failure synthesis

Recommended operational blocking rules:

- Preserve current failures:
  - `embedding_only requires a non-disabled embedding backend`
  - `hybrid requires an embedding backend for the secondary path`
- Reuse current reserved-mode failures for operational commands:
  - `embedding_only is reserved but not implemented in Phase 1`
  - `hybrid keeps lexical as the primary baseline, but the embedding secondary path is reserved in Phase 1`
- Add operational failures when `StatusReport` proves the baseline is not runnable:
  - missing schema -> `database schema is not initialized yet; run \`agent-memos init\` to create it`
  - missing base tables -> `foundation base tables are incomplete or missing`
  - missing lexical sidecars in lexical mode -> `lexical sidecar indexes are missing or incomplete`
  - existing unreadable or non-SQLite DB -> preserve the status-generated note as a blocking failure on operational paths

These failures should be rendered through `DoctorReport::render_text()` so the output shape stays aligned with existing diagnostics. [VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/doctor.rs`; ASSUMED]

### CLI integration

Recommended exact seam:

- Add one helper in `src/interfaces/cli.rs` that:
  1. collects `StatusReport`
  2. evaluates `DoctorReport` for `CommandPath::Ingest`, `Search`, or `AgentSearch`
  3. prints `doctor.render_text()` and returns `ExitCode::FAILURE` when not ready
  4. otherwise returns control so the command can open the DB and invoke services

This keeps `IngestService`, `SearchService`, and `AgentSearchOrchestrator` unchanged and local-first. [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `src/ingest/mod.rs`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/agent/orchestration.rs`]

## Testing Direction

### Best regression surface

The strongest test surface is command-level CLI integration, because the audit gap is about entrypoint behavior, not only internal library state. [VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`; VERIFIED: `tests/status_cli.rs`; VERIFIED: `tests/retrieval_cli.rs`]

Recommended split:

- Extend `tests/status_cli.rs` for informational semantics:
  - `status` remains exit-0 on reserved modes and bad DB files
  - `doctor` keeps structured failure output
  - `inspect schema` remains diagnostic
- Add a dedicated CLI regression file, e.g. `tests/runtime_gate_cli.rs`, for:
  - `ingest`, `search`, and `agent-search` all fail under the same reserved/invalid configs
  - lexical-ready config still allows ordinary retrieval and agent-search execution
  - missing-init / bad-db / missing-index lexical states block operational commands consistently

This avoids overloading `tests/agent_search.rs`, which is currently orchestration-focused rather than CLI-focused. [VERIFIED: `tests/agent_search.rs`; VERIFIED: `tests/status_cli.rs`; VERIFIED: `tests/retrieval_cli.rs`]

## Patterns To Reuse

### Pattern 1: Typed facts first, rendering second

- `StatusReport` and `DoctorReport` compute structured state, then CLI renders it. [VERIFIED: `src/core/status.rs`; VERIFIED: `src/core/doctor.rs`; VERIFIED: `src/interfaces/cli.rs`]
- Phase 6 should preserve this and avoid embedding raw readiness conditionals directly in each command.

### Pattern 2: Thin CLI wrappers

- Phase 2 established that CLI commands should stay thin wrappers over service objects. [VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`; VERIFIED: `src/interfaces/cli.rs`]
- Phase 6 should add a shared preflight wrapper, not command-specific copies of runtime validation logic.

### Pattern 3: Reserved modes stay explicit

- Phase 1 locked the rule that `embedding_only` and `hybrid` stay visible rather than silently downgrading to lexical-only. [VERIFIED: `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md`; VERIFIED: `src/core/app.rs`; VERIFIED: `src/core/doctor.rs`]
- Phase 6 should reinforce that rule by making operational commands fail truthfully instead of continuing with misleading behavior.

## Anti-Patterns To Avoid

- **Do not add per-command ad hoc checks in `ingest_command`, `search_command`, and `agent_search_command` separately.** That reintroduces rule drift immediately. [VERIFIED: `src/interfaces/cli.rs`]
- **Do not gate inside service layers just to compensate for the CLI bypass.** The audit gap is at the entrypoint seam, and duplicating validation in `IngestService` / `SearchService` / orchestrator will blur domain boundaries. [VERIFIED: `src/ingest/mod.rs`; VERIFIED: `src/search/mod.rs`; VERIFIED: `src/agent/orchestration.rs`]
- **Do not downgrade reserved semantic modes into lexical-only fallback.** That would violate explicit phase decisions and make operator output misleading. [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md`]
- **Do not make `status` or schema inspection start failing just because operational commands now block.** The diagnostic contract must remain informational. [VERIFIED: `.planning/phases/01-foundation-kernel/01-04-SUMMARY.md`; VERIFIED: `src/interfaces/cli.rs`]

## Recommended Plan Shape

Phase 6 cleanly fits two executable plans:

1. **Plan 06-01:** extend the command-path model and add one shared operational gate in CLI dispatch so runtime commands stop before DB/service execution.
2. **Plan 06-02:** add focused regression coverage for reserved/invalid modes, lexical not-ready states, and informational command semantics so the bypass cannot reappear silently.

This split matches the roadmap slots, keeps code motion small, and gives Phase 7 a cleaner baseline for follow-up evidence integration without mixing cognition work into runtime gate closure. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/v1.0-MILESTONE-AUDIT.md`]
