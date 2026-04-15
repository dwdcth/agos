---
phase: 02-ingest-and-lightweight-retrieval
verified: 2026-04-15T13:30:42Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 7/8
  gaps_closed:
    - "Status and readiness output are truthful about lexical capability while semantic capability stays deferred."
  gaps_remaining: []
  regressions: []
---

# Phase 2: Ingest And Lightweight Retrieval Verification Report

**Phase Goal:** 打通普通检索主线，使系统可以在不依赖 LLM 和模型文件的前提下 ingest 资料并执行中文词法检索 + 轻量关键词加权排序。
**Verified:** 2026-04-15T13:30:42Z
**Status:** passed
**Re-verification:** Yes — focused closure of the prior lexical-only status wording gap

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Developer can ingest notes, documents, or conversation text into normalized and chunked memory units with source linkage intact. | ✓ VERIFIED | Retained from prior verification; no code changes in the ingest path were involved in Plan `02-04`. |
| 2 | Agent or developer can run ordinary lexical search over the corpus and see Rust-side lightweight keyword weighting affect final ranking. | ✓ VERIFIED | Retained from prior verification; no search-path regressions were introduced by the status-wording fix. |
| 3 | Retrieval results include source, scope, and validity metadata that explain why each memory was returned. | ✓ VERIFIED | Retained from prior verification; the focused re-verification touched only status wording. |
| 4 | Ordinary retrieval is fully usable from CLI or library APIs without invoking Rig or any LLM. | ✓ VERIFIED | Retained from prior verification; no Rig, semantic retrieval, RRF, or embedding execution was introduced by the gap closure. |
| 5 | Persisted chunks keep source linkage, chunk ordering, provenance anchors, and explicit validity-window metadata on the authority store. | ✓ VERIFIED | Retained from prior verification; no authority-schema or repository changes were made in Plan `02-04`. |
| 6 | Lexical indexing and scoring remain lexical-first only: no semantic retrieval execution, embeddings, Rig, or RRF are required in Phase 2. | ✓ VERIFIED | Retained from prior verification; current code remains lexical-first only, and the gap-closure changes are confined to `src/core/app.rs` plus status tests. |
| 7 | Status and readiness output are truthful about lexical capability while semantic capability stays deferred. | ✓ VERIFIED | `src/core/app.rs:42-50` now emits Phase 2 lexical-baseline notes; `rtk cargo test --test status_cli -- --nocapture` passed; a live `init` + `status` check showed `lexical_dependency_state: ready`, `index_readiness: ready`, and notes that no longer claim lexical retrieval or index creation are deferred. |
| 8 | Filtering by scope, record type, truth layer, and validity affects candidate selection and is visible in the result trace. | ✓ VERIFIED | Retained from prior verification; no retrieval-filter code changed during this focused gap closure. |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `src/core/app.rs` | Truthful `lexical_only` runtime-readiness notes while preserving deferred semantic-mode notes | ✓ VERIFIED | `RuntimeReadiness::from_config()` now reports the real Phase 2 lexical baseline and no longer carries the stale deferred lexical wording. |
| `tests/status_cli.rs` | Canonical CLI regression coverage for lexical-only status output | ✓ VERIFIED | The suite still passes after the wording fix, and the focused `status` regression area remains wired to the real CLI path. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `src/core/app.rs` | `src/core/status.rs` | `StatusReport::collect()` renders `app.readiness.notes` | WIRED | The fixed readiness notes flow through to `status` output unchanged. |
| `tests/status_cli.rs` | `agent-memos status` | Real CLI integration coverage | WIRED | `rtk cargo test --test status_cli -- --nocapture` passed against the current `status` surface. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Status CLI suite stays green after the lexical-only wording fix | `rtk cargo test --test status_cli -- --nocapture` | `6 passed` | ✓ PASS |
| Initialized `lexical_only` `status` output is truthful | Temp config + `rtk cargo run --quiet -- --config <tmp> init` then `... status` | Output showed `lexical_dependency_state: ready`, `index_readiness: ready`, and notes without the old deferred lexical/index-creation claim | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `RET-01` | `02-02`, `02-04` | ordinary lexical search over Chinese and PinYin using `libsimple`-backed FTS | ✓ SATISFIED | The remaining verification gap for truthful lexical-only readiness/status wording is now closed, and the lexical path remains the active Phase 2 baseline. |

### Gaps Summary

The previously failed lexical-only status wording truth is now satisfied. Phase 2's initialized `lexical_only` status output no longer contradicts its own ready lexical fields, and semantic capability remains deferred for `embedding_only` and `hybrid`.

---

_Verified: 2026-04-15T13:30:42Z_
_Verifier: Claude (gsd-verifier)_
