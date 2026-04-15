---
phase: 03-truth-layer-governance
verified: 2026-04-15T15:16:49Z
status: passed
score: 7/7 must-haves verified
overrides_applied: 0
---

# Phase 3: Truth Layer Governance Verification Report

**Phase Goal:** 将 T1/T2/T3 真值分层、私有假设边界和共享晋升规则固化到系统模型中。  
**Verified:** 2026-04-15T15:16:49Z  
**Status:** passed  
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | System stores T1, T2, and T3 memory structures as distinct truth-layer states in storage and service APIs. | ✓ VERIFIED | `src/memory/truth.rs:253` defines `TruthRecord::{T1,T2,T3}`; `src/memory/repository.rs:578` maps `memory_records.truth_layer` into typed projections and hydrates layer-specific state; `tests/truth_governance.rs:105` proves typed reads for all three layers. |
| 2 | T3 records preserve provenance, confidence, and revocability so private hypotheses remain auditable. | ✓ VERIFIED | `migrations/0004_truth_layer_governance.sql:1` adds `truth_t3_state`; `src/memory/repository.rs:103` auto-creates default T3 governance state on insert; `tests/truth_governance.rs:136` and `tests/truth_governance.rs:447` verify default confidence/revocation state and post-review auditability. |
| 3 | T3-to-T2 promotion is blocked unless evidence review and metacognitive approval data are present. | ✓ VERIFIED | `src/memory/governance.rs:252` rejects approval when evidence is missing or any gate is not passed; `src/memory/governance.rs:273` checks all four gates explicitly; `tests/truth_governance.rs:286` verifies approval fails until metacognitive approval is recorded. |
| 4 | Approved T3 promotion creates a derived T2 record without mutating the source T3 record in place. | ✓ VERIFIED | `src/memory/governance.rs:287` builds a derived T2 authority row and `src/memory/governance.rs:293` inserts it via `MemoryRepository`; `src/memory/governance.rs:499` preserves source provenance and content while changing only the derived row's layer; `tests/truth_governance.rs:376` confirms source stays T3 and derived row is T2. |
| 5 | T2-to-T1 changes are represented as proposals/candidates, not automatic ontology rewrites. | ✓ VERIFIED | `src/memory/governance.rs:310` only creates an `OntologyCandidate`; `src/memory/repository.rs:532` persists candidate rows in `truth_ontology_candidates`; `tests/truth_governance.rs:529` confirms candidate creation does not create or mutate any T1 authority row. |
| 6 | Governance APIs expose pending promotion reviews and pending ontology candidates as distinct queues while preserving typed truth projections. | ✓ VERIFIED | `src/memory/governance.rs:357`, `src/memory/governance.rs:364`, and `src/memory/governance.rs:370` expose typed truth lookup plus separate pending review/candidate queues; `tests/truth_governance.rs:629` verifies queue separation and typed reads remain distinct. |
| 7 | Phase 2 lexical retrieval, filtering, and citations still work after schema version 4 lands. | ✓ VERIFIED | `src/search/lexical.rs:37` and `src/search/lexical.rs:73` still query only `memory_records_fts JOIN memory_records`; `tests/foundation_schema.rs:148` verifies the FTS sidecar and governance tables coexist, plus rejects `vec`/`rig` tables; `tests/retrieval_cli.rs:105` passes on a schema-version-4 database. |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `migrations/0004_truth_layer_governance.sql` | Additive governance tables and indexes over the existing authority backbone | ✓ VERIFIED | Substantive migration with `truth_t3_state`, `truth_promotion_reviews`, `truth_promotion_evidence`, and `truth_ontology_candidates`; no rewrite of `memory_records` or `memory_records_fts`. Wired by `src/core/migrations.rs:7`. |
| `src/core/migrations.rs` | Register schema v4 after the lexical sidecar migration | ✓ VERIFIED | `M::up(TRUTH_LAYER_GOVERNANCE_SQL)` is appended after the Phase 2 lexical migration at `src/core/migrations.rs:25`. |
| `src/memory/truth.rs` | Typed truth-layer and governance models | ✓ VERIFIED | Defines typed enums and structs for T3 state, promotion reviews/evidence, ontology candidates, and `TruthRecord` projections. Imported by `src/memory/repository.rs:8` and used by `src/memory/governance.rs:9`. |
| `src/memory/repository.rs` | Authority-row plus governance-table persistence and typed reads | ✓ VERIFIED | Persists base records, inserts default T3 state, manages review/evidence/candidate tables, and exposes `get_truth_record()` without moving retrieval off the authority table. |
| `src/memory/governance.rs` | Service-layer promotion/candidate orchestration and queue APIs | ✓ VERIFIED | Synchronous governance service enforces source-layer checks, review metadata, evidence attachment, all-gates-passed approval, candidate-only T2→T1 flow, and pending queue reads. |
| `tests/truth_governance.rs` | Integration coverage for TRU-01..TRU-04 | ✓ VERIFIED | Eight tests cover typed truth projections, T3 defaults, pending/rejected/approved promotion flows, candidate-only T2→T1 flow, and queue APIs. |
| `tests/retrieval_cli.rs` | Regression proving Phase 2 retrieval still works on schema v4 | ✓ VERIFIED | `library_search_returns_citations_and_filter_trace` asserts filter trace, citations, validity metadata, and lexical-first scoring on a version-4 database. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `src/core/migrations.rs` | `migrations/0004_truth_layer_governance.sql` | `include_str!` + `M::up(...)` registration | ✓ WIRED | `src/core/migrations.rs:7-28` loads and applies the new migration after the lexical sidecar migration. |
| `src/memory/repository.rs` | `src/memory/truth.rs` | Typed imports and `TruthRecord` projection | ✓ WIRED | `src/memory/repository.rs:8-12` imports typed governance models; `src/memory/repository.rs:578-600` returns layer-aware truth records. |
| `src/memory/governance.rs` | `src/memory/repository.rs` | Repository-backed orchestration | ✓ WIRED | `TruthGovernanceService` owns a `MemoryRepository` at `src/memory/governance.rs:143-151` and routes promotion/candidate operations through repository calls instead of SQL. |
| `src/memory/governance.rs` | `memory_records` | Derived T2 authority row creation | ✓ WIRED | `src/memory/governance.rs:287-307` builds and inserts a derived T2 `MemoryRecord`; source T3 stays unchanged. |
| `src/memory/governance.rs` | `truth_ontology_candidates` | Candidate-only T2→T1 persistence | ✓ WIRED | `src/memory/governance.rs:336-354` creates an `OntologyCandidate`; `src/memory/repository.rs:532-575` persists it to `truth_ontology_candidates`. |
| `tests/retrieval_cli.rs` | `memory_records_fts` | `SearchService` compatibility regression | ✓ WIRED | `tests/retrieval_cli.rs:155` exercises `SearchService`; `src/search/lexical.rs:37-48` and `src/search/lexical.rs:73-84` confirm the runtime path still uses the lexical sidecar. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `src/memory/repository.rs` | `TruthRecord` projections | `memory_records` plus `truth_t3_state`, `truth_promotion_reviews`, and `truth_ontology_candidates` SQL reads | Yes — `get_truth_record()` loads base rows and layer-specific tables directly from SQLite at `src/memory/repository.rs:578-600` | ✓ FLOWING |
| `src/memory/governance.rs` | `PromotionReviewReport` / `PromotionApprovalReport` / `OntologyCandidate` | Repository reads/writes plus derived-record construction | Yes — service methods fetch persisted reviews/evidence, enforce gates, and write real authority/candidate rows at `src/memory/governance.rs:154-374` | ✓ FLOWING |
| `src/search/lexical.rs` | `LexicalCandidate.record` | `memory_records_fts JOIN memory_records` | Yes — lexical recall still comes from the FTS sidecar and authority rows, with no governance-table dependency in `src/search/lexical.rs:15-85` | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Schema v4 bootstraps additive truth-governance tables without breaking the lexical backbone | `rtk cargo test --test foundation_schema foundation_migration_bootstraps_clean_db -- --nocapture` | `1 passed, 6 filtered out` | ✓ PASS |
| Phase 2 retrieval contract still works on a schema-version-4 database | `rtk cargo test --test retrieval_cli library_search_returns_citations_and_filter_trace -- --nocapture` | `1 passed, 2 filtered out` | ✓ PASS |
| T3 promotion requires all gate checks before derived T2 creation | `rtk cargo test --test truth_governance t3_promotion_requires_all_gate_checks -- --nocapture` | `1 passed, 7 filtered out` | ✓ PASS |
| T2→T1 remains candidate-only with no automatic T1 mutation | `rtk cargo test --test truth_governance t2_to_t1_creates_candidate_without_t1_mutation -- --nocapture` | `1 passed, 7 filtered out` | ✓ PASS |
| Full Phase 3 governance regression suite stays green | `rtk cargo test --test truth_governance -- --nocapture` | `8 passed` | ✓ PASS |
| Static quality gate stays clean for repository/service/test code | `rtk cargo clippy --all-targets -- -D warnings` | `0 errors`; only external Cargo config deprecation warnings from `/home/tongyuan/.cargo/config` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `TRU-01` | `03-01`, `03-03` | System distinguishes T1, T2, and T3 records in storage and service APIs | ✓ SATISFIED | Typed `TruthRecord` variants in `src/memory/truth.rs:253`; repository/service truth reads in `src/memory/repository.rs:578` and `src/memory/governance.rs:357`; verified by `tests/truth_governance.rs:105` and `tests/truth_governance.rs:629`. |
| `TRU-02` | `03-01`, `03-02` | T3 records carry explicit provenance, confidence, and revocability markers | ✓ SATISFIED | `truth_t3_state` schema at `migrations/0004_truth_layer_governance.sql:1`; automatic T3 state insert at `src/memory/repository.rs:103`; audit preservation after review at `tests/truth_governance.rs:447`. |
| `TRU-03` | `03-02` | T3 promotion toward T2 only through an explicit gate with evidence review and metacognitive approval state | ✓ SATISFIED | Promotion gate states stored in `truth_promotion_reviews`; approval checks in `src/memory/governance.rs:252-285`; derived T2 creation and source preservation verified by `tests/truth_governance.rs:286` and `tests/truth_governance.rs:376`. |
| `TRU-04` | `03-03` | T2-to-T1 ontology candidates can be created without automatically rewriting shared ontology | ✓ SATISFIED | Candidate persistence in `truth_ontology_candidates` via `src/memory/repository.rs:532`; candidate-only service flow in `src/memory/governance.rs:310`; verified by `tests/truth_governance.rs:529`. |

