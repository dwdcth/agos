---
status: complete
phase: 10-dual-channel-diagnostics-and-service-compatibility
source: [10-01-SUMMARY.md, 10-02-SUMMARY.md]
started: 2026-04-17T14:30:00+08:00
updated: 2026-04-17T14:30:00+08:00
---

## Current Test

[testing complete]

## Tests

### 1. Status And Doctor Truthfully Explain Dual-Channel Readiness
expected: `status` and `doctor` should make lexical, embedding, and hybrid readiness operator-readable, including active and gated channels, without breaking lexical-only defaults.
result: pass

### 2. Search Surface Supports Explicit Lexical-Only, Embedding-Only, And Hybrid Mode Selection
expected: The ordinary `search` CLI should accept one additive `--mode` control and route through the shared retrieval service for lexical-only, embedding-only, and hybrid execution.
result: pass

### 3. Agent-Search Reuses Ordinary Retrieval Under Dual-Channel Modes
expected: `agent-search` should accept the same `--mode` control, preserve structured citations/report output, and continue to reuse the ordinary retrieval seam instead of introducing a semantic-only bypass.
result: pass

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

None.
