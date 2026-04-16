---
phase: 09-dual-channel-retrieval-fusion
verified: 2026-04-16T11:45:00+08:00
status: passed
score: 7/7 must-haves verified
overrides_applied: 0
---

# Phase 9: Dual-Channel Retrieval Fusion Verification Report

**Phase Goal:** 在 ordinary retrieval 中引入 lexical-first + embedding second-channel 的 recall / fusion / rerank，使 dual-channel 结果仍然保持 explainable。  
**Verified:** 2026-04-16T11:45:00+08:00  
**Status:** passed  
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Retrieval-relevant fields from the real root `config.toml` are parsed into a shared config base. | ✓ VERIFIED | `RootRuntimeConfig::load_from(...)` and the dual-channel config parser in [src/core/config.rs](/home/tongyuan/project/agent_memos/src/core/config.rs); verified by [tests/dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L8). |
| 2 | One shared parsed config base can derive lexical-only, embedding-only, and hybrid runtime variants. | ✓ VERIFIED | `retrieval_mode_variants()` in [src/core/config.rs](/home/tongyuan/project/agent_memos/src/core/config.rs) and assertions in [tests/dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L24). |
| 3 | `SearchService` now supports lexical-only, embedding-only, and hybrid retrieval through one ordinary retrieval seam. | ✓ VERIFIED | Dual-channel dispatch in [src/search/mod.rs](/home/tongyuan/project/agent_memos/src/search/mod.rs); behavior covered in [tests/dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L69). |
| 4 | Hybrid mode dedupes lexical and embedding candidates by authority record identity before final ranking. | ✓ VERIFIED | Candidate merge/dedupe logic in [src/search/mod.rs](/home/tongyuan/project/agent_memos/src/search/mod.rs); regression in [tests/dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L122). |
| 5 | Lexical-first explanation behavior remains intact even when embedding contributes retrieval signal. | ✓ VERIFIED | `SearchResult` still uses lexical citations/provenance as its main explanation surface; lexical-only regressions remain green in [tests/lexical_search.rs](/home/tongyuan/project/agent_memos/tests/lexical_search.rs). |
| 6 | Final traces explicitly report lexical-only, embedding-only, or hybrid contribution. | ✓ VERIFIED | `ChannelContribution` and additive trace enrichment in [src/search/rerank.rs](/home/tongyuan/project/agent_memos/src/search/rerank.rs); regression in [tests/dual_channel_retrieval.rs](/home/tongyuan/project/agent_memos/tests/dual_channel_retrieval.rs#L165). |
| 7 | Ordinary retrieval consumers remain compatible after the trace/rerank extension. | ✓ VERIFIED | Retrieval CLI contract remains green in [tests/retrieval_cli.rs](/home/tongyuan/project/agent_memos/tests/retrieval_cli.rs) and downstream test fixtures were updated successfully in agent-search / working-memory / rumination suites. |

**Score:** 7/7 truths verified

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `DCR-01` | `09-01`, `09-02` | Ordinary retrieval can run lexical-first plus embedding second-channel in one flow. | ✓ SATISFIED | Config-derived mode matrix and SearchService mode dispatch verified by `tests/dual_channel_retrieval.rs`. |
| `DCR-02` | `09-02`, `09-03` | Lexical remains the primary explanation source. | ✓ SATISFIED | Lexical search tests stay green and final traces remain additive rather than replacing lexical citations. |
| `DCR-03` | `09-02` | Fusion dedupes and ranks lexical and embedding candidates into one stable result contract. | ✓ SATISFIED | Hybrid dedupe regression plus stable `SearchResponse`/`SearchResult` behavior. |
| `DCR-04` | `09-01`, `09-03` | Search results expose enough trace data to explain lexical vs embedding contribution. | ✓ SATISFIED | `ChannelContribution` trace field and dual-channel trace regressions. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Config-derived dual-channel mode matrix | `cargo test --test dual_channel_retrieval -- --nocapture` | `5 passed` | ✓ PASS |
| Lexical-only baseline stability | `cargo test --test lexical_search -- --nocapture` | `2 passed` | ✓ PASS |
| Retrieval CLI compatibility after trace extension | `cargo test --test retrieval_cli -- --nocapture` | `3 passed` | ✓ PASS |
| Full lint hygiene | `cargo clippy --all-targets -- -D warnings` | Passed | ✓ PASS |

### Gaps Summary

No blocking gaps found. Phase 09 delivers the planned dual-channel retrieval behavior while keeping lexical-first explainability and consumer compatibility intact.

---

_Verified: 2026-04-16T11:45:00+08:00_  
_Verifier: Codex_
