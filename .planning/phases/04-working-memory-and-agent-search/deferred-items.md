# Deferred Items

## 2026-04-16

- `tests/ingest_pipeline.rs:12` — pre-existing unused `NormalizedSource` import still blocks `rtk cargo clippy --all-targets -- -D warnings`. This file is outside Plan `04-01` scope, so it was not modified during working-memory execution.
