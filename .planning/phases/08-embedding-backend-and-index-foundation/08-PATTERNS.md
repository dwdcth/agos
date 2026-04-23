# Phase 8: Embedding Backend And Index Foundation - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 8
**Analogs found:** 8 / 8

## Revision Notes

- Phase 8 should mirror Phase 2/6 patterns: typed config/readiness, additive schema, truthful diagnostics.
- The main caution is not to accidentally turn semantic retrieval “on” while only building the substrate.
- Chunk-aligned storage is the safest analog because the whole codebase already treats chunked authority rows as retrieval units.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/core/config.rs` | config | parse/validate | `src/core/config.rs` | exact |
| `src/core/app.rs` | readiness | transform | `src/core/app.rs` | exact |
| `src/core/status.rs` | diagnostics | inspect/render | `src/core/status.rs` | exact |
| `src/core/doctor.rs` | policy | gate/warn | `src/core/doctor.rs` | exact |
| `migrations/0006_embedding_foundation.sql` | migration | additive storage | `migrations/0005_rumination_writeback.sql` + prior additive migrations | composite |
| `src/core/migrations.rs` | config | migration chain | `src/core/migrations.rs` | exact |
| `src/ingest/mod.rs` | service | ingest->persist | `src/ingest/mod.rs` | exact |
| `tests/status_cli.rs` / `tests/foundation_schema.rs` / `tests/ingest_pipeline.rs` | tests | integration | existing phase 2/6 test surfaces | exact/composite |

## Pattern Assignments

### `src/core/config.rs` (config, parse/validate)

**Analog:** `src/core/config.rs:1-140`

**Phase 8 application**

- Extend `EmbeddingBackend` with real backend variants while preserving `Disabled` as default.
- Keep `EmbeddingConfig { backend, model, endpoint }` explicit and typed.
- Follow the current parse-test style: one test per supported mode/backend contract.

### `src/core/app.rs` (readiness, transform)

**Analog:** `src/core/app.rs:27-80`

**Phase 8 application**

- Expand `RuntimeReadiness` notes to reflect real embedding backend readiness and optional substrate state.
- Preserve explicit lexical/hybrid/embedding intent; do not collapse into one “ready” boolean for all retrieval modes.

### `src/core/status.rs` (diagnostics, inspect/render)

**Analog:** `src/core/status.rs:45-199`

**Phase 8 application**

- Reuse capability-state reporting style for embedding/vector readiness.
- Additive diagnostic fields should fit the existing `dependencies:` and `notes:` model.
- Keep `status` informational even when embedding substrate is missing or misconfigured.

### `migrations/0006_embedding_foundation.sql` (migration, additive storage)

**Analog:** `migrations/0002_ingest_foundation.sql`, `migrations/0004_truth_layer_governance.sql`, `migrations/0005_rumination_writeback.sql`

**Phase 8 application**

- Add embedding tables and optional vector-sidecar/index state additively.
- Use foreign keys back to `memory_records(id)` where persistence is record/chunk aligned.
- Do not rewrite prior authority/search/governance tables.

### `src/ingest/mod.rs` (service, ingest->persist)

**Analog:** `src/ingest/mod.rs:47-108`

**Phase 8 application**

- Piggyback on ingest completion to compute/persist embeddings or schedule embedding persistence.
- Preserve the existing normalize -> chunk -> persist flow shape.
- If backend is disabled/not ready, ingest should remain usable for lexical-only operation.

### `tests/status_cli.rs` / `tests/foundation_schema.rs` / `tests/ingest_pipeline.rs`

**Phase 8 application**

- `tests/status_cli.rs` owns config/readiness/operator-surface regressions.
- `tests/foundation_schema.rs` (or an adjacent schema test) owns additive embedding schema assertions.
- `tests/ingest_pipeline.rs` or a new dedicated ingest+embedding test owns chunk-aligned embedding persistence behavior.

## Anti-Patterns To Avoid

- turning embedding backend presence into an implicit green light for `embedding_only` behavior before Phase 9
- storing vectors in opaque blobs disconnected from `record_id`
- introducing document-level-only embedding storage while retrieval/report contracts remain chunk-based
- bypassing ingest and writing embedding rows through a standalone one-off tool path only
