---
status: complete
phase: 08-embedding-backend-and-index-foundation
source: [08-01-SUMMARY.md, 08-02-SUMMARY.md, 08-03-SUMMARY.md]
started: 2026-04-16T09:30:00+08:00
updated: 2026-04-16T09:30:00+08:00
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Embedding Backend Stays Optional And Lexical-First
expected: A developer should be able to configure a concrete embedding backend foundation (`builtin`) without breaking lexical-only defaults. `status` and `doctor` should make it clear that lexical-only remains the stable baseline, and that embedding-only / hybrid are still foundation-only rather than silently enabled retrieval modes.
result: pass

### 2. Ingest Persists Chunk-Aligned Embedding Sidecars
expected: With embedding foundation enabled, ingest should persist additive embedding sidecar rows aligned to chunk-level authority records; with embeddings disabled, lexical-only ingest should still succeed and create no embedding rows.
result: pass

### 3. Embedding Vector Sidecar Readiness Is Inspectable
expected: `status` and `inspect schema` should expose embedding/vector substrate readiness separately from lexical index readiness, so operators can tell when the embedding sidecar/index state is ready or missing without confusing it with lexical readiness.
result: pass

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