All Phase 03 requirement IDs from the roadmap (`TRU-01`, `TRU-02`, `TRU-03`, `TRU-04`) appear in Phase 03 plan frontmatter. No orphaned Phase 03 requirements were found.

### Anti-Patterns Found

No blocker or warning anti-patterns were found in the scanned Phase 03 implementation files. `rg` scans over the phase artifacts found no placeholder markers, empty implementations, hardcoded empty user-visible data paths, or console-log-only behavior.

Residual test gap from the disconfirmation pass: there is no dedicated search test that runs ordinary retrieval immediately after ontology candidate creation. This does not block the phase because `src/search/lexical.rs:15-85` only queries `memory_records_fts JOIN memory_records`, and candidate rows live exclusively in `truth_ontology_candidates` with no FTS wiring.

### Human Verification Required

None. Phase 03 behavior is storage- and service-layer only, and the shipped code paths are covered by automated migration, repository, governance, retrieval, and clippy checks.

### Gaps Summary

No gaps found. Phase 03 achieves the roadmap goal: truth layers are modeled explicitly, T3 audit state is preserved, T3 promotion is governed by explicit review gates, T2-to-T1 evolution is candidate-only, Phase 2 lexical retrieval compatibility remains intact, and no premature Rig, working-memory, semantic-retrieval, or rumination implementation was introduced.

---

_Verified: 2026-04-15T15:16:49Z_  
_Verifier: Claude (gsd-verifier)_
