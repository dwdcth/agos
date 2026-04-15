## Deferred Items

- 2026-04-15: `cargo clippy --all-targets -- -D warnings` fails on `tests/ingest_pipeline.rs:12` due an unused `NormalizedSource` import. This predates Plan `02-02` changes and was left untouched per executor scope rules.
