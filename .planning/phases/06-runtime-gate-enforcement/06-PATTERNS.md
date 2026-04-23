# Phase 6: Runtime Gate Enforcement - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 6
**Analogs found:** 6 / 6

## Revision Notes

- Phase 6 is a propagation/refinement phase, not a new subsystem.
- The main implementation seam is already obvious in the codebase: typed diagnostics in `core`, thin dispatch in `interfaces/cli`.
- The safest route is to keep facts in `StatusReport`, failure policy in `DoctorReport`, and command enforcement in one reusable CLI helper.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/core/doctor.rs` | policy | transform | `src/core/doctor.rs` | exact |
| `src/core/status.rs` | diagnostics | transform | `src/core/status.rs` | exact |
| `src/interfaces/cli.rs` | interface | request/response | `src/interfaces/cli.rs` | exact |
| `tests/status_cli.rs` | test | command-level | `tests/status_cli.rs` | exact |
| `tests/retrieval_cli.rs` | test | command-level | `tests/retrieval_cli.rs` | exact |
| `tests/runtime_gate_cli.rs` | test | command-level | `tests/status_cli.rs` + `tests/retrieval_cli.rs` | composite |

## Pattern Assignments

### `src/core/doctor.rs` (policy, transform)

**Analog:** `src/core/doctor.rs:1-87`

**Pattern to copy**

```rust
pub enum CommandPath {
    Init,
    Doctor,
}

impl DoctorReport {
    pub fn evaluate(status: &StatusReport, command_path: CommandPath) -> Self {
        let mut failures = Vec::new();
        let mut warnings = Vec::new();
        // classify from typed status
    }
}
```

**Phase 6 application**

- Extend `CommandPath` rather than creating a second operational-gate enum elsewhere.
- Keep `DoctorReport { ready, failures, warnings }` as the single rendered contract.
- Promote `StatusReport` non-ready states into failures only for operational command paths; keep `Init` narrower and `Status` non-blocking.

### `src/core/status.rs` (diagnostics, transform)

**Analog:** `src/core/status.rs:45-199`

**Pattern to copy**

```rust
let ready = app.readiness.ready
    && matches!(inspection.schema_state, CapabilityState::Ready)
    && matches!(inspection.base_table_state, CapabilityState::Ready)
    && match app.config.retrieval.mode {
        RetrievalMode::LexicalOnly => matches!(index_readiness, CapabilityState::Ready),
        RetrievalMode::EmbeddingOnly | RetrievalMode::Hybrid => false,
    };
```

**Phase 6 application**

- Reuse `ready`, capability states, and `readiness_notes` as gate inputs.
- Do not rerun SQL inspection in the CLI layer.
- If additional notes are needed for operational failures, derive them from existing capability states instead of bolting on new ad hoc strings.

### `src/interfaces/cli.rs` (interface, request/response)

**Analog:** `src/interfaces/cli.rs:108-221` and `src/interfaces/cli.rs:230-324`

**Pattern to copy**

```rust
let preflight_status = StatusReport::collect(app)?;
let doctor = DoctorReport::evaluate(&preflight_status, CommandPath::Init);

if !doctor.ready {
    println!("{}", doctor.render_text());
    return Ok(ExitCode::FAILURE);
}
```

**Phase 6 application**

- Extract this gate shape into one shared helper before `Database::open(...)`.
- Call the helper from `ingest_command`, `search_command`, and `agent_search_command`.
- Keep JSON/text rendering after service calls exactly as Phase 2/4 established.

### `tests/status_cli.rs` (test, command-level)

**Analog:** `tests/status_cli.rs:1-329`

**Pattern to copy**

- temp-dir fixture per test
- config writer with explicit `mode` / `backend`
- `run_cli(&config_path, &[...])`
- assert on exit status plus rendered stdout text

**Phase 6 application**

- Extend this file for “diagnostic paths stay informational” assertions.
- Keep operator-facing string checks exact; this file is already the precedent for exit semantics plus text contract.

### `tests/retrieval_cli.rs` (test, command-level)

**Analog:** `tests/retrieval_cli.rs:1-260`

**Pattern to copy**

- CLI fixture with config file and temp DB
- successful `ingest` / `search` end-to-end coverage
- library + CLI parity checks

**Phase 6 application**

- Reuse the same fixture style to prove lexical-ready commands still work after gating.
- Do not overload this file with all negative-path gate coverage; keep it focused on ordinary retrieval success-path parity if Phase 6 only needs a narrow success assertion here.

### `tests/runtime_gate_cli.rs` (test, command-level)

**Composite analog:** `tests/status_cli.rs` + `tests/retrieval_cli.rs`

**Phase 6 application**

- Build a new cross-command regression file if needed, rather than stuffing all runtime-gate cases into unrelated test modules.
- Use one shared fixture helper and assert all three operational commands:
  - fail for invalid/reserved semantic configs
  - fail for lexical missing-init / bad-db / missing-index states
  - succeed for lexical-ready config

## Recommended File Ownership

- `src/core/doctor.rs` owns command-path-specific failure/warning policy.
- `src/core/status.rs` remains the only source of readiness facts and explanatory notes.
- `src/interfaces/cli.rs` owns one reusable operational gate helper and command dispatch.
- `tests/status_cli.rs` owns informational command semantics.
- `tests/runtime_gate_cli.rs` owns cross-command operational blocking regressions.

## Anti-Patterns To Avoid

- Adding separate `gate_ingest`, `gate_search`, and `gate_agent_search` implementations in `src/interfaces/cli.rs`.
- Repeating mode/backend strings in tests without using the existing config-writer pattern.
- Using `tests/agent_search.rs` for CLI process tests; that file is currently orchestration-focused and has a different responsibility boundary.
