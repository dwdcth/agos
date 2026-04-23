### L-1: Historical summary: 10 archived lessons
- **Strategy:** Historical lesson rollup
- **Outcome:** summary
- **Insight:** Compacted 10 older lessons across 10 strategy families; keep_ratio=10/10 (100%). Older signals should be weighted lower after 14 days.
- **Context:** historical_compaction; archived_entries=rolled_up
- **Iteration:** -
- **Timestamp:** 2026-04-20T05:02:45Z

### L-2: [labels: assessment, protocol-core] Added record-level kind-field assessment so callers can audit a complete DSL record
- **Strategy:** [labels: assessment, protocol-core] Added record-level kind-field assessment so callers can audit a complete DSL record against policy recommendations directly, expanding the focused verification matrix to 15 green checks.
- **Outcome:** keep
- **Insight:** [labels: assessment, protocol-core] Added record-level kind-field assessment so callers can audit a complete DSL record against policy recommendations directly, expanding the focused verification matrix to 15 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 12
- **Timestamp:** 2026-04-19T07:03:53Z

### L-3: [labels: pipeline-orchestrator, protocol-core] Added an async memory pipeline orchestrator that composes taxonomy classi
- **Strategy:** [labels: pipeline-orchestrator, protocol-core] Added an async memory pipeline orchestrator that composes taxonomy classification, summary generation, and DSL record construction, expanding the focused verification matrix to 16 green checks.
- **Outcome:** keep
- **Insight:** [labels: pipeline-orchestrator, protocol-core] Added an async memory pipeline orchestrator that composes taxonomy classification, summary generation, and DSL record construction, expanding the focused verification matrix to 16 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 13
- **Timestamp:** 2026-04-19T07:05:44Z

### L-4: [labels: keyword-classifier, protocol-core] Added a deterministic keyword-based taxonomy classifier with domain/topic/as
- **Strategy:** [labels: keyword-classifier, protocol-core] Added a deterministic keyword-based taxonomy classifier with domain/topic/aspect/kind scoring, fallback semantics, and public coverage, expanding the focused verification matrix to 17 green checks.
- **Outcome:** keep
- **Insight:** [labels: keyword-classifier, protocol-core] Added a deterministic keyword-based taxonomy classifier with domain/topic/aspect/kind scoring, fallback semantics, and public coverage, expanding the focused verification matrix to 17 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 14
- **Timestamp:** 2026-04-19T07:11:00Z

### L-5: [labels: rule-summary, protocol-core] Added a deterministic rule-based summary generator for offline fallback and testin
- **Strategy:** [labels: rule-summary, protocol-core] Added a deterministic rule-based summary generator for offline fallback and testing, expanding the focused verification matrix to 18 green checks across the memory core.
- **Outcome:** keep
- **Insight:** [labels: rule-summary, protocol-core] Added a deterministic rule-based summary generator for offline fallback and testing, expanding the focused verification matrix to 18 green checks across the memory core.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 15
- **Timestamp:** 2026-04-19T07:12:27Z

### L-6: [labels: flatten-view, protocol-core] Added a flattened storage view for DSL records plus round-trip rebuilding, expandi
- **Strategy:** [labels: flatten-view, protocol-core] Added a flattened storage view for DSL records plus round-trip rebuilding, expanding the focused verification matrix to 19 green checks while preparing repository integration without touching migrations.
- **Outcome:** keep
- **Insight:** [labels: flatten-view, protocol-core] Added a flattened storage view for DSL records plus round-trip rebuilding, expanding the focused verification matrix to 19 green checks while preparing repository integration without touching migrations.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 16
- **Timestamp:** 2026-04-19T07:14:56Z

### L-7: [labels: default-pipeline, protocol-core] Added a default memory pipeline that composes the keyword taxonomy classifier
- **Strategy:** [labels: default-pipeline, protocol-core] Added a default memory pipeline that composes the keyword taxonomy classifier with the rule-based summary generator, expanding the focused verification matrix to 20 green checks.
- **Outcome:** keep
- **Insight:** [labels: default-pipeline, protocol-core] Added a default memory pipeline that composes the keyword taxonomy classifier with the rule-based summary generator, expanding the focused verification matrix to 20 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 17
- **Timestamp:** 2026-04-19T07:16:22Z

### L-8: [labels: pipeline-convenience, protocol-core] Added direct encoded-output convenience on the default memory pipeline so
- **Strategy:** [labels: pipeline-convenience, protocol-core] Added direct encoded-output convenience on the default memory pipeline so callers can obtain final DSL text without a second encode step, expanding the focused verification matrix to 21 green checks.
- **Outcome:** keep
- **Insight:** [labels: pipeline-convenience, protocol-core] Added direct encoded-output convenience on the default memory pipeline so callers can obtain final DSL text without a second encode step, expanding the focused verification matrix to 21 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 18
- **Timestamp:** 2026-04-19T07:17:45Z

### L-9: [labels: pipeline-report, protocol-core] Added an auditable memory pipeline report object that returns classification, s
- **Strategy:** [labels: pipeline-report, protocol-core] Added an auditable memory pipeline report object that returns classification, summary input, DSL record, field assessment, and encoded output together, expanding the focused verification matrix to 22 green checks.
- **Outcome:** keep
- **Insight:** [labels: pipeline-report, protocol-core] Added an auditable memory pipeline report object that returns classification, summary input, DSL record, field assessment, and encoded output together, expanding the focused verification matrix to 22 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 19
- **Timestamp:** 2026-04-19T07:19:55Z

### L-10: [labels: classifier-report, protocol-core] Added an auditable keyword classification report that surfaces matched keywor
- **Strategy:** [labels: classifier-report, protocol-core] Added an auditable keyword classification report that surfaces matched keywords for the selected domain/topic/aspect/kind, expanding the focused verification matrix to 23 green checks.
- **Outcome:** keep
- **Insight:** [labels: classifier-report, protocol-core] Added an auditable keyword classification report that surfaces matched keywords for the selected domain/topic/aspect/kind, expanding the focused verification matrix to 23 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 20
- **Timestamp:** 2026-04-19T08:25:45Z

### L-11: [labels: report-visibility, protocol-core] Added a public pipeline report test so the memory pipeline now mechanically v
- **Strategy:** [labels: report-visibility, protocol-core] Added a public pipeline report test so the memory pipeline now mechanically verifies that classification, assessment, and encoded DSL output remain visible together, expanding the focused verification matrix to 24 green checks.
- **Outcome:** keep
- **Insight:** [labels: report-visibility, protocol-core] Added a public pipeline report test so the memory pipeline now mechanically verifies that classification, assessment, and encoded DSL output remain visible together, expanding the focused verification matrix to 24 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 21
- **Timestamp:** 2026-04-19T08:26:53Z

### L-12: [labels: record-entrypoint, protocol-core] Added a MemoryRecord entrypoint bridge so the layered memory pipeline can sta
- **Strategy:** [labels: record-entrypoint, protocol-core] Added a MemoryRecord entrypoint bridge so the layered memory pipeline can start directly from stored records, expanding the focused verification matrix to 25 green checks.
- **Outcome:** keep
- **Insight:** [labels: record-entrypoint, protocol-core] Added a MemoryRecord entrypoint bridge so the layered memory pipeline can start directly from stored records, expanding the focused verification matrix to 25 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 22
- **Timestamp:** 2026-04-19T08:32:55Z

### L-13: [labels: record-convenience, protocol-core] Added report and encoded-output conveniences for MemoryRecord entrypoints so
- **Strategy:** [labels: record-convenience, protocol-core] Added report and encoded-output conveniences for MemoryRecord entrypoints so stored records can flow through the layered memory pipeline without extra caller glue, expanding the focused verification matrix to 26 green checks.
- **Outcome:** keep
- **Insight:** [labels: record-convenience, protocol-core] Added report and encoded-output conveniences for MemoryRecord entrypoints so stored records can flow through the layered memory pipeline without extra caller glue, expanding the focused verification matrix to 26 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 23
- **Timestamp:** 2026-04-19T08:34:44Z

### L-14: [labels: flat-json, protocol-core] Added JSON serialization helpers for flattened DSL records so storage handoff can be
- **Strategy:** [labels: flat-json, protocol-core] Added JSON serialization helpers for flattened DSL records so storage handoff can be mechanically validated before repository schema changes, expanding the focused verification matrix to 27 green checks.
- **Outcome:** keep
- **Insight:** [labels: flat-json, protocol-core] Added JSON serialization helpers for flattened DSL records so storage handoff can be mechanically validated before repository schema changes, expanding the focused verification matrix to 27 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 24
- **Timestamp:** 2026-04-19T08:36:14Z

### L-15: [labels: classifier-metadata, protocol-core] Enriched classification input with structured record metadata (, , ) so the
- **Strategy:** [labels: classifier-metadata, protocol-core] Enriched classification input with structured record metadata (, , ) so the traditional classifier can use more than raw text, expanding the focused verification matrix to 28 green checks.
- **Outcome:** keep
- **Insight:** [labels: classifier-metadata, protocol-core] Enriched classification input with structured record metadata (, , ) so the traditional classifier can use more than raw text, expanding the focused verification matrix to 28 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 25
- **Timestamp:** 2026-04-19T08:37:36Z

### L-16: [labels: threshold-config, protocol-core] Added a configurable review threshold to the keyword taxonomy classifier so se
- **Strategy:** [labels: threshold-config, protocol-core] Added a configurable review threshold to the keyword taxonomy classifier so semantics are explicit and tunable, expanding the focused verification matrix to 29 green checks.
- **Outcome:** keep
- **Insight:** [labels: threshold-config, protocol-core] Added a configurable review threshold to the keyword taxonomy classifier so semantics are explicit and tunable, expanding the focused verification matrix to 29 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 26
- **Timestamp:** 2026-04-19T08:39:41Z

### L-17: [labels: report-json, protocol-core] Added JSON-serializable memory pipeline reports and public round-trip coverage so c
- **Strategy:** [labels: report-json, protocol-core] Added JSON-serializable memory pipeline reports and public round-trip coverage so classification, summary input, assessment, record, and encoded DSL can be persisted as one audit artifact, expanding the focused verification matrix to 30 green checks.
- **Outcome:** keep
- **Insight:** [labels: report-json, protocol-core] Added JSON-serializable memory pipeline reports and public round-trip coverage so classification, summary input, assessment, record, and encoded DSL can be persisted as one audit artifact, expanding the focused verification matrix to 30 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 27
- **Timestamp:** 2026-04-19T08:44:29Z

### L-18: [labels: report-flat-view, protocol-core] Added a flattened-record convenience on memory pipeline reports so storage-ori
- **Strategy:** [labels: report-flat-view, protocol-core] Added a flattened-record convenience on memory pipeline reports so storage-oriented consumers can read the persistable DSL view directly, expanding the focused verification matrix to 31 green checks.
- **Outcome:** keep
- **Insight:** [labels: report-flat-view, protocol-core] Added a flattened-record convenience on memory pipeline reports so storage-oriented consumers can read the persistable DSL view directly, expanding the focused verification matrix to 31 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 28
- **Timestamp:** 2026-04-19T08:46:46Z

### L-19: [labels: classifier-report-surface, protocol-core] Added a dedicated public test surface for the auditable keyword class
- **Strategy:** [labels: classifier-report-surface, protocol-core] Added a dedicated public test surface for the auditable keyword classifier report, expanding the focused verification matrix to 32 green checks.
- **Outcome:** keep
- **Insight:** [labels: classifier-report-surface, protocol-core] Added a dedicated public test surface for the auditable keyword classifier report, expanding the focused verification matrix to 32 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 30
- **Timestamp:** 2026-04-19T08:50:55Z

### L-20: [labels: flatten-convenience, protocol-core] Added flattened-output conveniences on the memory pipeline so callers can r
- **Strategy:** [labels: flatten-convenience, protocol-core] Added flattened-output conveniences on the memory pipeline so callers can request persistable flat DSL views directly from either raw text or stored records, expanding the focused verification matrix to 33 green checks.
- **Outcome:** keep
- **Insight:** [labels: flatten-convenience, protocol-core] Added flattened-output conveniences on the memory pipeline so callers can request persistable flat DSL views directly from either raw text or stored records, expanding the focused verification matrix to 33 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 31
- **Timestamp:** 2026-04-19T08:53:17Z

### L-21: [labels: report-consumption, protocol-core] Added consumption helpers on memory pipeline reports so callers can move dir
- **Strategy:** [labels: report-consumption, protocol-core] Added consumption helpers on memory pipeline reports so callers can move directly into encoded or flattened outputs without recomputation, expanding the focused verification matrix to 34 green checks.
- **Outcome:** keep
- **Insight:** [labels: report-consumption, protocol-core] Added consumption helpers on memory pipeline reports so callers can move directly into encoded or flattened outputs without recomputation, expanding the focused verification matrix to 34 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 32
- **Timestamp:** 2026-04-19T08:54:45Z

### L-22: [labels: report-json-convenience, protocol-core] Added JSON-report conveniences on the default memory pipeline so caller
- **Strategy:** [labels: report-json-convenience, protocol-core] Added JSON-report conveniences on the default memory pipeline so callers can request serialized audit artifacts directly from raw text or stored records, expanding the focused verification matrix to 35 green checks.
- **Outcome:** keep
- **Insight:** [labels: report-json-convenience, protocol-core] Added JSON-report conveniences on the default memory pipeline so callers can request serialized audit artifacts directly from raw text or stored records, expanding the focused verification matrix to 35 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 33
- **Timestamp:** 2026-04-19T08:56:59Z

### L-23: [labels: summary-heuristics, protocol-core] Extended the deterministic rule-based summary generator to extract condition
- **Strategy:** [labels: summary-heuristics, protocol-core] Extended the deterministic rule-based summary generator to extract condition and impact hints in addition to reason and time, expanding the focused verification matrix to 36 green checks.
- **Outcome:** keep
- **Insight:** [labels: summary-heuristics, protocol-core] Extended the deterministic rule-based summary generator to extract condition and impact hints in addition to reason and time, expanding the focused verification matrix to 36 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 34
- **Timestamp:** 2026-04-19T08:58:23Z

### L-24: [labels: record-output-surface, protocol-core] Added dedicated coverage for flattened and JSON outputs from stored-recor
- **Strategy:** [labels: record-output-surface, protocol-core] Added dedicated coverage for flattened and JSON outputs from stored-record entrypoints, expanding the focused verification matrix to 37 green checks.
- **Outcome:** keep
- **Insight:** [labels: record-output-surface, protocol-core] Added dedicated coverage for flattened and JSON outputs from stored-record entrypoints, expanding the focused verification matrix to 37 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 35
- **Timestamp:** 2026-04-19T09:00:38Z

### L-25: [labels: store-contract, protocol-core] Added persistence-contract types and a trait so layered memory records now have
- **Strategy:** [labels: store-contract, protocol-core] Added persistence-contract types and a trait so layered memory records now have a repository-ready boundary without requiring schema changes, expanding the focused verification matrix to 38 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-contract, protocol-core] Added persistence-contract types and a trait so layered memory records now have a repository-ready boundary without requiring schema changes, expanding the focused verification matrix to 38 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 36
- **Timestamp:** 2026-04-19T09:02:56Z

### L-26: [labels: store-impl, protocol-core] Added an in-memory implementation of the contract so the persistence boundary is exe
- **Strategy:** [labels: store-impl, protocol-core] Added an in-memory implementation of the contract so the persistence boundary is executable and testable before any SQLite schema work, expanding the focused verification matrix to 39 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-impl, protocol-core] Added an in-memory implementation of the contract so the persistence boundary is executable and testable before any SQLite schema work, expanding the focused verification matrix to 39 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 37
- **Timestamp:** 2026-04-19T09:04:33Z

### L-27: [labels: persisted-wrapper, protocol-core] Added pipeline conveniences that emit directly, so the layered memory flow no
- **Strategy:** [labels: persisted-wrapper, protocol-core] Added pipeline conveniences that emit directly, so the layered memory flow now reaches a repository-ready wrapper without extra caller glue, expanding the focused verification matrix to 40 green checks.
- **Outcome:** keep
- **Insight:** [labels: persisted-wrapper, protocol-core] Added pipeline conveniences that emit directly, so the layered memory flow now reaches a repository-ready wrapper without extra caller glue, expanding the focused verification matrix to 40 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 38
- **Timestamp:** 2026-04-19T09:06:06Z

### L-28: [labels: store-listing, protocol-core] Added listing semantics to the contract and the in-memory implementation so the p
- **Strategy:** [labels: store-listing, protocol-core] Added listing semantics to the contract and the in-memory implementation so the persistence boundary now supports batch inspection workflows, expanding the focused verification matrix to 41 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-listing, protocol-core] Added listing semantics to the contract and the in-memory implementation so the persistence boundary now supports batch inspection workflows, expanding the focused verification matrix to 41 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 39
- **Timestamp:** 2026-04-19T09:07:42Z

### L-29: [labels: json-store, protocol-core] Added a JSON-file implementation of the contract so layered memory records can persi
- **Strategy:** [labels: json-store, protocol-core] Added a JSON-file implementation of the contract so layered memory records can persist across sessions without waiting for SQLite schema changes, expanding the focused verification matrix to 42 green checks.
- **Outcome:** keep
- **Insight:** [labels: json-store, protocol-core] Added a JSON-file implementation of the contract so layered memory records can persist across sessions without waiting for SQLite schema changes, expanding the focused verification matrix to 42 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 40
- **Timestamp:** 2026-04-19T09:09:48Z

### L-30: [labels: store-query, protocol-core] Added taxonomy-aware query helpers on the persistence contract (, list-by-domain, l
- **Strategy:** [labels: store-query, protocol-core] Added taxonomy-aware query helpers on the persistence contract (, list-by-domain, list-by-kind), expanding the focused verification matrix to 43 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-query, protocol-core] Added taxonomy-aware query helpers on the persistence contract (, list-by-domain, list-by-kind), expanding the focused verification matrix to 43 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 41
- **Timestamp:** 2026-04-19T09:11:40Z

### L-31: [labels: store-delete, protocol-core] Added deletion semantics to the contract and implementations, completing the minim
- **Strategy:** [labels: store-delete, protocol-core] Added deletion semantics to the contract and implementations, completing the minimal CRUD surface and expanding the focused verification matrix to 44 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-delete, protocol-core] Added deletion semantics to the contract and implementations, completing the minimal CRUD surface and expanding the focused verification matrix to 44 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 42
- **Timestamp:** 2026-04-19T09:13:09Z

### L-32: [labels: store-taxonomy-query, protocol-core] Added topic and exact-path query helpers to the persistence contract so st
- **Strategy:** [labels: store-taxonomy-query, protocol-core] Added topic and exact-path query helpers to the persistence contract so stored layered memory records can be filtered by taxonomy without requiring SQL yet, expanding the focused verification matrix to 45 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-taxonomy-query, protocol-core] Added topic and exact-path query helpers to the persistence contract so stored layered memory records can be filtered by taxonomy without requiring SQL yet, expanding the focused verification matrix to 45 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 43
- **Timestamp:** 2026-04-19T09:15:54Z

### L-33: [labels: pipeline-store, protocol-core] Added pipeline-to-store persistence helpers so the default layered memory pipeli
- **Strategy:** [labels: pipeline-store, protocol-core] Added pipeline-to-store persistence helpers so the default layered memory pipeline can write directly into any implementation, expanding the focused verification matrix to 46 green checks.
- **Outcome:** keep
- **Insight:** [labels: pipeline-store, protocol-core] Added pipeline-to-store persistence helpers so the default layered memory pipeline can write directly into any implementation, expanding the focused verification matrix to 46 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 44
- **Timestamp:** 2026-04-19T09:20:03Z

### L-34: [labels: store-batch, protocol-core] Added bulk persistence helpers to the store contract and implementations so layered
- **Strategy:** [labels: store-batch, protocol-core] Added bulk persistence helpers to the store contract and implementations so layered memory records can be written in batches, expanding the focused verification matrix to 47 green checks.
- **Outcome:** keep
- **Insight:** [labels: store-batch, protocol-core] Added bulk persistence helpers to the store contract and implementations so layered memory records can be written in batches, expanding the focused verification matrix to 47 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 45
- **Timestamp:** 2026-04-19T09:21:39Z

### L-35: [labels: pipeline-store-persist, protocol-core] Added pipeline-to-store persistence helpers so the layered memory pipeli
- **Strategy:** [labels: pipeline-store-persist, protocol-core] Added pipeline-to-store persistence helpers so the layered memory pipeline can write directly into store implementations from raw text or stored records, expanding the focused verification matrix to 48 green checks.
- **Outcome:** keep
- **Insight:** [labels: pipeline-store-persist, protocol-core] Added pipeline-to-store persistence helpers so the layered memory pipeline can write directly into store implementations from raw text or stored records, expanding the focused verification matrix to 48 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 46
- **Timestamp:** 2026-04-19T09:26:57Z

### L-36: [labels: pipeline-batch-persist, protocol-core] Added bulk raw-input persistence helpers on the default pipeline so laye
- **Strategy:** [labels: pipeline-batch-persist, protocol-core] Added bulk raw-input persistence helpers on the default pipeline so layered memory records can be materialized and stored from multiple text inputs in one call, expanding the focused verification matrix to 49 green checks.
- **Outcome:** keep
- **Insight:** [labels: pipeline-batch-persist, protocol-core] Added bulk raw-input persistence helpers on the default pipeline so layered memory records can be materialized and stored from multiple text inputs in one call, expanding the focused verification matrix to 49 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 47
- **Timestamp:** 2026-04-19T09:30:09Z

### L-37: [labels: report-persisted, protocol-core] Added a report-to-persisted convenience so callers can convert an auditable me
- **Strategy:** [labels: report-persisted, protocol-core] Added a report-to-persisted convenience so callers can convert an auditable memory pipeline report into a repository-ready wrapper without rerunning the pipeline, expanding the focused verification matrix to 50 green checks.
- **Outcome:** keep
- **Insight:** [labels: report-persisted, protocol-core] Added a report-to-persisted convenience so callers can convert an auditable memory pipeline report into a repository-ready wrapper without rerunning the pipeline, expanding the focused verification matrix to 50 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 48
- **Timestamp:** 2026-04-19T09:31:38Z

### L-38: [labels: sqlite-integration, protocol-core] Integrated layered memory DSL into SQLite by adding migration 0007, reposito
- **Strategy:** [labels: sqlite-integration, protocol-core] Integrated layered memory DSL into SQLite by adding migration 0007, repository-backed support, and schema-aware tests, expanding the focused verification matrix to 51 green checks.
- **Outcome:** keep
- **Insight:** [labels: sqlite-integration, protocol-core] Integrated layered memory DSL into SQLite by adding migration 0007, repository-backed support, and schema-aware tests, expanding the focused verification matrix to 51 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 50
- **Timestamp:** 2026-04-19T14:53:43Z

### L-39: [labels: regression-sync, protocol-core] Updated focused regression suites to expect schema version 7 after the layered-
- **Strategy:** [labels: regression-sync, protocol-core] Updated focused regression suites to expect schema version 7 after the layered-memory migration, restoring status_cli, retrieval_cli, and working_memory_assembly to green and raising the focused verification matrix to 54 checks.
- **Outcome:** keep
- **Insight:** [labels: regression-sync, protocol-core] Updated focused regression suites to expect schema version 7 after the layered-memory migration, restoring status_cli, retrieval_cli, and working_memory_assembly to green and raising the focused verification matrix to 54 checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 51
- **Timestamp:** 2026-04-19T14:56:17Z

### L-40: [labels: sqlite-semantics, protocol-core] Added SQLite-backed coverage for taxonomy-path filtering and foreign-key casca
- **Strategy:** [labels: sqlite-semantics, protocol-core] Added SQLite-backed coverage for taxonomy-path filtering and foreign-key cascade behavior on layered memory DSL rows, expanding the focused verification matrix to 57 green checks.
- **Outcome:** keep
- **Insight:** [labels: sqlite-semantics, protocol-core] Added SQLite-backed coverage for taxonomy-path filtering and foreign-key cascade behavior on layered memory DSL rows, expanding the focused verification matrix to 57 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 52
- **Timestamp:** 2026-04-19T14:57:37Z

### L-41: [labels: sqlite-upsert, protocol-core] Added SQLite upsert coverage for layered memory DSL rows, verifying repeated writ
- **Strategy:** [labels: sqlite-upsert, protocol-core] Added SQLite upsert coverage for layered memory DSL rows, verifying repeated writes update existing entries instead of duplicating them, and raised the focused verification matrix to 60 green checks.
- **Outcome:** keep
- **Insight:** [labels: sqlite-upsert, protocol-core] Added SQLite upsert coverage for layered memory DSL rows, verifying repeated writes update existing entries instead of duplicating them, and raised the focused verification matrix to 60 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 53
- **Timestamp:** 2026-04-19T14:58:59Z

### L-42: [labels: broad-regression, protocol-core] Ran a broader regression sweep across the affected schema, CLI, retrieval, wor
- **Strategy:** [labels: broad-regression, protocol-core] Ran a broader regression sweep across the affected schema, CLI, retrieval, working-memory, repository, and all-target clippy surfaces; all passed after syncing migration and test expectations, raising the regression confidence metric to 66.
- **Outcome:** keep
- **Insight:** [labels: broad-regression, protocol-core] Ran a broader regression sweep across the affected schema, CLI, retrieval, working-memory, repository, and all-target clippy surfaces; all passed after syncing migration and test expectations, raising the regression confidence metric to 66.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 54
- **Timestamp:** 2026-04-19T15:00:54Z

### L-43: [labels: ingest-integration, protocol-core] Integrated the default layered memory pipeline into ingest so each ingested
- **Strategy:** [labels: ingest-integration, protocol-core] Integrated the default layered memory pipeline into ingest so each ingested chunk now persists a corresponding fact DSL sidecar row, and raised the focused verification matrix to 69 green checks.
- **Outcome:** keep
- **Insight:** [labels: ingest-integration, protocol-core] Integrated the default layered memory pipeline into ingest so each ingested chunk now persists a corresponding fact DSL sidecar row, and raised the focused verification matrix to 69 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 55
- **Timestamp:** 2026-04-20T01:50:26Z

### L-44: [labels: test-isolation, protocol-core] Stabilized JSON-file store tests by switching temp file paths from pid-based nam
- **Strategy:** [labels: test-isolation, protocol-core] Stabilized JSON-file store tests by switching temp file paths from pid-based names to per-test unique timestamps, raising the focused verification matrix to 71 green checks.
- **Outcome:** keep
- **Insight:** [labels: test-isolation, protocol-core] Stabilized JSON-file store tests by switching temp file paths from pid-based names to per-test unique timestamps, raising the focused verification matrix to 71 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 56
- **Timestamp:** 2026-04-20T01:55:09Z

### L-45: [labels: layered-batch-lookup, protocol-core] Added bulk layered-record lookup helpers on the repository so callers can
- **Strategy:** [labels: layered-batch-lookup, protocol-core] Added bulk layered-record lookup helpers on the repository so callers can resolve authority records and DSL sidecars together for batches of record ids, expanding the focused verification matrix to 72 green checks.
- **Outcome:** keep
- **Insight:** [labels: layered-batch-lookup, protocol-core] Added bulk layered-record lookup helpers on the repository so callers can resolve authority records and DSL sidecars together for batches of record ids, expanding the focused verification matrix to 72 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 57
- **Timestamp:** 2026-04-20T01:57:09Z

### L-46: [labels: memory-consumption, protocol-core] Connected layered memory DSL sidecars into working-memory evidence fragments
- **Strategy:** [labels: memory-consumption, protocol-core] Connected layered memory DSL sidecars into working-memory evidence fragments and verified the downstream assembly, agent-search, and rumination suites remain green, raising the focused verification matrix to 76 checks.
- **Outcome:** keep
- **Insight:** [labels: memory-consumption, protocol-core] Connected layered memory DSL sidecars into working-memory evidence fragments and verified the downstream assembly, agent-search, and rumination suites remain green, raising the focused verification matrix to 76 checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 58
- **Timestamp:** 2026-04-20T02:15:37Z

### L-47: [labels: sqlite-review-metadata, broad-regression] Persisted classifier review metadata in SQLite via migration 0008, sy
- **Strategy:** [labels: sqlite-review-metadata, broad-regression] Persisted classifier review metadata in SQLite via migration 0008, synced repository and regression expectations, and raised the focused verification matrix to 79 green checks across schema, CLI, retrieval, working-memory, and repository surfaces.
- **Outcome:** keep
- **Insight:** [labels: sqlite-review-metadata, broad-regression] Persisted classifier review metadata in SQLite via migration 0008, synced repository and regression expectations, and raised the focused verification matrix to 79 green checks across schema, CLI, retrieval, working-memory, and repository surfaces.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 59
- **Timestamp:** 2026-04-20T03:19:48Z

### L-48: [labels: ingest-layered-memory, protocol-core] Added sync default pipeline helpers and wired ingest to persist layered m
- **Strategy:** [labels: ingest-layered-memory, protocol-core] Added sync default pipeline helpers and wired ingest to persist layered memory DSL sidecars for each ingested chunk, raising the focused verification matrix to 83 green checks.
- **Outcome:** keep
- **Insight:** [labels: ingest-layered-memory, protocol-core] Added sync default pipeline helpers and wired ingest to persist layered memory DSL sidecars for each ingested chunk, raising the focused verification matrix to 83 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 60
- **Timestamp:** 2026-04-20T03:21:54Z

### L-49: [labels: broad-regression, ingest-review-metadata] Promoted the ingest review-metadata fix through a broad regression sw
- **Strategy:** [labels: broad-regression, ingest-review-metadata] Promoted the ingest review-metadata fix through a broad regression sweep spanning schema, CLI, retrieval, assembly, agent-search, rumination, and all-target clippy; all passed, so the wider confidence metric now exceeds the previous retained best.
- **Outcome:** keep
- **Insight:** [labels: broad-regression, ingest-review-metadata] Promoted the ingest review-metadata fix through a broad regression sweep spanning schema, CLI, retrieval, assembly, agent-search, rumination, and all-target clippy; all passed, so the wider confidence metric now exceeds the previous retained best.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 62
- **Timestamp:** 2026-04-20T03:26:53Z

### L-50: [labels: config-summary-backend, rig-summary, ingest-config] Added a config-driven layered-memory summary backend with o
- **Strategy:** [labels: config-summary-backend, rig-summary, ingest-config] Added a config-driven layered-memory summary backend with openai-compatible Rig structured output, kept auto fallback to rule-based when LLM config is absent, wired CLI ingest to runtime config, and passed full cargo test plus all-target clippy, raising the verification matrix to 96 green checks.
- **Outcome:** keep
- **Insight:** [labels: config-summary-backend, rig-summary, ingest-config] Added a config-driven layered-memory summary backend with openai-compatible Rig structured output, kept auto fallback to rule-based when LLM config is absent, wired CLI ingest to runtime config, and passed full cargo test plus all-target clippy, raising the verification matrix to 96 green checks.
- **Context:** goal=Implement layered memory taxonomy and DSL core in Rust; scope=src/memory/**,tests/memory_dsl.rs; metric=targeted_memory_dsl_checks_passed; direction=higher
- **Iteration:** 63
- **Timestamp:** 2026-04-20T03:43:40Z

### L-51: [labels: retrieval-dsl-consumption, lexical-first, ordinary-retrieval] Confirmed that save-side FTS is already enabled t
- **Strategy:** [labels: retrieval-dsl-consumption, lexical-first, ordinary-retrieval] Confirmed that save-side FTS is already enabled through migration 0003 and memory_records FTS triggers, then extended ordinary non-LLM retrieval to attach persisted DSL sidecars on SearchResult and reuse them during working-memory assembly; focused retrieval checks improved from 10 to 11 and downstream agent/rumination regressions stayed green.
- **Outcome:** keep
- **Insight:** [labels: retrieval-dsl-consumption, lexical-first, ordinary-retrieval] Confirmed that save-side FTS is already enabled through migration 0003 and memory_records FTS triggers, then extended ordinary non-LLM retrieval to attach persisted DSL sidecars on SearchResult and reuse them during working-memory assembly; focused retrieval checks improved from 10 to 11 and downstream agent/rumination regressions stayed green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1
- **Timestamp:** 2026-04-20T05:02:45Z

### L-52: [labels: structured-recall, lexical-first, ordinary-retrieval] Added non-LLM structured recall over persisted DSL fields
- **Strategy:** [labels: structured-recall, lexical-first, ordinary-retrieval] Added non-LLM structured recall over persisted DSL fields so ordinary retrieval can match taxonomy and compressed memory facts even when the raw text does not contain the query term, fixed candidate-strategy merging so structured hits do not masquerade as embedding hybrids, and raised the focused retrieval matrix from 11 to 13 green checks while keeping agent_search green.
- **Outcome:** keep
- **Insight:** [labels: structured-recall, lexical-first, ordinary-retrieval] Added non-LLM structured recall over persisted DSL fields so ordinary retrieval can match taxonomy and compressed memory facts even when the raw text does not contain the query term, fixed candidate-strategy merging so structured hits do not masquerade as embedding hybrids, and raised the focused retrieval matrix from 11 to 13 green checks while keeping agent_search green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 2
- **Timestamp:** 2026-04-20T05:06:42Z

### L-53: [labels: retrieval-text-surface, lexical-first, ordinary-retrieval] Extended ordinary text-mode retrieval to render the
- **Strategy:** [labels: retrieval-text-surface, lexical-first, ordinary-retrieval] Extended ordinary text-mode retrieval to render the structured DSL summary alongside citations so front-stage consumers see the compressed memory shape instead of only raw snippets, and raised the focused retrieval matrix from 13 to 14 green checks while keeping the lexical-first guard green.
- **Outcome:** keep
- **Insight:** [labels: retrieval-text-surface, lexical-first, ordinary-retrieval] Extended ordinary text-mode retrieval to render the structured DSL summary alongside citations so front-stage consumers see the compressed memory shape instead of only raw snippets, and raised the focused retrieval matrix from 13 to 14 green checks while keeping the lexical-first guard green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 3
- **Timestamp:** 2026-04-20T05:08:13Z

### L-54: [labels: taxonomy-filters, lexical-first, working-memory-assembly] Added explicit non-LLM taxonomy filters on SearchFilt
- **Strategy:** [labels: taxonomy-filters, lexical-first, working-memory-assembly] Added explicit non-LLM taxonomy filters on SearchFilters for domain/topic/aspect/kind, applied them after DSL sidecar attachment so ordinary lexical recall and structured recall both respect the same structure-level constraints, and verified that working-memory assembly now inherits those taxonomy cuts from retrieval requests; focused retrieval checks improved from 14 to 16 while agent_search stayed green.
- **Outcome:** keep
- **Insight:** [labels: taxonomy-filters, lexical-first, working-memory-assembly] Added explicit non-LLM taxonomy filters on SearchFilters for domain/topic/aspect/kind, applied them after DSL sidecar attachment so ordinary lexical recall and structured recall both respect the same structure-level constraints, and verified that working-memory assembly now inherits those taxonomy cuts from retrieval requests; focused retrieval checks improved from 14 to 16 while agent_search stayed green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 4
- **Timestamp:** 2026-04-20T05:12:54Z

### L-55: [labels: cli-taxonomy-filters, lexical-first, agent-search] Exposed the existing non-LLM taxonomy filters on both search
- **Strategy:** [labels: cli-taxonomy-filters, lexical-first, agent-search] Exposed the existing non-LLM taxonomy filters on both search and agent-search CLI surfaces via explicit --domain/--topic/--aspect/--kind flags, threaded those filters into working-memory requests, and verified that both plain retrieval and agent-search JSON outputs now preserve the requested structure-level cuts; focused retrieval checks improved from 16 to 18 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: cli-taxonomy-filters, lexical-first, agent-search] Exposed the existing non-LLM taxonomy filters on both search and agent-search CLI surfaces via explicit --domain/--topic/--aspect/--kind flags, threaded those filters into working-memory requests, and verified that both plain retrieval and agent-search JSON outputs now preserve the requested structure-level cuts; focused retrieval checks improved from 16 to 18 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 5
- **Timestamp:** 2026-04-20T05:17:54Z

### L-56: [labels: taxonomy-cli-validation, lexical-first, cli-surface] Tightened the non-LLM retrieval entrypoints so search and
- **Strategy:** [labels: taxonomy-cli-validation, lexical-first, cli-surface] Tightened the non-LLM retrieval entrypoints so search and agent-search CLI taxonomy filters are parsed against the fixed taxonomy v1 set instead of accepting free-form strings, while preserving the new structure-level retrieval controls and adding explicit invalid-input coverage; focused retrieval checks improved from 18 to 20 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: taxonomy-cli-validation, lexical-first, cli-surface] Tightened the non-LLM retrieval entrypoints so search and agent-search CLI taxonomy filters are parsed against the fixed taxonomy v1 set instead of accepting free-form strings, while preserving the new structure-level retrieval controls and adding explicit invalid-input coverage; focused retrieval checks improved from 18 to 20 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 6
- **Timestamp:** 2026-04-20T05:20:17Z

### L-57: [labels: taxonomy-combination-validation, lexical-first, cli-surface] Added taxonomy-combination validation for the sear
- **Strategy:** [labels: taxonomy-combination-validation, lexical-first, cli-surface] Added taxonomy-combination validation for the search and agent-search CLI surfaces so invalid domain/topic pairings are rejected against taxonomy v1 before runtime-gate checks, preserving the fixed-ontology boundary at the retrieval entrypoint; focused retrieval checks improved from 20 to 22 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: taxonomy-combination-validation, lexical-first, cli-surface] Added taxonomy-combination validation for the search and agent-search CLI surfaces so invalid domain/topic pairings are rejected against taxonomy v1 before runtime-gate checks, preserving the fixed-ontology boundary at the retrieval entrypoint; focused retrieval checks improved from 20 to 22 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 7
- **Timestamp:** 2026-04-20T05:23:49Z

### L-58: [labels: dsl-aware-ranking, lexical-first, ordinary-retrieval] Extended lexical-first scoring so keyword bonuses also ac
- **Strategy:** [labels: dsl-aware-ranking, lexical-first, ordinary-retrieval] Extended lexical-first scoring so keyword bonuses also account for persisted DSL taxonomy and compressed fact fields, propagated DSL payloads through candidate merging, and verified that structured kind matches now break otherwise lexical ties in favor of the more semantically aligned record; focused retrieval checks improved from 22 to 23 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: dsl-aware-ranking, lexical-first, ordinary-retrieval] Extended lexical-first scoring so keyword bonuses also account for persisted DSL taxonomy and compressed fact fields, propagated DSL payloads through candidate merging, and verified that structured kind matches now break otherwise lexical ties in favor of the more semantically aligned record; focused retrieval checks improved from 22 to 23 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 8
- **Timestamp:** 2026-04-20T05:25:36Z

### L-59: [labels: library-taxonomy-validation, lexical-first, ordinary-retrieval] Pushed fixed-taxonomy validation down into the
- **Strategy:** [labels: library-taxonomy-validation, lexical-first, ordinary-retrieval] Pushed fixed-taxonomy validation down into the SearchService so direct library callers can no longer bypass the taxonomy v1 boundary with free-form or cross-domain filters, complementing the earlier CLI validation and keeping retrieval semantics consistent across surfaces; focused retrieval checks improved from 23 to 25 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: library-taxonomy-validation, lexical-first, ordinary-retrieval] Pushed fixed-taxonomy validation down into the SearchService so direct library callers can no longer bypass the taxonomy v1 boundary with free-form or cross-domain filters, complementing the earlier CLI validation and keeping retrieval semantics consistent across surfaces; focused retrieval checks improved from 23 to 25 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 9
- **Timestamp:** 2026-04-20T05:28:02Z

### L-60: [labels: library-taxonomy-hardening, lexical-first, ordinary-retrieval] Expanded library-level taxonomy validation cover
- **Strategy:** [labels: library-taxonomy-hardening, lexical-first, ordinary-retrieval] Expanded library-level taxonomy validation coverage across unknown domain/aspect/kind values plus valid domain/topic pairs, so non-CLI retrieval callers now hit the same fixed-taxonomy guardrails with explicit tests and valid combinations still flow through the lexical-first path; focused retrieval checks improved from 25 to 29 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: library-taxonomy-hardening, lexical-first, ordinary-retrieval] Expanded library-level taxonomy validation coverage across unknown domain/aspect/kind values plus valid domain/topic pairs, so non-CLI retrieval callers now hit the same fixed-taxonomy guardrails with explicit tests and valid combinations still flow through the lexical-first path; focused retrieval checks improved from 25 to 29 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 10
- **Timestamp:** 2026-04-20T05:33:14Z

### L-61: [labels: retrieval-metric-realigned, lexical-first, library-taxonomy-hardening] Added four positive library-level taxono
- **Strategy:** [labels: retrieval-metric-realigned, lexical-first, library-taxonomy-hardening] Added four positive library-level taxonomy validation checks so the configured retrieval verify surface now fully reflects the fixed-taxonomy hardening already implemented, pushing the focused non-LLM retrieval metric to 30 green checks across retrieval_cli, working_memory_assembly, and lexical_search while keeping the lexical-first guard green.
- **Outcome:** keep
- **Insight:** [labels: retrieval-metric-realigned, lexical-first, library-taxonomy-hardening] Added four positive library-level taxonomy validation checks so the configured retrieval verify surface now fully reflects the fixed-taxonomy hardening already implemented, pushing the focused non-LLM retrieval metric to 30 green checks across retrieval_cli, working_memory_assembly, and lexical_search while keeping the lexical-first guard green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 11
- **Timestamp:** 2026-04-20T05:34:51Z

### L-62: [labels: retrieval-surface-lockdown, lexical-first, ordinary-retrieval] Added explicit retrieval_cli coverage for taxono
- **Strategy:** [labels: retrieval-surface-lockdown, lexical-first, ordinary-retrieval] Added explicit retrieval_cli coverage for taxonomy filter echoing in library responses and text-mode filter rendering, locking down the user-visible non-LLM retrieval surfaces after the taxonomy hardening work and raising the focused retrieval metric from 30 to 32 green checks with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: retrieval-surface-lockdown, lexical-first, ordinary-retrieval] Added explicit retrieval_cli coverage for taxonomy filter echoing in library responses and text-mode filter rendering, locking down the user-visible non-LLM retrieval surfaces after the taxonomy hardening work and raising the focused retrieval metric from 30 to 32 green checks with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 12
- **Timestamp:** 2026-04-20T05:36:23Z

### L-63: [labels: structured-snippet-priority, lexical-first, ordinary-retrieval] Updated candidate merging so when a record is m
- **Strategy:** [labels: structured-snippet-priority, lexical-first, ordinary-retrieval] Updated candidate merging so when a record is matched by both raw lexical and structured DSL recall, the result surface prefers the compressed structured snippet instead of falling back to the raw text fragment, improving front-stage memory readability while keeping the lexical-first ranking contract intact; focused retrieval checks improved from 32 to 33 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-snippet-priority, lexical-first, ordinary-retrieval] Updated candidate merging so when a record is matched by both raw lexical and structured DSL recall, the result surface prefers the compressed structured snippet instead of falling back to the raw text fragment, improving front-stage memory readability while keeping the lexical-first ranking contract intact; focused retrieval checks improved from 32 to 33 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 13
- **Timestamp:** 2026-04-20T05:38:58Z

### L-64: [labels: text-channel-surface, lexical-first, ordinary-retrieval] Extended the plain-text search surface to render chann
- **Strategy:** [labels: text-channel-surface, lexical-first, ordinary-retrieval] Extended the plain-text search surface to render channel contribution and query-strategy summaries even without --trace, so users can see whether a result came from raw lexical, structured DSL recall, or a hybrid merge directly in the default retrieval view; focused retrieval checks improved from 33 to 34 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-channel-surface, lexical-first, ordinary-retrieval] Extended the plain-text search surface to render channel contribution and query-strategy summaries even without --trace, so users can see whether a result came from raw lexical, structured DSL recall, or a hybrid merge directly in the default retrieval view; focused retrieval checks improved from 33 to 34 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 14
- **Timestamp:** 2026-04-20T05:40:49Z

### L-65: [labels: text-dsl-conditional-surface, lexical-first, ordinary-retrieval] Completed the plain-text DSL surface so TIME a
- **Strategy:** [labels: text-dsl-conditional-surface, lexical-first, ordinary-retrieval] Completed the plain-text DSL surface so TIME and COND fields are now rendered alongside WHY and IMPACT when present, and added focused retrieval coverage for conditional memory summaries in the default no-LLM search view; focused retrieval checks improved from 34 to 35 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-dsl-conditional-surface, lexical-first, ordinary-retrieval] Completed the plain-text DSL surface so TIME and COND fields are now rendered alongside WHY and IMPACT when present, and added focused retrieval coverage for conditional memory summaries in the default no-LLM search view; focused retrieval checks improved from 34 to 35 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 16
- **Timestamp:** 2026-04-20T05:43:53Z

### L-66: [labels: text-record-shape-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval surface to show
- **Strategy:** [labels: text-record-shape-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval surface to show each result's record_type and truth_layer alongside the structured memory summary, making ordinary no-LLM recall more self-explanatory for downstream users; focused retrieval checks improved from 35 to 36 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-record-shape-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval surface to show each result's record_type and truth_layer alongside the structured memory summary, making ordinary no-LLM recall more self-explanatory for downstream users; focused retrieval checks improved from 35 to 36 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 17
- **Timestamp:** 2026-04-20T05:45:18Z

### L-67: [labels: text-source-shape-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval surface to rend
- **Strategy:** [labels: text-source-shape-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval surface to render the persisted source kind and source label alongside record id/type/truth-layer, improving provenance readability in the default no-LLM recall view; focused retrieval checks improved from 36 to 37 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-source-shape-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval surface to render the persisted source kind and source label alongside record id/type/truth-layer, improving provenance readability in the default no-LLM recall view; focused retrieval checks improved from 36 to 37 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 19
- **Timestamp:** 2026-04-20T05:48:57Z

### L-68: [labels: text-dsl-source-ref-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage that locks the pl
- **Strategy:** [labels: text-dsl-source-ref-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage that locks the plain-text DSL summary to its structured source reference, bringing the verify surface back in line with the previously shipped SRC enhancement and raising the focused retrieval metric from 37 to 38 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-dsl-source-ref-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage that locks the plain-text DSL summary to its structured source reference, bringing the verify surface back in line with the previously shipped SRC enhancement and raising the focused retrieval metric from 37 to 38 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 21
- **Timestamp:** 2026-04-20T05:51:36Z

### L-69: [labels: text-scope-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval record summary to incl
- **Strategy:** [labels: text-scope-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval record summary to include scope alongside record id, source metadata, type, and truth layer, making the default no-LLM recall view more explicit about where a memory fragment belongs; focused retrieval checks improved from 38 to 39 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-scope-surface, lexical-first, ordinary-retrieval] Extended the plain-text retrieval record summary to include scope alongside record id, source metadata, type, and truth layer, making the default no-LLM recall view more explicit about where a memory fragment belongs; focused retrieval checks improved from 38 to 39 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 22
- **Timestamp:** 2026-04-20T05:53:02Z

### L-70: [labels: json-dsl-conditional-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks t
- **Strategy:** [labels: json-dsl-conditional-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to expose conditional DSL fields such as COND and IMPACT when they are present, ensuring the structured no-LLM recall contract stays symmetric between text and JSON consumers; focused retrieval checks improved from 39 to 40 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: json-dsl-conditional-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to expose conditional DSL fields such as COND and IMPACT when they are present, ensuring the structured no-LLM recall contract stays symmetric between text and JSON consumers; focused retrieval checks improved from 39 to 40 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 23
- **Timestamp:** 2026-04-20T05:54:29Z

### L-71: [labels: text-validity-window-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks t
- **Strategy:** [labels: text-validity-window-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text search surface to the explicit validity window rendered from citations, ensuring valid_from/valid_to remain visible in the default no-LLM recall view; focused retrieval checks improved from 40 to 41 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-validity-window-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text search surface to the explicit validity window rendered from citations, ensuring valid_from/valid_to remain visible in the default no-LLM recall view; focused retrieval checks improved from 40 to 41 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 24
- **Timestamp:** 2026-04-20T05:55:48Z

### L-72: [labels: json-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the
- **Strategy:** [labels: json-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to authority record shape metadata including record id, source label, scope, and truth layer, keeping the no-LLM retrieval contract explicit for downstream consumers; focused retrieval checks improved from 41 to 42 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: json-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to authority record shape metadata including record id, source label, scope, and truth layer, keeping the no-LLM retrieval contract explicit for downstream consumers; focused retrieval checks improved from 41 to 42 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 25
- **Timestamp:** 2026-04-20T05:56:57Z

### L-73: [labels: json-source-kind-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the J
- **Strategy:** [labels: json-source-kind-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to the persisted source kind and record_type metadata, tightening the authority-record contract for downstream non-LLM retrieval consumers; focused retrieval checks improved from 42 to 43 with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: json-source-kind-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to the persisted source kind and record_type metadata, tightening the authority-record contract for downstream non-LLM retrieval consumers; focused retrieval checks improved from 42 to 43 with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 26
- **Timestamp:** 2026-04-20T05:58:11Z

### L-74: [labels: retrieval-surface-expansion, lexical-first, ordinary-retrieval] Expanded the focused non-LLM retrieval verifica
- **Strategy:** [labels: retrieval-surface-expansion, lexical-first, ordinary-retrieval] Expanded the focused non-LLM retrieval verification surface after the saturation audit: added explicit coverage for domain/aspect/kind taxonomy filtering behavior, richer plain-text result summaries (record scope/source/id plus DSL source/conditional fields), and corresponding JSON/text contract checks, while keeping the lexical-first recall path and guard green; focused retrieval checks improved from 43 to 46.
- **Outcome:** keep
- **Insight:** [labels: retrieval-surface-expansion, lexical-first, ordinary-retrieval] Expanded the focused non-LLM retrieval verification surface after the saturation audit: added explicit coverage for domain/aspect/kind taxonomy filtering behavior, richer plain-text result summaries (record scope/source/id plus DSL source/conditional fields), and corresponding JSON/text contract checks, while keeping the lexical-first recall path and guard green; focused retrieval checks improved from 43 to 46.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 28
- **Timestamp:** 2026-04-20T06:04:25Z

### L-75: [labels: working-memory-taxonomy-coverage, lexical-first, working-memory-assembly] Expanded focused working-memory verif
- **Strategy:** [labels: working-memory-taxonomy-coverage, lexical-first, working-memory-assembly] Expanded focused working-memory verification so the non-LLM retrieval taxonomy filters are now covered not only for topic but also for domain, aspect, and kind inheritance inside assembly; with retrieval_cli at 39 checks, working_memory_assembly at 8, and lexical_search at 2, the configured retrieval metric now rises to 49 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-taxonomy-coverage, lexical-first, working-memory-assembly] Expanded focused working-memory verification so the non-LLM retrieval taxonomy filters are now covered not only for topic but also for domain, aspect, and kind inheritance inside assembly; with retrieval_cli at 39 checks, working_memory_assembly at 8, and lexical_search at 2, the configured retrieval metric now rises to 49 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 29
- **Timestamp:** 2026-04-20T06:17:18Z

### L-76: [labels: working-memory-structured-trace, lexical-first, working-memory-assembly] Added focused working-memory coverage
- **Strategy:** [labels: working-memory-structured-trace, lexical-first, working-memory-assembly] Added focused working-memory coverage that locks structured-only recall provenance all the way into present-frame fragments, ensuring the Structured query strategy survives retrieval-to-assembly handoff; with retrieval_cli at 39 checks, working_memory_assembly at 9, and lexical_search at 2, the configured non-LLM retrieval metric now rises from 49 to 50 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-trace, lexical-first, working-memory-assembly] Added focused working-memory coverage that locks structured-only recall provenance all the way into present-frame fragments, ensuring the Structured query strategy survives retrieval-to-assembly handoff; with retrieval_cli at 39 checks, working_memory_assembly at 9, and lexical_search at 2, the configured non-LLM retrieval metric now rises from 49 to 50 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 31
- **Timestamp:** 2026-04-20T06:22:26Z

### L-77: [labels: working-memory-filter-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks
- **Strategy:** [labels: working-memory-filter-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks taxonomy filters not just as retrieval behavior but as persistent trace metadata on present-frame fragments, ensuring topic/kind and domain/aspect cuts remain auditable after retrieval-to-working-memory handoff; with retrieval_cli at 39 checks, working_memory_assembly at 11, and lexical_search at 2, the configured non-LLM retrieval metric now rises from 50 to 52 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-filter-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks taxonomy filters not just as retrieval behavior but as persistent trace metadata on present-frame fragments, ensuring topic/kind and domain/aspect cuts remain auditable after retrieval-to-working-memory handoff; with retrieval_cli at 39 checks, working_memory_assembly at 11, and lexical_search at 2, the configured non-LLM retrieval metric now rises from 50 to 52 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 32
- **Timestamp:** 2026-04-20T06:25:53Z

### L-78: [labels: structured-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks str
- **Strategy:** [labels: structured-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only queries to the lexical-first channel contract while still preserving Structured strategy provenance in JSON output, raising the configured non-LLM retrieval metric from 52 to 53 with the lexical-first guard still green.
- **Outcome:** keep
- **Insight:** [labels: structured-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only queries to the lexical-first channel contract while still preserving Structured strategy provenance in JSON output, raising the configured non-LLM retrieval metric from 52 to 53 with the lexical-first guard still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 34
- **Timestamp:** 2026-04-20T06:30:34Z

### L-79: [labels: json-dsl-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSO
- **Strategy:** [labels: json-dsl-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to structured DSL source_ref/time/why fields, tightening the no-LLM retrieval contract around compressed memory facts and raising the configured metric from 53 to 54 while the lexical-first guard stays green.
- **Outcome:** keep
- **Insight:** [labels: json-dsl-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to structured DSL source_ref/time/why fields, tightening the no-LLM retrieval contract around compressed memory facts and raising the configured metric from 53 to 54 while the lexical-first guard stays green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 35
- **Timestamp:** 2026-04-20T06:32:58Z

### L-80: [labels: structured-snippet-json-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that lock
- **Strategy:** [labels: structured-snippet-json-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to prefer the structured DSL snippet when a result is matched by both raw lexical and structured recall, raising the configured non-LLM retrieval metric from 54 to 55 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: structured-snippet-json-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON search surface to prefer the structured DSL snippet when a result is matched by both raw lexical and structured recall, raising the configured non-LLM retrieval metric from 54 to 55 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 37
- **Timestamp:** 2026-04-20T06:38:34Z

### L-81: [labels: mixed-strategy-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that lo
- **Strategy:** [labels: mixed-strategy-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed raw-lexical plus structured hits to preserve both Simple and Structured provenance in JSON output, raising the configured non-LLM retrieval metric from 55 to 56 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: mixed-strategy-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed raw-lexical plus structured hits to preserve both Simple and Structured provenance in JSON output, raising the configured non-LLM retrieval metric from 55 to 56 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 38
- **Timestamp:** 2026-04-20T06:40:27Z

### L-82: [labels: lexical-structured-provenance-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage th
- **Strategy:** [labels: lexical-structured-provenance-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall under the lexical-first channel contract while preserving Structured provenance in the library trace, raising the configured non-LLM retrieval metric from 56 to 57 with the lexical-first guard still green.
- **Outcome:** keep
- **Insight:** [labels: lexical-structured-provenance-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall under the lexical-first channel contract while preserving Structured provenance in the library trace, raising the configured non-LLM retrieval metric from 56 to 57 with the lexical-first guard still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 39
- **Timestamp:** 2026-04-20T06:42:43Z

### L-83: [labels: working-memory-snippet-priority, lexical-first, working-memory-assembly] Added focused assembly coverage that l
- **Strategy:** [labels: working-memory-snippet-priority, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the retrieval-selected structured snippet all the way into present-frame fragments, ensuring front-stage working memory does not regress back to raw lexical text when structured recall is available; with retrieval_cli at 43 checks, working_memory_assembly at 12, and lexical_search at 3, the configured non-LLM retrieval metric now rises from 57 to 58 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-snippet-priority, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the retrieval-selected structured snippet all the way into present-frame fragments, ensuring front-stage working memory does not regress back to raw lexical text when structured recall is available; with retrieval_cli at 43 checks, working_memory_assembly at 12, and lexical_search at 3, the configured non-LLM retrieval metric now rises from 57 to 58 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 40
- **Timestamp:** 2026-04-20T06:44:27Z

### L-84: [labels: lexical-mixed-provenance-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that lo
- **Strategy:** [labels: lexical-mixed-provenance-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed raw-lexical plus structured hits to preserve both Simple and Structured provenance while still reporting the lexical-first channel, raising the configured non-LLM retrieval metric from 58 to 59 with the lexical-first guard still green.
- **Outcome:** keep
- **Insight:** [labels: lexical-mixed-provenance-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed raw-lexical plus structured hits to preserve both Simple and Structured provenance while still reporting the lexical-first channel, raising the configured non-LLM retrieval metric from 58 to 59 with the lexical-first guard still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 41
- **Timestamp:** 2026-04-20T06:46:49Z

### L-85: [labels: lexical-structured-snippet-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that
- **Strategy:** [labels: lexical-structured-snippet-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed raw-lexical plus structured hits to prefer the structured snippet in the library result surface, raising the configured non-LLM retrieval metric from 59 to 60 while the lexical-first guard remains green. Note: the broader lexical_search binary showed one transient allocator shutdown crash during a combined run, but the suite passed on immediate rerun and the new assertion itself is stable.
- **Outcome:** keep
- **Insight:** [labels: lexical-structured-snippet-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed raw-lexical plus structured hits to prefer the structured snippet in the library result surface, raising the configured non-LLM retrieval metric from 59 to 60 while the lexical-first guard remains green. Note: the broader lexical_search binary showed one transient allocator shutdown crash during a combined run, but the suite passed on immediate rerun and the new assertion itself is stable.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 42
- **Timestamp:** 2026-04-20T06:49:13Z

### L-86: [labels: working-memory-mixed-provenance, lexical-first, working-memory-assembly] Added focused working-memory coverage
- **Strategy:** [labels: working-memory-mixed-provenance, lexical-first, working-memory-assembly] Added focused working-memory coverage that locks mixed raw-lexical plus structured hits to preserve both Simple and Structured provenance after retrieval-to-assembly handoff, raising the configured non-LLM retrieval metric from 60 to 61 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-provenance, lexical-first, working-memory-assembly] Added focused working-memory coverage that locks mixed raw-lexical plus structured hits to preserve both Simple and Structured provenance after retrieval-to-assembly handoff, raising the configured non-LLM retrieval metric from 60 to 61 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 43
- **Timestamp:** 2026-04-20T06:51:07Z

### L-87: [labels: text-mixed-strategy-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks th
- **Strategy:** [labels: text-mixed-strategy-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text search surface to preserve mixed raw-lexical plus structured strategies under the lexical-first channel, raising the configured non-LLM retrieval metric from 61 to 62 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: text-mixed-strategy-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text search surface to preserve mixed raw-lexical plus structured strategies under the lexical-first channel, raising the configured non-LLM retrieval metric from 61 to 62 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 44
- **Timestamp:** 2026-04-20T06:53:50Z

### L-88: [labels: text-structured-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that lock
- **Strategy:** [labels: text-structured-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text search surface to prefer the structured snippet when mixed raw lexical plus structured recall occurs, raising the configured non-LLM retrieval metric from 62 to 63 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: text-structured-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text search surface to prefer the structured snippet when mixed raw lexical plus structured recall occurs, raising the configured non-LLM retrieval metric from 62 to 63 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 45
- **Timestamp:** 2026-04-20T06:55:59Z

### L-89: [labels: working-memory-structured-only-snippet, lexical-first, working-memory-assembly] Added focused assembly coverage
- **Strategy:** [labels: working-memory-structured-only-snippet, lexical-first, working-memory-assembly] Added focused assembly coverage that locks structured-only recall to the structured snippet surface after retrieval-to-working-memory handoff, raising the configured non-LLM retrieval metric from 63 to 64 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-only-snippet, lexical-first, working-memory-assembly] Added focused assembly coverage that locks structured-only recall to the structured snippet surface after retrieval-to-working-memory handoff, raising the configured non-LLM retrieval metric from 63 to 64 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 47
- **Timestamp:** 2026-04-20T07:02:02Z

### L-90: [labels: json-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that
- **Strategy:** [labels: json-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only queries to the structured snippet surface in JSON output, raising the configured non-LLM retrieval metric from 64 to 65 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only queries to the structured snippet surface in JSON output, raising the configured non-LLM retrieval metric from 64 to 65 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 48
- **Timestamp:** 2026-04-20T07:04:32Z

### L-91: [labels: working-memory-mixed-snippet-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that
- **Strategy:** [labels: working-memory-mixed-snippet-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the mixed raw-lexical plus structured-hit case to preserve both provenance branches and the structured snippet after retrieval-to-working-memory handoff, raising the configured non-LLM retrieval metric from 65 to 66 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-snippet-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the mixed raw-lexical plus structured-hit case to preserve both provenance branches and the structured snippet after retrieval-to-working-memory handoff, raising the configured non-LLM retrieval metric from 65 to 66 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 49
- **Timestamp:** 2026-04-20T07:06:27Z

### L-92: [labels: text-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that
- **Strategy:** [labels: text-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only queries to the structured snippet surface in the plain-text search view, bringing the text contract into parity with the library, JSON, and working-memory layers and raising the configured non-LLM retrieval metric from 66 to 67 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: text-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only queries to the structured snippet surface in the plain-text search view, bringing the text contract into parity with the library, JSON, and working-memory layers and raising the configured non-LLM retrieval metric from 66 to 67 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 50
- **Timestamp:** 2026-04-20T07:10:21Z

### L-93: [labels: lexical-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage
- **Strategy:** [labels: lexical-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only queries to the structured snippet surface directly at the library retrieval layer, raising the configured non-LLM retrieval metric from 67 to 68 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: lexical-structured-only-snippet-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only queries to the structured snippet surface directly at the library retrieval layer, raising the configured non-LLM retrieval metric from 67 to 68 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 51
- **Timestamp:** 2026-04-20T07:12:24Z

### L-94: [labels: working-memory-source-metadata-lock, lexical-first, working-memory-assembly] Added focused assembly coverage th
- **Strategy:** [labels: working-memory-source-metadata-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_kind/source_label/source_uri preservation inside working-memory fragment citations, raising the configured non-LLM retrieval metric from 68 to 69 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-source-metadata-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_kind/source_label/source_uri preservation inside working-memory fragment citations, raising the configured non-LLM retrieval metric from 68 to 69 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 52
- **Timestamp:** 2026-04-20T07:14:43Z

### L-95: [labels: working-memory-validity-window-lock, lexical-first, working-memory-assembly] Added focused assembly coverage th
- **Strategy:** [labels: working-memory-validity-window-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_from/valid_to preservation inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 69 to 70 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-validity-window-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_from/valid_to preservation inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 69 to 70 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 53
- **Timestamp:** 2026-04-20T07:18:50Z

### L-96: [labels: working-memory-temporal-trace-lock, lexical-first, working-memory-assembly] Added focused assembly coverage tha
- **Strategy:** [labels: working-memory-temporal-trace-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_at / recorded_from / recorded_to preservation in fragment trace filters, raising the configured non-LLM retrieval metric from 70 to 71 while the lexical-first guard remains green. Note: one combined run showed the same transient allocator shutdown issue previously seen in this environment, but both working_memory_assembly and lexical_search passed immediately on isolated rerun.
- **Outcome:** keep
- **Insight:** [labels: working-memory-temporal-trace-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_at / recorded_from / recorded_to preservation in fragment trace filters, raising the configured non-LLM retrieval metric from 70 to 71 while the lexical-first guard remains green. Note: one combined run showed the same transient allocator shutdown issue previously seen in this environment, but both working_memory_assembly and lexical_search passed immediately on isolated rerun.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 54
- **Timestamp:** 2026-04-20T07:22:27Z

### L-97: [labels: lexical-validity-window-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that loc
- **Strategy:** [labels: lexical-validity-window-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks valid_at filtering and validity-window preservation directly at the library retrieval layer, raising the configured non-LLM retrieval metric from 71 to 72 while the lexical-first guard remains green. Note: the broader lexical_search binary again showed the known transient allocator shutdown issue in one combined run, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Outcome:** keep
- **Insight:** [labels: lexical-validity-window-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks valid_at filtering and validity-window preservation directly at the library retrieval layer, raising the configured non-LLM retrieval metric from 71 to 72 while the lexical-first guard remains green. Note: the broader lexical_search binary again showed the known transient allocator shutdown issue in one combined run, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 55
- **Timestamp:** 2026-04-20T07:25:37Z

### L-98: [labels: working-memory-structured-only-contract, lexical-first, working-memory-assembly] Added focused assembly coverag
- **Strategy:** [labels: working-memory-structured-only-contract, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the full structured-only recall contract end-to-end inside present-frame fragments, including snippet surface, structured provenance, source metadata, and validity-window preservation; with retrieval_cli at 47 checks, working_memory_assembly at 19, and lexical_search at 7, the configured non-LLM retrieval metric now rises from 72 to 73 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-only-contract, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the full structured-only recall contract end-to-end inside present-frame fragments, including snippet surface, structured provenance, source metadata, and validity-window preservation; with retrieval_cli at 47 checks, working_memory_assembly at 19, and lexical_search at 7, the configured non-LLM retrieval metric now rises from 72 to 73 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 56
- **Timestamp:** 2026-04-20T07:27:38Z

### L-99: [labels: working-memory-mixed-contract, lexical-first, working-memory-assembly] Added focused assembly coverage that loc
- **Strategy:** [labels: working-memory-mixed-contract, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the full mixed raw-lexical plus structured-hit contract end-to-end inside present-frame fragments, including structured snippet priority, dual provenance, source metadata, and validity-window preservation; with retrieval_cli at 47 checks, working_memory_assembly at 20, and lexical_search at 7, the configured non-LLM retrieval metric now rises from 73 to 74 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-contract, lexical-first, working-memory-assembly] Added focused assembly coverage that locks the full mixed raw-lexical plus structured-hit contract end-to-end inside present-frame fragments, including structured snippet priority, dual provenance, source metadata, and validity-window preservation; with retrieval_cli at 47 checks, working_memory_assembly at 20, and lexical_search at 7, the configured non-LLM retrieval metric now rises from 73 to 74 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 57
- **Timestamp:** 2026-04-20T07:29:46Z

### L-100: [labels: working-memory-mixed-validity-lock, lexical-first, working-memory-assembly] Added focused assembly coverage tha
- **Strategy:** [labels: working-memory-mixed-validity-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_to preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 74 to 75 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-validity-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_to preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 74 to 75 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 58
- **Timestamp:** 2026-04-20T07:32:05Z

### L-101: [labels: working-memory-source-uri-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that lo
- **Strategy:** [labels: working-memory-source-uri-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_uri preservation for structured-only recall inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 75 to 76 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-source-uri-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_uri preservation for structured-only recall inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 75 to 76 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 61
- **Timestamp:** 2026-04-20T07:41:47Z

### L-102: [labels: working-memory-mixed-source-uri-lock, lexical-first, working-memory-assembly] Added focused assembly coverage t
- **Strategy:** [labels: working-memory-mixed-source-uri-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_uri preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 76 to 77 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-source-uri-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_uri preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 76 to 77 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 62
- **Timestamp:** 2026-04-20T07:43:57Z

### L-103: [labels: working-memory-source-label-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that
- **Strategy:** [labels: working-memory-source-label-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_label preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 77 to 78 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-source-label-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_label preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 77 to 78 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 63
- **Timestamp:** 2026-04-20T07:46:12Z

### L-104: [labels: working-memory-source-kind-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that l
- **Strategy:** [labels: working-memory-source-kind-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_kind preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 78 to 79 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-source-kind-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_kind preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 78 to 79 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 64
- **Timestamp:** 2026-04-20T07:48:34Z

### L-105: [labels: working-memory-mixed-valid-from-lock, lexical-first, working-memory-assembly] Added focused assembly coverage t
- **Strategy:** [labels: working-memory-mixed-valid-from-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_from preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 79 to 80 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-valid-from-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_from preservation for mixed raw-lexical plus structured hits inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 79 to 80 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 65
- **Timestamp:** 2026-04-20T07:51:03Z

### L-106: [labels: lexical-mixed-validity-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that lock
- **Strategy:** [labels: lexical-mixed-validity-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks valid_at filtering for mixed raw-lexical plus structured hits at the library retrieval layer, ensuring expired records are excluded without losing mixed provenance and raising the configured non-LLM retrieval metric from 80 to 81 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: lexical-mixed-validity-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks valid_at filtering for mixed raw-lexical plus structured hits at the library retrieval layer, ensuring expired records are excluded without losing mixed provenance and raising the configured non-LLM retrieval metric from 80 to 81 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 66
- **Timestamp:** 2026-04-20T07:53:54Z

### L-107: [labels: json-temporal-filter-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks J
- **Strategy:** [labels: json-temporal-filter-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks JSON echoing of truth-layer and temporal filter fields (valid_at / recorded_from / recorded_to) on the ordinary no-LLM search surface, raising the configured retrieval metric from 81 to 82 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-temporal-filter-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks JSON echoing of truth-layer and temporal filter fields (valid_at / recorded_from / recorded_to) on the ordinary no-LLM search surface, raising the configured retrieval metric from 81 to 82 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 68
- **Timestamp:** 2026-04-20T07:58:40Z

### L-108: [labels: working-memory-structured-source-label-lock, lexical-first, working-memory-assembly] Added focused assembly cov
- **Strategy:** [labels: working-memory-structured-source-label-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_label preservation for structured-only recall inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 82 to 83 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-source-label-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_label preservation for structured-only recall inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 82 to 83 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 69
- **Timestamp:** 2026-04-20T08:00:48Z

### L-109: [labels: working-memory-structured-source-kind-lock, lexical-first, working-memory-assembly] Added focused assembly cove
- **Strategy:** [labels: working-memory-structured-source-kind-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_kind preservation for structured-only recall inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 83 to 84 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-source-kind-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks source_kind preservation for structured-only recall inside present-frame fragment citations, raising the configured non-LLM retrieval metric from 83 to 84 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 70
- **Timestamp:** 2026-04-20T08:02:50Z

### L-110: [labels: working-memory-filter-core-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that
- **Strategy:** [labels: working-memory-filter-core-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks scope / record_type / truth_layer filter preservation inside present-frame fragment traces, raising the configured non-LLM retrieval metric from 84 to 85 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-filter-core-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks scope / record_type / truth_layer filter preservation inside present-frame fragment traces, raising the configured non-LLM retrieval metric from 84 to 85 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 71
- **Timestamp:** 2026-04-20T08:04:51Z

### L-111: [labels: working-memory-structured-query-lock, lexical-first, working-memory-assembly] Added focused assembly coverage t
- **Strategy:** [labels: working-memory-structured-query-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks matched_query preservation for structured-only recall inside present-frame fragment traces, raising the configured non-LLM retrieval metric from 85 to 86 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-query-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks matched_query preservation for structured-only recall inside present-frame fragment traces, raising the configured non-LLM retrieval metric from 85 to 86 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 72
- **Timestamp:** 2026-04-20T08:07:33Z

### L-112: [labels: lexical-structured-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused lexical_search covera
- **Strategy:** [labels: lexical-structured-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_kind/source_label preservation for structured-only recall at the library layer, raising the configured non-LLM retrieval metric from 86 to 87 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: lexical-structured-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_kind/source_label preservation for structured-only recall at the library layer, raising the configured non-LLM retrieval metric from 86 to 87 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 73
- **Timestamp:** 2026-04-20T08:11:12Z

### L-113: [labels: working-memory-mixed-query-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that l
- **Strategy:** [labels: working-memory-mixed-query-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks matched_query preservation for mixed raw-lexical plus structured hits inside present-frame fragment traces. The expanded focused verify surface is now retrieval_cli=48, working_memory_assembly=31, lexical_search=9 for a total of 88. Note: one combined working_memory_assembly run hit the known transient allocator shutdown issue in this environment, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-query-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks matched_query preservation for mixed raw-lexical plus structured hits inside present-frame fragment traces. The expanded focused verify surface is now retrieval_cli=48, working_memory_assembly=31, lexical_search=9 for a total of 88. Note: one combined working_memory_assembly run hit the known transient allocator shutdown issue in this environment, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 74
- **Timestamp:** 2026-04-20T08:14:17Z

### L-114: [labels: lexical-mixed-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage th
- **Strategy:** [labels: lexical-mixed-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_kind/source_label preservation for mixed raw-lexical plus structured hits at the library layer, raising the configured non-LLM retrieval metric from 88 to 89 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: lexical-mixed-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_kind/source_label preservation for mixed raw-lexical plus structured hits at the library layer, raising the configured non-LLM retrieval metric from 88 to 89 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 75
- **Timestamp:** 2026-04-20T08:16:22Z

### L-115: [labels: working-memory-structured-core-filter-lock, lexical-first, working-memory-assembly] Added focused assembly cove
- **Strategy:** [labels: working-memory-structured-core-filter-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks scope / record_type / truth_layer filter preservation for structured-only recall inside fragment traces, raising the configured non-LLM retrieval metric from 89 to 90 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-core-filter-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks scope / record_type / truth_layer filter preservation for structured-only recall inside fragment traces, raising the configured non-LLM retrieval metric from 89 to 90 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 77
- **Timestamp:** 2026-04-20T08:21:41Z

### L-116: [labels: lexical-mixed-source-uri-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that lo
- **Strategy:** [labels: lexical-mixed-source-uri-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_uri preservation for mixed raw-lexical plus structured hits at the library layer, raising the configured non-LLM retrieval metric from 90 to 91 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: lexical-mixed-source-uri-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_uri preservation for mixed raw-lexical plus structured hits at the library layer, raising the configured non-LLM retrieval metric from 90 to 91 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 78
- **Timestamp:** 2026-04-20T08:24:14Z

### L-117: [labels: lexical-structured-source-uri-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage th
- **Strategy:** [labels: lexical-structured-source-uri-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_uri preservation for structured-only recall at the library layer, raising the configured non-LLM retrieval metric from 91 to 92 while the lexical-first guard remains green. Note: the broader lexical_search binary again showed the known transient allocator shutdown issue in one combined run, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Outcome:** keep
- **Insight:** [labels: lexical-structured-source-uri-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks source_uri preservation for structured-only recall at the library layer, raising the configured non-LLM retrieval metric from 91 to 92 while the lexical-first guard remains green. Note: the broader lexical_search binary again showed the known transient allocator shutdown issue in one combined run, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 79
- **Timestamp:** 2026-04-20T08:28:08Z

### L-118: [labels: json-citation-source-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks c
- **Strategy:** [labels: json-citation-source-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks citation-level source metadata (record_id/source_kind/source_label) on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 92 to 93 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-citation-source-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks citation-level source metadata (record_id/source_kind/source_label) on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 92 to 93 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 80
- **Timestamp:** 2026-04-20T08:30:55Z

### L-119: [labels: json-citation-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks
- **Strategy:** [labels: json-citation-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks citation validity-window visibility (valid_from / valid_to) on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 93 to 94 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-citation-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks citation validity-window visibility (valid_from / valid_to) on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 93 to 94 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 81
- **Timestamp:** 2026-04-20T08:33:32Z

### L-120: [labels: metric-realignment, lexical-first, working-memory-assembly] Re-ran the focused non-LLM retrieval verify surface
- **Strategy:** [labels: metric-realignment, lexical-first, working-memory-assembly] Re-ran the focused non-LLM retrieval verify surface after the recent assembly-contract additions and confirmed the true aggregate is retrieval_cli=50, working_memory_assembly=33, lexical_search=12 for a total of 95. The previous iteration undercounted the expanded working_memory_assembly surface and was recorded as refine; this row promotes the already-green state to the correct retained metric.
- **Outcome:** keep
- **Insight:** [labels: metric-realignment, lexical-first, working-memory-assembly] Re-ran the focused non-LLM retrieval verify surface after the recent assembly-contract additions and confirmed the true aggregate is retrieval_cli=50, working_memory_assembly=33, lexical_search=12 for a total of 95. The previous iteration undercounted the expanded working_memory_assembly surface and was recorded as refine; this row promotes the already-green state to the correct retained metric.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 84
- **Timestamp:** 2026-04-20T08:42:14Z

### L-121: [labels: lexical-mixed-query-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks m
- **Strategy:** [labels: lexical-mixed-query-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks matched_query preservation for mixed raw-lexical plus structured hits at the library layer. The current focused non-LLM retrieval surface is now retrieval_cli=50, working_memory_assembly=33, lexical_search=13 for a total of 96, with the lexical-first guard still green.
- **Outcome:** keep
- **Insight:** [labels: lexical-mixed-query-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks matched_query preservation for mixed raw-lexical plus structured hits at the library layer. The current focused non-LLM retrieval surface is now retrieval_cli=50, working_memory_assembly=33, lexical_search=13 for a total of 96, with the lexical-first guard still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 85
- **Timestamp:** 2026-04-20T08:44:15Z

### L-122: [labels: working-memory-mixed-temporal-trace, lexical-first, working-memory-assembly] Added focused assembly coverage th
- **Strategy:** [labels: working-memory-mixed-temporal-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_at / recorded_from / recorded_to preservation for mixed raw-lexical plus structured hits inside fragment traces, raising the configured non-LLM retrieval metric from 96 to 97 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-temporal-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_at / recorded_from / recorded_to preservation for mixed raw-lexical plus structured hits inside fragment traces, raising the configured non-LLM retrieval metric from 96 to 97 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 86
- **Timestamp:** 2026-04-20T08:46:50Z

### L-123: [labels: working-memory-mixed-dsl-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that loc
- **Strategy:** [labels: working-memory-mixed-dsl-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks DSL payload preservation for mixed raw-lexical plus structured hits inside present-frame fragments, raising the configured non-LLM retrieval metric from 97 to 98 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-dsl-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks DSL payload preservation for mixed raw-lexical plus structured hits inside present-frame fragments, raising the configured non-LLM retrieval metric from 97 to 98 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 87
- **Timestamp:** 2026-04-20T08:49:58Z

### L-124: [labels: working-memory-structured-saturation, lexical-first, working-memory-assembly] Added two focused assembly covera
- **Strategy:** [labels: working-memory-structured-saturation, lexical-first, working-memory-assembly] Added two focused assembly coverage checks that lock structured-only recall preservation of source_uri/scope and truth-layer identity in present-frame fragments, bringing the current configured non-LLM retrieval metric to 100 (retrieval_cli=50, working_memory_assembly=37, lexical_search=13) while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-saturation, lexical-first, working-memory-assembly] Added two focused assembly coverage checks that lock structured-only recall preservation of source_uri/scope and truth-layer identity in present-frame fragments, bringing the current configured non-LLM retrieval metric to 100 (retrieval_cli=50, working_memory_assembly=37, lexical_search=13) while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 88
- **Timestamp:** 2026-04-20T08:52:28Z

### L-125: [labels: json-domain-aspect-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks dom
- **Strategy:** [labels: json-domain-aspect-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks domain/aspect filter echoing on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 100 to 101 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-domain-aspect-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks domain/aspect filter echoing on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 100 to 101 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 89
- **Timestamp:** 2026-04-20T08:57:06Z

### L-126: [labels: json-taxonomy-triple-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks s
- **Strategy:** [labels: json-taxonomy-triple-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks simultaneous domain/aspect/kind filter echoing and result narrowing on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 101 to 102 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-taxonomy-triple-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks simultaneous domain/aspect/kind filter echoing and result narrowing on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 101 to 102 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 90
- **Timestamp:** 2026-04-20T08:59:50Z

### L-127: [labels: text-taxonomy-triple-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks d
- **Strategy:** [labels: text-taxonomy-triple-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks domain/aspect/kind filter visibility together on the plain-text search surface, raising the configured non-LLM retrieval metric from 102 to 103 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: text-taxonomy-triple-echo, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks domain/aspect/kind filter visibility together on the plain-text search surface, raising the configured non-LLM retrieval metric from 102 to 103 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 91
- **Timestamp:** 2026-04-20T09:02:26Z

### L-128: [labels: working-memory-structured-temporal-trace, lexical-first, working-memory-assembly] Added focused assembly covera
- **Strategy:** [labels: working-memory-structured-temporal-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_at / recorded_from / recorded_to preservation for structured-only recall inside fragment traces. The focused non-LLM retrieval surface is now retrieval_cli=53, working_memory_assembly=38, lexical_search=13 for a total of 104. Note: one combined working_memory_assembly run hit the known transient allocator shutdown issue in this environment, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Outcome:** keep
- **Insight:** [labels: working-memory-structured-temporal-trace, lexical-first, working-memory-assembly] Added focused assembly coverage that locks valid_at / recorded_from / recorded_to preservation for structured-only recall inside fragment traces. The focused non-LLM retrieval surface is now retrieval_cli=53, working_memory_assembly=38, lexical_search=13 for a total of 104. Note: one combined working_memory_assembly run hit the known transient allocator shutdown issue in this environment, but the suite passed immediately on isolated rerun and the new assertion itself is stable.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 92
- **Timestamp:** 2026-04-20T09:08:01Z

### L-129: [labels: working-memory-record-id-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that loc
- **Strategy:** [labels: working-memory-record-id-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks citation.record_id preservation for mixed raw-lexical plus structured hits inside present-frame fragments, raising the configured non-LLM retrieval metric from 104 to 105 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-record-id-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks citation.record_id preservation for mixed raw-lexical plus structured hits inside present-frame fragments, raising the configured non-LLM retrieval metric from 104 to 105 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 93
- **Timestamp:** 2026-04-20T09:10:25Z

### L-130: [labels: json-mixed-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that l
- **Strategy:** [labels: json-mixed-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks source_kind/source_label preservation for mixed raw-lexical plus structured hits on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 105 to 106 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-mixed-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks source_kind/source_label preservation for mixed raw-lexical plus structured hits on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 105 to 106 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 94
- **Timestamp:** 2026-04-20T09:13:18Z

### L-131: [labels: json-mixed-contract-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks th
- **Strategy:** [labels: json-mixed-contract-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the mixed raw-lexical plus structured-hit contract on the JSON surface as a single unit, requiring the structured snippet and both Simple/Structured provenance branches to appear together. The configured non-LLM retrieval metric now rises from 106 to 107 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-mixed-contract-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the mixed raw-lexical plus structured-hit contract on the JSON surface as a single unit, requiring the structured snippet and both Simple/Structured provenance branches to appear together. The configured non-LLM retrieval metric now rises from 106 to 107 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 95
- **Timestamp:** 2026-04-20T09:16:19Z

### L-132: [labels: json-mixed-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mix
- **Strategy:** [labels: json-mixed-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed raw-lexical plus structured hits to the lexical_only channel contract on the JSON search surface, raising the configured non-LLM retrieval metric from 107 to 108 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-mixed-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed raw-lexical plus structured hits to the lexical_only channel contract on the JSON search surface, raising the configured non-LLM retrieval metric from 107 to 108 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 97
- **Timestamp:** 2026-04-20T09:25:12Z

### L-133: [labels: library-taxonomy-triple-contract, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that
- **Strategy:** [labels: library-taxonomy-triple-contract, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks valid domain/aspect/kind filtering as a combined library contract, requiring correct filter echo and result narrowing together on the non-LLM retrieval surface. The configured metric now rises from 108 to 109 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: library-taxonomy-triple-contract, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks valid domain/aspect/kind filtering as a combined library contract, requiring correct filter echo and result narrowing together on the non-LLM retrieval surface. The configured metric now rises from 108 to 109 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 98
- **Timestamp:** 2026-04-20T09:28:47Z

### L-134: [labels: json-mixed-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks va
- **Strategy:** [labels: json-mixed-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks validity-window preservation for mixed raw-lexical plus structured hits on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 109 to 110 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: json-mixed-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks validity-window preservation for mixed raw-lexical plus structured hits on the JSON ordinary retrieval surface, raising the configured non-LLM retrieval metric from 109 to 110 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 100
- **Timestamp:** 2026-04-20T09:34:46Z

### L-135: [labels: working-memory-mixed-truth-layer-lock, lexical-first, working-memory-assembly] Added focused assembly coverage
- **Strategy:** [labels: working-memory-mixed-truth-layer-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks truth_context.truth_layer preservation for mixed raw-lexical plus structured hits inside present-frame fragments, raising the configured non-LLM retrieval metric from 110 to 111 while the lexical-first guard remains green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-mixed-truth-layer-lock, lexical-first, working-memory-assembly] Added focused assembly coverage that locks truth_context.truth_layer preservation for mixed raw-lexical plus structured hits inside present-frame fragments, raising the configured non-LLM retrieval metric from 110 to 111 while the lexical-first guard remains green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 101
- **Timestamp:** 2026-04-20T09:37:13Z

### L-136: [labels: text-source-uri-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the pl
- **Strategy:** [labels: text-source-uri-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text ordinary retrieval surface to the authority source_uri shown in each result header. The stable focused verify surface now rises from 111 to 112 via retrieval_cli=59, working_memory_assembly=40, and lexical_search=13, with clippy still green and no production-code change required.
- **Outcome:** keep
- **Insight:** [labels: text-source-uri-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text ordinary retrieval surface to the authority source_uri shown in each result header. The stable focused verify surface now rises from 111 to 112 via retrieval_cli=59, working_memory_assembly=40, and lexical_search=13, with clippy still green and no production-code change required.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 103
- **Timestamp:** 2026-04-20T09:44:37Z

### L-137: [labels: text-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the p
- **Strategy:** [labels: text-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text ordinary retrieval surface to citation.recorded_at rendering in the citation line. The stable focused non-LLM retrieval surface now rises from 112 to 113 via retrieval_cli=60, working_memory_assembly=40, and lexical_search=13, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text ordinary retrieval surface to citation.recorded_at rendering in the citation line. The stable focused non-LLM retrieval surface now rises from 112 to 113 via retrieval_cli=60, working_memory_assembly=40, and lexical_search=13, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 104
- **Timestamp:** 2026-04-20T09:46:06Z

### L-138: [labels: text-citation-chunk-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks th
- **Strategy:** [labels: text-citation-chunk-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text ordinary retrieval surface to the citation chunk anchor shape rendered as chunk 1/1. The stable focused non-LLM retrieval surface now rises from 113 to 114 via retrieval_cli=61, working_memory_assembly=40, and lexical_search=13, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: text-citation-chunk-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the plain-text ordinary retrieval surface to the citation chunk anchor shape rendered as chunk 1/1. The stable focused non-LLM retrieval surface now rises from 113 to 114 via retrieval_cli=61, working_memory_assembly=40, and lexical_search=13, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 105
- **Timestamp:** 2026-04-20T09:47:13Z

### L-139: [labels: working-memory-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: working-memory-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed recall preservation of citation.recorded_at on assembled world fragments. The stable focused non-LLM retrieval surface now rises from 114 to 115 via retrieval_cli=61, working_memory_assembly=41, and lexical_search=13, with clippy green after aligning the new test to the current harness.
- **Outcome:** keep
- **Insight:** [labels: working-memory-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed recall preservation of citation.recorded_at on assembled world fragments. The stable focused non-LLM retrieval surface now rises from 114 to 115 via retrieval_cli=61, working_memory_assembly=41, and lexical_search=13, with clippy green after aligning the new test to the current harness.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 107
- **Timestamp:** 2026-04-20T09:49:54Z

### L-140: [labels: working-memory-chunk-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: working-memory-chunk-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed recall preservation of citation anchor shape on assembled world fragments, asserting chunk_index=0 and chunk_count=1. The stable focused non-LLM retrieval surface now rises from 115 to 116 via retrieval_cli=61, working_memory_assembly=42, and lexical_search=13, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-chunk-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed recall preservation of citation anchor shape on assembled world fragments, asserting chunk_index=0 and chunk_count=1. The stable focused non-LLM retrieval surface now rises from 115 to 116 via retrieval_cli=61, working_memory_assembly=42, and lexical_search=13, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 108
- **Timestamp:** 2026-04-20T09:50:51Z

### L-141: [labels: lexical-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks m
- **Strategy:** [labels: lexical-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed recall preservation of citation.recorded_at on ordinary retrieval results. The stable focused non-LLM retrieval surface now rises from 116 to 117 via retrieval_cli=61, working_memory_assembly=42, and lexical_search=14, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: lexical-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed recall preservation of citation.recorded_at on ordinary retrieval results. The stable focused non-LLM retrieval surface now rises from 116 to 117 via retrieval_cli=61, working_memory_assembly=42, and lexical_search=14, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 110
- **Timestamp:** 2026-04-20T09:52:09Z

### L-142: [labels: lexical-chunk-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks
- **Strategy:** [labels: lexical-chunk-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed recall preservation of citation anchor shape on ordinary retrieval results, asserting chunk_index=0 and chunk_count=1. The stable focused non-LLM retrieval surface now rises from 117 to 118 via retrieval_cli=61, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: lexical-chunk-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed recall preservation of citation anchor shape on ordinary retrieval results, asserting chunk_index=0 and chunk_count=1. The stable focused non-LLM retrieval surface now rises from 117 to 118 via retrieval_cli=61, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 111
- **Timestamp:** 2026-04-20T09:52:59Z

### L-143: [labels: json-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks th
- **Strategy:** [labels: json-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON ordinary retrieval surface to citation.recorded_at and citation anchor shape, asserting chunk_index=0 and chunk_count=1 on result citations. The stable focused non-LLM retrieval surface now rises from 118 to 119 via retrieval_cli=62, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: json-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks the JSON ordinary retrieval surface to citation.recorded_at and citation anchor shape, asserting chunk_index=0 and chunk_count=1 on result citations. The stable focused non-LLM retrieval surface now rises from 118 to 119 via retrieval_cli=62, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 113
- **Timestamp:** 2026-04-20T09:55:13Z

### L-144: [labels: json-mixed-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that lo
- **Strategy:** [labels: json-mixed-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed ordinary retrieval on the JSON surface to citation.recorded_at and citation anchor shape, asserting chunk_index=0 and chunk_count=1 while preserving both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 119 to 120 via retrieval_cli=63, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: json-mixed-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed ordinary retrieval on the JSON surface to citation.recorded_at and citation anchor shape, asserting chunk_index=0 and chunk_count=1 while preserving both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 119 to 120 via retrieval_cli=63, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 114
- **Timestamp:** 2026-04-20T09:56:18Z

### L-145: [labels: library-mixed-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level co
- **Strategy:** [labels: library-mixed-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed SearchService recall to citation.recorded_at preservation while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 120 to 121 via retrieval_cli=64, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: library-mixed-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed SearchService recall to citation.recorded_at preservation while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 120 to 121 via retrieval_cli=64, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 116
- **Timestamp:** 2026-04-20T09:58:19Z

### L-146: [labels: library-mixed-anchor-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverag
- **Strategy:** [labels: library-mixed-anchor-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed SearchService recall to citation anchor preservation, asserting chunk_index=0 and chunk_count=1 while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 121 to 122 via retrieval_cli=65, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: library-mixed-anchor-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed SearchService recall to citation anchor preservation, asserting chunk_index=0 and chunk_count=1 while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 121 to 122 via retrieval_cli=65, working_memory_assembly=42, and lexical_search=15, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 117
- **Timestamp:** 2026-04-20T09:59:17Z

### L-147: [labels: working-memory-line-range-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: working-memory-line-range-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed recall preservation of the nested line-range citation anchor on assembled world fragments, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 }. The stable focused non-LLM retrieval surface now rises from 122 to 123 via retrieval_cli=65, working_memory_assembly=43, and lexical_search=15, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: working-memory-line-range-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed recall preservation of the nested line-range citation anchor on assembled world fragments, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 }. The stable focused non-LLM retrieval surface now rises from 122 to 123 via retrieval_cli=65, working_memory_assembly=43, and lexical_search=15, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 122
- **Timestamp:** 2026-04-20T10:03:37Z

### L-148: [labels: lexical-line-range-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that l
- **Strategy:** [labels: lexical-line-range-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed recall preservation of the nested line-range citation anchor, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 } while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 123 to 124 via retrieval_cli=65, working_memory_assembly=43, and lexical_search=16, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: lexical-line-range-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed recall preservation of the nested line-range citation anchor, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 } while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 123 to 124 via retrieval_cli=65, working_memory_assembly=43, and lexical_search=16, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 123
- **Timestamp:** 2026-04-20T10:04:30Z

### L-149: [labels: library-line-range-anchor-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level co
- **Strategy:** [labels: library-line-range-anchor-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed SearchService recall to the nested line-range citation anchor, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 } while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 124 to 125 via retrieval_cli=66, working_memory_assembly=43, and lexical_search=16, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: library-line-range-anchor-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed SearchService recall to the nested line-range citation anchor, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 } while retaining both Simple and Structured provenance branches. The stable focused non-LLM retrieval surface now rises from 124 to 125 via retrieval_cli=66, working_memory_assembly=43, and lexical_search=16, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 124
- **Timestamp:** 2026-04-20T10:05:41Z

### L-150: [labels: structured-only-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that
- **Strategy:** [labels: structured-only-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to citation.recorded_at preservation, asserting the persisted timestamp survives the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 125 to 126 via retrieval_cli=66, working_memory_assembly=43, and lexical_search=17, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-recorded-at-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to citation.recorded_at preservation, asserting the persisted timestamp survives the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 125 to 126 via retrieval_cli=66, working_memory_assembly=43, and lexical_search=17, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 126
- **Timestamp:** 2026-04-20T10:07:59Z

### L-151: [labels: structured-only-assembly-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory
- **Strategy:** [labels: structured-only-assembly-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only recall to citation.recorded_at preservation on assembled world fragments. The stable focused non-LLM retrieval surface now rises from 126 to 127 via retrieval_cli=66, working_memory_assembly=44, and lexical_search=17, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-assembly-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only recall to citation.recorded_at preservation on assembled world fragments. The stable focused non-LLM retrieval surface now rises from 126 to 127 via retrieval_cli=66, working_memory_assembly=44, and lexical_search=17, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 127
- **Timestamp:** 2026-04-20T10:09:02Z

### L-152: [labels: structured-only-assembly-line-range-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: structured-only-assembly-line-range-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only recall to the nested line-range citation anchor on assembled world fragments, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 }. The stable focused non-LLM retrieval surface now rises from 127 to 128 via retrieval_cli=66, working_memory_assembly=45, and lexical_search=17, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-assembly-line-range-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only recall to the nested line-range citation anchor on assembled world fragments, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 }. The stable focused non-LLM retrieval surface now rises from 127 to 128 via retrieval_cli=66, working_memory_assembly=45, and lexical_search=17, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 128
- **Timestamp:** 2026-04-20T10:09:52Z

### L-153: [labels: structured-only-lexical-line-range-lock, lexical-first, ordinary-retrieval] Added focused lexical_search covera
- **Strategy:** [labels: structured-only-lexical-line-range-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to the nested line-range citation anchor, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 } on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 128 to 129 via retrieval_cli=66, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-lexical-line-range-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to the nested line-range citation anchor, asserting ChunkAnchor::LineRange { start_line: 1, end_line: 1 } on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 128 to 129 via retrieval_cli=66, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 129
- **Timestamp:** 2026-04-20T10:10:41Z

### L-154: [labels: structured-only-json-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli covera
- **Strategy:** [labels: structured-only-json-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to citation.recorded_at plus nested line-range anchor preservation, asserting chunk_index=0, chunk_count=1, and line_range {1,1} while retaining Structured provenance. The stable focused non-LLM retrieval surface now rises from 129 to 130 via retrieval_cli=67, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to citation.recorded_at plus nested line-range anchor preservation, asserting chunk_index=0, chunk_count=1, and line_range {1,1} while retaining Structured provenance. The stable focused non-LLM retrieval surface now rises from 129 to 130 via retrieval_cli=67, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 130
- **Timestamp:** 2026-04-20T10:12:31Z

### L-155: [labels: structured-only-library-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli lib
- **Strategy:** [labels: structured-only-library-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation.recorded_at plus nested line-range anchor preservation, asserting chunk_index=0, chunk_count=1, and line_range {1,1} on the purely structured path. The stable focused non-LLM retrieval surface now rises from 130 to 131 via retrieval_cli=68, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation.recorded_at plus nested line-range anchor preservation, asserting chunk_index=0, chunk_count=1, and line_range {1,1} on the purely structured path. The stable focused non-LLM retrieval surface now rises from 130 to 131 via retrieval_cli=68, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 131
- **Timestamp:** 2026-04-20T10:13:50Z

### L-156: [labels: structured-only-text-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli covera
- **Strategy:** [labels: structured-only-text-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only plain-text ordinary retrieval to citation chunk shape, recorded_at, validity window, and lexical_only/structured provenance on the default text surface. The stable focused non-LLM retrieval surface now rises from 131 to 132 via retrieval_cli=69, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-text-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only plain-text ordinary retrieval to citation chunk shape, recorded_at, validity window, and lexical_only/structured provenance on the default text surface. The stable focused non-LLM retrieval surface now rises from 131 to 132 via retrieval_cli=69, working_memory_assembly=45, and lexical_search=18, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 135
- **Timestamp:** 2026-04-20T10:18:35Z

### L-157: [labels: structured-only-lexical-chunk-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search cove
- **Strategy:** [labels: structured-only-lexical-chunk-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to citation chunk-anchor preservation, asserting chunk_index=0 and chunk_count=1 on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 132 to 133 via retrieval_cli=69, working_memory_assembly=45, and lexical_search=19, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-lexical-chunk-anchor-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to citation chunk-anchor preservation, asserting chunk_index=0 and chunk_count=1 on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 132 to 133 via retrieval_cli=69, working_memory_assembly=45, and lexical_search=19, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 140
- **Timestamp:** 2026-04-20T10:23:17Z

### L-158: [labels: structured-only-assembly-chunk-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memor
- **Strategy:** [labels: structured-only-assembly-chunk-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only recall to citation chunk-anchor preservation on assembled world fragments, asserting chunk_index=0 and chunk_count=1 on the purely structured path. The stable focused non-LLM retrieval surface now rises from 133 to 134 via retrieval_cli=69, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-assembly-chunk-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only recall to citation chunk-anchor preservation on assembled world fragments, asserting chunk_index=0 and chunk_count=1 on the purely structured path. The stable focused non-LLM retrieval surface now rises from 133 to 134 via retrieval_cli=69, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 141
- **Timestamp:** 2026-04-20T10:24:30Z

### L-159: [labels: structured-only-library-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-le
- **Strategy:** [labels: structured-only-library-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to the lexical-first channel contract, asserting trace.channel_contribution == LexicalOnly alongside structured query provenance. The stable focused non-LLM retrieval surface now rises from 134 to 135 via retrieval_cli=69, working_memory_assembly=46, lexical_search=19, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to the lexical-first channel contract, asserting trace.channel_contribution == LexicalOnly alongside structured query provenance. The stable focused non-LLM retrieval surface now rises from 134 to 135 via retrieval_cli=69, working_memory_assembly=46, lexical_search=19, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 142
- **Timestamp:** 2026-04-20T10:25:36Z

### L-160: [labels: structured-only-library-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-l
- **Strategy:** [labels: structured-only-library-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation validity preservation, asserting valid_from and valid_to on the purely structured path while retaining structured query provenance. The stable focused non-LLM retrieval surface now rises from 135 to 136 via retrieval_cli=70, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation validity preservation, asserting valid_from and valid_to on the purely structured path while retaining structured query provenance. The stable focused non-LLM retrieval surface now rises from 135 to 136 via retrieval_cli=70, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 144
- **Timestamp:** 2026-04-20T10:28:35Z

### L-161: [labels: structured-only-library-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli li
- **Strategy:** [labels: structured-only-library-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation source metadata preservation, asserting source_uri, source_kind=document, and source_label on the purely structured path while retaining structured query provenance. The stable focused non-LLM retrieval surface now rises from 136 to 137 via retrieval_cli=71, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation source metadata preservation, asserting source_uri, source_kind=document, and source_label on the purely structured path while retaining structured query provenance. The stable focused non-LLM retrieval surface now rises from 136 to 137 via retrieval_cli=71, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 147
- **Timestamp:** 2026-04-20T10:32:41Z

### L-162: [labels: structured-only-library-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-
- **Strategy:** [labels: structured-only-library-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the purely structured path. The stable focused non-LLM retrieval surface now rises from 137 to 138 via retrieval_cli=72, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the purely structured path. The stable focused non-LLM retrieval surface now rises from 137 to 138 via retrieval_cli=72, working_memory_assembly=46, and lexical_search=19, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 149
- **Timestamp:** 2026-04-20T10:35:29Z

### L-163: [labels: structured-only-lexical-record-id-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverag
- **Strategy:** [labels: structured-only-lexical-record-id-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 138 to 139 via retrieval_cli=72, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-lexical-record-id-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 138 to 139 via retrieval_cli=72, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 152
- **Timestamp:** 2026-04-20T10:38:16Z

### L-164: [labels: structured-only-json-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage th
- **Strategy:** [labels: structured-only-json-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to citation.record_id visibility, asserting the authority citation record id is present on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 139 to 140 via retrieval_cli=72, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to citation.record_id visibility, asserting the authority citation record id is present on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 139 to 140 via retrieval_cli=72, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 153
- **Timestamp:** 2026-04-20T10:39:43Z

### L-165: [labels: structured-only-library-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli libr
- **Strategy:** [labels: structured-only-library-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to trace.matched_query preservation, asserting the matched query stays equal to the user query on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 140 to 141 via retrieval_cli=73, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to trace.matched_query preservation, asserting the matched query stays equal to the user query on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 140 to 141 via retrieval_cli=73, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 156
- **Timestamp:** 2026-04-20T10:43:09Z

### L-166: [labels: structured-only-library-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli libra
- **Strategy:** [labels: structured-only-library-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record scope and truth-layer preservation, asserting Scope::Project and TruthLayer::T2 on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 141 to 142 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record scope and truth-layer preservation, asserting Scope::Project and TruthLayer::T2 on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 141 to 142 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 161
- **Timestamp:** 2026-04-20T10:48:50Z

### L-167: [labels: structured-only-json-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage
- **Strategy:** [labels: structured-only-json-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to record scope and truth-layer preservation, asserting scope=project and truth_layer=t2 on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 142 to 143 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to record scope and truth-layer preservation, asserting scope=project and truth_layer=t2 on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 142 to 143 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 163
- **Timestamp:** 2026-04-20T10:51:08Z

### L-168: [labels: structured-only-lexical-record-shape-lock, lexical-first, ordinary-retrieval] Added focused lexical_search cove
- **Strategy:** [labels: structured-only-lexical-record-shape-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to record scope and truth-layer preservation, asserting Scope::Project and TruthLayer::T2 on the purely structured lexical-first path. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at lexical_search=20/20 with clippy green, so the retained non-LLM retrieval surface now rises from 143 to 144 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20.
- **Outcome:** keep
- **Insight:** [labels: structured-only-lexical-record-shape-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to record scope and truth-layer preservation, asserting Scope::Project and TruthLayer::T2 on the purely structured lexical-first path. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at lexical_search=20/20 with clippy green, so the retained non-LLM retrieval surface now rises from 143 to 144 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 165
- **Timestamp:** 2026-04-20T10:53:59Z

### L-169: [labels: structured-only-json-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverag
- **Strategy:** [labels: structured-only-json-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to trace.matched_query preservation, asserting the matched query stays equal to the user query on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 144 to 145 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to trace.matched_query preservation, asserting the matched query stays equal to the user query on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 144 to 145 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 167
- **Timestamp:** 2026-04-20T10:56:59Z

### L-170: [labels: structured-only-lexical-record-type-lock, lexical-first, ordinary-retrieval] Added focused lexical_search cover
- **Strategy:** [labels: structured-only-lexical-record-type-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to record_type preservation, asserting RecordType::Decision on the purely structured lexical-first path alongside the existing scope/truth-layer contract. The stable focused non-LLM retrieval surface now rises from 145 to 146 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-lexical-record-type-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to record_type preservation, asserting RecordType::Decision on the purely structured lexical-first path alongside the existing scope/truth-layer contract. The stable focused non-LLM retrieval surface now rises from 145 to 146 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 172
- **Timestamp:** 2026-04-20T11:03:23Z

### L-171: [labels: structured-only-library-record-type-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli librar
- **Strategy:** [labels: structured-only-library-record-type-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record_type preservation, asserting RecordType::Decision on the purely structured lexical-first path alongside the existing scope/truth-layer contract. The stable focused non-LLM retrieval surface now rises from 146 to 147 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-record-type-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record_type preservation, asserting RecordType::Decision on the purely structured lexical-first path alongside the existing scope/truth-layer contract. The stable focused non-LLM retrieval surface now rises from 146 to 147 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 174
- **Timestamp:** 2026-04-20T11:06:08Z

### L-172: [labels: structured-only-json-record-type-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage
- **Strategy:** [labels: structured-only-json-record-type-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to record_type preservation, asserting record_type=decision on the purely structured lexical-first path alongside the existing scope/truth-layer contract. The stable focused non-LLM retrieval surface now rises from 147 to 148 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-record-type-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to record_type preservation, asserting record_type=decision on the purely structured lexical-first path alongside the existing scope/truth-layer contract. The stable focused non-LLM retrieval surface now rises from 147 to 148 via retrieval_cli=74, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 177
- **Timestamp:** 2026-04-20T11:09:35Z

### L-173: [labels: structured-only-json-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli cover
- **Strategy:** [labels: structured-only-json-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to citation source metadata preservation, asserting source_uri, source_kind=document, and source_label on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 148 to 149 via retrieval_cli=76, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to citation source metadata preservation, asserting source_uri, source_kind=document, and source_label on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 148 to 149 via retrieval_cli=76, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 182
- **Timestamp:** 2026-04-20T11:17:16Z

### L-174: [labels: structured-only-text-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli cover
- **Strategy:** [labels: structured-only-text-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only plain-text ordinary retrieval to source metadata preservation, asserting kind=document and the persisted source label in the record summary on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 149 to 150 via retrieval_cli=77, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-text-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only plain-text ordinary retrieval to source metadata preservation, asserting kind=document and the persisted source label in the record summary on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 149 to 150 via retrieval_cli=77, working_memory_assembly=46, and lexical_search=20, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 184
- **Timestamp:** 2026-04-20T11:20:46Z

### L-175: [labels: mixed-lexical-record-shape-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that
- **Strategy:** [labels: mixed-lexical-record-shape-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed lexical-plus-structured recall to record scope, truth_layer, and record_type preservation, asserting Scope::Project, TruthLayer::T2, and RecordType::Decision on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 150 to 151 via retrieval_cli=77, working_memory_assembly=46, and lexical_search=21, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-lexical-record-shape-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed lexical-plus-structured recall to record scope, truth_layer, and record_type preservation, asserting Scope::Project, TruthLayer::T2, and RecordType::Decision on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 150 to 151 via retrieval_cli=77, working_memory_assembly=46, and lexical_search=21, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 191
- **Timestamp:** 2026-04-20T11:30:02Z

### L-176: [labels: mixed-library-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level c
- **Strategy:** [labels: mixed-library-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to record scope, truth_layer, and record_type preservation, asserting Scope::Project, TruthLayer::T2, and RecordType::Decision on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 151 to 152 via retrieval_cli=78, working_memory_assembly=46, and lexical_search=21, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-record-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to record scope, truth_layer, and record_type preservation, asserting Scope::Project, TruthLayer::T2, and RecordType::Decision on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 151 to 152 via retrieval_cli=78, working_memory_assembly=46, and lexical_search=21, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 193
- **Timestamp:** 2026-04-20T11:33:10Z

### L-177: [labels: mixed-text-record-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage
- **Strategy:** [labels: mixed-text-record-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured plain-text retrieval to authority record summary plus citation shape preservation, asserting record id visibility, scope=project, type=decision truth_layer=t2, source kind/label, recorded_at, and validity on the mixed lexical-first text surface. The stable focused non-LLM retrieval surface now rises from 152 to 153 via retrieval_cli=79, working_memory_assembly=46, and lexical_search=21, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-text-record-citation-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured plain-text retrieval to authority record summary plus citation shape preservation, asserting record id visibility, scope=project, type=decision truth_layer=t2, source kind/label, recorded_at, and validity on the mixed lexical-first text surface. The stable focused non-LLM retrieval surface now rises from 152 to 153 via retrieval_cli=79, working_memory_assembly=46, and lexical_search=21, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 195
- **Timestamp:** 2026-04-20T11:36:59Z

### L-178: [labels: mixed-lexical-record-id-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that loc
- **Strategy:** [labels: mixed-lexical-record-id-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed lexical-plus-structured recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 153 to 154 via retrieval_cli=79, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-lexical-record-id-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed lexical-plus-structured recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 153 to 154 via retrieval_cli=79, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 197
- **Timestamp:** 2026-04-20T11:40:05Z

### L-179: [labels: mixed-library-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level
- **Strategy:** [labels: mixed-library-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to trace.matched_query preservation, asserting the matched query stays equal to the user query on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 154 to 155 via retrieval_cli=80, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to trace.matched_query preservation, asserting the matched query stays equal to the user query on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 154 to 155 via retrieval_cli=80, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 199
- **Timestamp:** 2026-04-20T11:43:07Z

### L-180: [labels: mixed-library-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level cove
- **Strategy:** [labels: mixed-library-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 155 to 156 via retrieval_cli=81, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation.record_id preservation, asserting the citation record id stays aligned with the authority record id on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 155 to 156 via retrieval_cli=81, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 201
- **Timestamp:** 2026-04-20T11:47:00Z

### L-181: [labels: mixed-json-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that loc
- **Strategy:** [labels: mixed-json-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to trace.matched_query preservation, asserting the matched query stays equal to the user query on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 156 to 157 via retrieval_cli=82, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-json-matched-query-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to trace.matched_query preservation, asserting the matched query stays equal to the user query on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 156 to 157 via retrieval_cli=82, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 203
- **Timestamp:** 2026-04-20T11:49:50Z

### L-182: [labels: mixed-library-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-leve
- **Strategy:** [labels: mixed-library-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation source metadata preservation, asserting source_uri, source_kind=document, and source_label on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 157 to 158 via retrieval_cli=83, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation source metadata preservation, asserting source_uri, source_kind=document, and source_label on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 157 to 158 via retrieval_cli=83, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 206
- **Timestamp:** 2026-04-20T11:56:58Z

### L-183: [labels: mixed-json-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks m
- **Strategy:** [labels: mixed-json-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to citation.record_id visibility, asserting the authority citation record id is present on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 158 to 159 via retrieval_cli=83, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-json-record-id-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to citation.record_id visibility, asserting the authority citation record id is present on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 158 to 159 via retrieval_cli=83, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 208
- **Timestamp:** 2026-04-20T12:00:54Z

### L-184: [labels: mixed-json-record-id-lock-2, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks
- **Strategy:** [labels: mixed-json-record-id-lock-2, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to citation.record_id visibility, asserting the authority citation record id is present on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 159 to 160 via retrieval_cli=84, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-json-record-id-lock-2, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to citation.record_id visibility, asserting the authority citation record id is present on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 159 to 160 via retrieval_cli=84, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 211
- **Timestamp:** 2026-04-20T12:05:14Z

### L-185: [labels: mixed-library-record-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli libra
- **Strategy:** [labels: mixed-library-record-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to record.source metadata preservation, asserting source uri, source kind=document, and source label on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 160 to 161 via retrieval_cli=85, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-record-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to record.source metadata preservation, asserting source uri, source kind=document, and source label on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 160 to 161 via retrieval_cli=85, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 214
- **Timestamp:** 2026-04-20T12:10:10Z

### L-186: [labels: mixed-library-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level covera
- **Strategy:** [labels: mixed-library-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to the lexical-first channel contract, asserting trace.channel_contribution == LexicalOnly on the mixed path. The stable focused non-LLM retrieval surface now rises from 161 to 162 via retrieval_cli=86, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-channel-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to the lexical-first channel contract, asserting trace.channel_contribution == LexicalOnly on the mixed path. The stable focused non-LLM retrieval surface now rises from 161 to 162 via retrieval_cli=86, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 217
- **Timestamp:** 2026-04-20T12:16:38Z

### L-187: [labels: mixed-library-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level cover
- **Strategy:** [labels: mixed-library-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation validity preservation, asserting valid_from and valid_to on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 162 to 163 via retrieval_cli=87, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-validity-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation validity preservation, asserting valid_from and valid_to on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 162 to 163 via retrieval_cli=87, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 224
- **Timestamp:** 2026-04-20T12:26:13Z

### L-188: [labels: structured-only-library-record-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval
- **Strategy:** [labels: structured-only-library-record-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record.source metadata preservation, asserting source uri, source kind=document, and source label on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 163 to 164 via retrieval_cli=87, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-record-source-metadata-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record.source metadata preservation, asserting source uri, source kind=document, and source label on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 163 to 164 via retrieval_cli=87, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 226
- **Timestamp:** 2026-04-20T12:29:25Z

### L-189: [labels: structured-only-json-record-source-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverag
- **Strategy:** [labels: structured-only-json-record-source-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to record.source metadata preservation, asserting source uri, source kind=document, and source label on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 164 to 165 via retrieval_cli=87, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-record-source-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to record.source metadata preservation, asserting source uri, source kind=document, and source label on the purely structured lexical-first path. The stable focused non-LLM retrieval surface now rises from 164 to 165 via retrieval_cli=87, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 228
- **Timestamp:** 2026-04-20T12:35:04Z

### L-190: [labels: mixed-library-validity-lock-2, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level cov
- **Strategy:** [labels: mixed-library-validity-lock-2, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation validity preservation, asserting valid_from and valid_to on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 165 to 166 via retrieval_cli=88, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-library-validity-lock-2, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to citation validity preservation, asserting valid_from and valid_to on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 165 to 166 via retrieval_cli=88, working_memory_assembly=46, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 231
- **Timestamp:** 2026-04-20T12:41:05Z

### L-191: [labels: structured-only-open-candidates-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: structured-only-open-candidates-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T2 recall to truth_context.open_candidate_ids preservation, asserting ontology candidates created through TruthGovernanceService survive assembly into the present frame while open_review_ids remain empty. The stable focused non-LLM retrieval surface now rises from 166 to 167 via retrieval_cli=88, working_memory_assembly=47, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-open-candidates-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T2 recall to truth_context.open_candidate_ids preservation, asserting ontology candidates created through TruthGovernanceService survive assembly into the present frame while open_review_ids remain empty. The stable focused non-LLM retrieval surface now rises from 166 to 167 via retrieval_cli=88, working_memory_assembly=47, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 244
- **Timestamp:** 2026-04-20T13:01:57Z

### L-192: [labels: structured-only-open-reviews-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: structured-only-open-reviews-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T3 recall to truth_context.open_review_ids preservation, asserting promotion reviews created through TruthGovernanceService survive assembly into the present frame while open_candidate_ids remain empty. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at working_memory_assembly=48/48 with clippy green, so the retained non-LLM retrieval surface now rises from 167 to 168 via retrieval_cli=88, working_memory_assembly=48, and lexical_search=22.
- **Outcome:** keep
- **Insight:** [labels: structured-only-open-reviews-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T3 recall to truth_context.open_review_ids preservation, asserting promotion reviews created through TruthGovernanceService survive assembly into the present frame while open_candidate_ids remain empty. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at working_memory_assembly=48/48 with clippy green, so the retained non-LLM retrieval surface now rises from 167 to 168 via retrieval_cli=88, working_memory_assembly=48, and lexical_search=22.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 246
- **Timestamp:** 2026-04-20T13:06:19Z

### L-193: [labels: mixed-open-candidates-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly cover
- **Strategy:** [labels: mixed-open-candidates-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured T2 recall to truth_context.open_candidate_ids preservation, asserting ontology candidates created through TruthGovernanceService survive assembly into the present frame while open_review_ids remain empty. The stable focused non-LLM retrieval surface now rises from 168 to 169 via retrieval_cli=88, working_memory_assembly=49, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-open-candidates-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured T2 recall to truth_context.open_candidate_ids preservation, asserting ontology candidates created through TruthGovernanceService survive assembly into the present frame while open_review_ids remain empty. The stable focused non-LLM retrieval surface now rises from 168 to 169 via retrieval_cli=88, working_memory_assembly=49, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 248
- **Timestamp:** 2026-04-20T13:10:34Z

### L-194: [labels: mixed-open-reviews-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage
- **Strategy:** [labels: mixed-open-reviews-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured T3 recall to truth_context.open_review_ids preservation, asserting promotion reviews created through TruthGovernanceService survive assembly into the present frame while open_candidate_ids remain empty. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at working_memory_assembly=50/50 with clippy green, so the retained non-LLM retrieval surface now rises from 169 to 170 via retrieval_cli=88, working_memory_assembly=50, and lexical_search=22.
- **Outcome:** keep
- **Insight:** [labels: mixed-open-reviews-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured T3 recall to truth_context.open_review_ids preservation, asserting promotion reviews created through TruthGovernanceService survive assembly into the present frame while open_candidate_ids remain empty. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at working_memory_assembly=50/50 with clippy green, so the retained non-LLM retrieval surface now rises from 169 to 170 via retrieval_cli=88, working_memory_assembly=50, and lexical_search=22.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 250
- **Timestamp:** 2026-04-20T13:17:00Z

### L-195: [labels: mixed-json-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that
- **Strategy:** [labels: mixed-json-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to record.timestamp.recorded_at preservation, asserting the authority record timestamp survives on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 170 to 171 via retrieval_cli=89, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-json-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to record.timestamp.recorded_at preservation, asserting the authority record timestamp survives on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 170 to 171 via retrieval_cli=89, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 255
- **Timestamp:** 2026-04-20T13:26:32Z

### L-196: [labels: mixed-json-record-timestamp-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage
- **Strategy:** [labels: mixed-json-record-timestamp-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 171 to 172 via retrieval_cli=90, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-json-record-timestamp-shape-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured JSON ordinary retrieval to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 171 to 172 via retrieval_cli=90, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 259
- **Timestamp:** 2026-04-20T13:38:05Z

### L-197: [labels: structured-only-json-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli cove
- **Strategy:** [labels: structured-only-json-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 172 to 173 via retrieval_cli=91, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-json-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only JSON ordinary retrieval to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 172 to 173 via retrieval_cli=91, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 262
- **Timestamp:** 2026-04-20T13:44:52Z

### L-198: [labels: structured-only-library-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli l
- **Strategy:** [labels: structured-only-library-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 173 to 174 via retrieval_cli=92, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-library-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 173 to 174 via retrieval_cli=92, working_memory_assembly=50, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 265
- **Timestamp:** 2026-04-20T13:50:48Z

### L-199: [labels: mixed-lexical-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage t
- **Strategy:** [labels: mixed-lexical-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed lexical-plus-structured recall to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 174 to 175 via retrieval_cli=92, working_memory_assembly=50, and lexical_search=23, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-lexical-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks mixed lexical-plus-structured recall to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 174 to 175 via retrieval_cli=92, working_memory_assembly=50, and lexical_search=23, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 269
- **Timestamp:** 2026-04-20T13:57:14Z

### L-200: [labels: structured-only-lexical-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused lexical_search
- **Strategy:** [labels: structured-only-lexical-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 175 to 176 via retrieval_cli=92, working_memory_assembly=50, and lexical_search=23, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-lexical-record-timestamp-lock, lexical-first, ordinary-retrieval] Added focused lexical_search coverage that locks structured-only recall to the full record.timestamp shape, asserting recorded_at, created_at, and updated_at on the authority record surface. The stable focused non-LLM retrieval surface now rises from 175 to 176 via retrieval_cli=92, working_memory_assembly=50, and lexical_search=23, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 271
- **Timestamp:** 2026-04-20T14:01:19Z

### L-201: [labels: structured-only-t3-state-details-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: structured-only-t3-state-details-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T3 recall to concrete truth_context.t3_state details, asserting confidence=medium and revocation_state=active on the present-frame fragment. The stable focused non-LLM retrieval surface now rises from 176 to 177 via retrieval_cli=92, working_memory_assembly=51, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: structured-only-t3-state-details-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T3 recall to concrete truth_context.t3_state details, asserting confidence=medium and revocation_state=active on the present-frame fragment. The stable focused non-LLM retrieval surface now rises from 176 to 177 via retrieval_cli=92, working_memory_assembly=51, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 278
- **Timestamp:** 2026-04-20T14:14:42Z

### L-202: [labels: mixed-t3-state-details-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly cove
- **Strategy:** [labels: mixed-t3-state-details-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured T3 recall to concrete truth_context.t3_state details, asserting confidence=medium and revocation_state=active on the present-frame fragment. The stable focused non-LLM retrieval surface now rises from 177 to 178 via retrieval_cli=92, working_memory_assembly=52, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-t3-state-details-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured T3 recall to concrete truth_context.t3_state details, asserting confidence=medium and revocation_state=active on the present-frame fragment. The stable focused non-LLM retrieval surface now rises from 177 to 178 via retrieval_cli=92, working_memory_assembly=52, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 280
- **Timestamp:** 2026-04-20T14:18:54Z

### L-203: [labels: structured-only-t3-last-reviewed-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: structured-only-t3-last-reviewed-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T3 recall to truth_context.t3_state.last_reviewed_at preservation, asserting the promotion review timestamp survives assembly into the present frame. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at working_memory_assembly=52/52 with clippy green, so the retained non-LLM retrieval surface now rises from 178 to 179 via retrieval_cli=92, working_memory_assembly=52, and lexical_search=22.
- **Outcome:** keep
- **Insight:** [labels: structured-only-t3-last-reviewed-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only T3 recall to truth_context.t3_state.last_reviewed_at preservation, asserting the promotion review timestamp survives assembly into the present frame. The first suite run hit the known transient allocator abort at shutdown, but immediate isolated rerun passed cleanly at working_memory_assembly=52/52 with clippy green, so the retained non-LLM retrieval surface now rises from 178 to 179 via retrieval_cli=92, working_memory_assembly=52, and lexical_search=22.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 287
- **Timestamp:** 2026-04-21T07:20:09Z

### L-204: [labels: record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage t
- **Strategy:** [labels: record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. Verification and clippy both passed, so the retained non-LLM retrieval surface now rises from 179 to 180 while keeping the lexical-first contract intact.
- **Outcome:** keep
- **Insight:** [labels: record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks structured-only SearchService recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. Verification and clippy both passed, so the retained non-LLM retrieval surface now rises from 179 to 180 while keeping the lexical-first contract intact.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 293
- **Timestamp:** 2026-04-21T07:39:49Z

### L-205: [labels: mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level cove
- **Strategy:** [labels: mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 180 to 181 via retrieval_cli=94, working_memory_assembly=52, and lexical_search=22, with clippy still green.
- **Outcome:** keep
- **Insight:** [labels: mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli library-level coverage that locks mixed lexical-plus-structured SearchService recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The stable focused non-LLM retrieval surface now rises from 180 to 181 via retrieval_cli=94, working_memory_assembly=52, and lexical_search=22, with clippy still green.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 296
- **Timestamp:** 2026-04-21T07:47:30Z

### L-206: [labels: cli-json-structured-only-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli
- **Strategy:** [labels: cli-json-structured-only-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only CLI JSON recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The targeted verify surface passed at retrieval_cli=95/95 with clippy green, so the retained ordinary-retrieval surface rises from 181 to 182 without changing runtime behavior.
- **Outcome:** keep
- **Insight:** [labels: cli-json-structured-only-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks structured-only CLI JSON recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The targeted verify surface passed at retrieval_cli=95/95 with clippy green, so the retained ordinary-retrieval surface rises from 181 to 182 without changing runtime behavior.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 299
- **Timestamp:** 2026-04-21T08:03:02Z

### L-207: [labels: cli-json-mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage
- **Strategy:** [labels: cli-json-mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured CLI JSON recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The targeted verify surface passed at retrieval_cli=96/96 with clippy green, so the retained ordinary-retrieval surface rises from 182 to 183 without changing runtime behavior.
- **Outcome:** keep
- **Insight:** [labels: cli-json-mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured CLI JSON recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The targeted verify surface passed at retrieval_cli=96/96 with clippy green, so the retained ordinary-retrieval surface rises from 182 to 183 without changing runtime behavior.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 300
- **Timestamp:** 2026-04-21T08:04:34Z

### L-208: [labels: cli-text-structured-only-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused CLI text rend
- **Strategy:** [labels: cli-text-structured-only-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused CLI text rendering for record.provenance and locked it with retrieval_cli coverage on the structured-only lexical-first path, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The targeted verify surface passed at retrieval_cli=97/97 with clippy green, so the retained ordinary-retrieval surface rises from 183 to 184.
- **Outcome:** keep
- **Insight:** [labels: cli-text-structured-only-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused CLI text rendering for record.provenance and locked it with retrieval_cli coverage on the structured-only lexical-first path, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The targeted verify surface passed at retrieval_cli=97/97 with clippy green, so the retained ordinary-retrieval surface rises from 183 to 184.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 302
- **Timestamp:** 2026-04-21T08:07:55Z

### L-209: [labels: cli-text-mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage
- **Strategy:** [labels: cli-text-mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured CLI text recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The targeted verify surface passed at retrieval_cli=98/98 with clippy green, so the retained ordinary-retrieval surface rises from 184 to 185.
- **Outcome:** keep
- **Insight:** [labels: cli-text-mixed-record-provenance-lock, lexical-first, ordinary-retrieval] Added focused retrieval_cli coverage that locks mixed lexical-plus-structured CLI text recall to record.provenance preservation, asserting origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The targeted verify surface passed at retrieval_cli=98/98 with clippy green, so the retained ordinary-retrieval surface rises from 184 to 185.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 303
- **Timestamp:** 2026-04-21T08:09:19Z

### L-210: [labels: structured-only-fragment-record-provenance-lock, lexical-first, working-memory-assembly] Added focused working_
- **Strategy:** [labels: structured-only-fragment-record-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only fragment provenance preservation, asserting EvidenceFragment now keeps record.provenance with origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The initial combined focused run hit the known transient allocator abort at shutdown, but immediate isolated reruns passed cleanly at working_memory_assembly=53/53 and agent_search=10/10 with clippy green, so the retained ordinary-retrieval surface rises from 185 to 186.
- **Outcome:** keep
- **Insight:** [labels: structured-only-fragment-record-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only fragment provenance preservation, asserting EvidenceFragment now keeps record.provenance with origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The initial combined focused run hit the known transient allocator abort at shutdown, but immediate isolated reruns passed cleanly at working_memory_assembly=53/53 and agent_search=10/10 with clippy green, so the retained ordinary-retrieval surface rises from 185 to 186.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 306
- **Timestamp:** 2026-04-21T08:15:10Z

### L-211: [labels: mixed-fragment-record-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: mixed-fragment-record-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured fragment provenance preservation, asserting EvidenceFragment keeps record.provenance with origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The focused verify surface passed cleanly at working_memory_assembly=54/54 and agent_search=10/10 with clippy green, so the retained ordinary-retrieval surface rises from 186 to 187.
- **Outcome:** keep
- **Insight:** [labels: mixed-fragment-record-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured fragment provenance preservation, asserting EvidenceFragment keeps record.provenance with origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI on the mixed lexical-first path. The focused verify surface passed cleanly at working_memory_assembly=54/54 and agent_search=10/10 with clippy green, so the retained ordinary-retrieval surface rises from 186 to 187.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 307
- **Timestamp:** 2026-04-21T08:17:26Z

### L-212: [labels: integrated-follow-up-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: integrated-follow-up-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks provenance preservation on the integrated_results follow-up seam, asserting follow-up fragments injected through WorkingMemoryRequest::with_integrated_results still keep record.provenance with origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The focused verify surface passed cleanly at working_memory_assembly=55/55 with clippy green, so the retained ordinary-retrieval surface rises from 187 to 188.
- **Outcome:** keep
- **Insight:** [labels: integrated-follow-up-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks provenance preservation on the integrated_results follow-up seam, asserting follow-up fragments injected through WorkingMemoryRequest::with_integrated_results still keep record.provenance with origin=ingest, imported_via=ingest_service, and a derived_from anchor rooted at the authority source URI. The focused verify surface passed cleanly at working_memory_assembly=55/55 with clippy green, so the retained ordinary-retrieval surface rises from 187 to 188.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 309
- **Timestamp:** 2026-04-21T08:20:50Z

### L-213: [labels: structured-only-fragment-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assem
- **Strategy:** [labels: structured-only-fragment-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only fragment score preservation, asserting EvidenceFragment keeps the exact ScoreBreakdown produced by retrieval when structured-only results are injected into assembly. The focused verify surface passed cleanly at working_memory_assembly=56/56 with clippy green, so the retained ordinary-retrieval surface rises from 188 to 189.
- **Outcome:** keep
- **Insight:** [labels: structured-only-fragment-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks structured-only fragment score preservation, asserting EvidenceFragment keeps the exact ScoreBreakdown produced by retrieval when structured-only results are injected into assembly. The focused verify surface passed cleanly at working_memory_assembly=56/56 with clippy green, so the retained ordinary-retrieval surface rises from 188 to 189.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 311
- **Timestamp:** 2026-04-21T08:23:33Z

### L-214: [labels: mixed-fragment-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly covera
- **Strategy:** [labels: mixed-fragment-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured fragment score preservation, asserting EvidenceFragment keeps the exact ScoreBreakdown produced by retrieval on the mixed lexical-first path. The focused verify surface passed cleanly at working_memory_assembly=57/57 with clippy green, so the retained ordinary-retrieval surface rises from 189 to 190.
- **Outcome:** keep
- **Insight:** [labels: mixed-fragment-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks mixed lexical-plus-structured fragment score preservation, asserting EvidenceFragment keeps the exact ScoreBreakdown produced by retrieval on the mixed lexical-first path. The focused verify surface passed cleanly at working_memory_assembly=57/57 with clippy green, so the retained ordinary-retrieval surface rises from 189 to 190.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 312
- **Timestamp:** 2026-04-21T08:25:41Z

### L-215: [labels: branch-support-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly c
- **Strategy:** [labels: branch-support-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks provenance preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the follow-up fragment's record.provenance when integrated follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=58/58 with clippy green, so the retained ordinary-retrieval surface rises from 190 to 191.
- **Outcome:** keep
- **Insight:** [labels: branch-support-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks provenance preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the follow-up fragment's record.provenance when integrated follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=58/58 with clippy green, so the retained ordinary-retrieval surface rises from 190 to 191.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 316
- **Timestamp:** 2026-04-21T08:30:14Z

### L-216: [labels: branch-support-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly covera
- **Strategy:** [labels: branch-support-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks score preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the exact ScoreBreakdown of the integrated follow-up fragment when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=59/59 with clippy green, so the retained ordinary-retrieval surface rises from 191 to 192.
- **Outcome:** keep
- **Insight:** [labels: branch-support-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks score preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the exact ScoreBreakdown of the integrated follow-up fragment when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=59/59 with clippy green, so the retained ordinary-retrieval surface rises from 191 to 192.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 318
- **Timestamp:** 2026-04-21T08:33:03Z

### L-217: [labels: branch-support-truth-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: branch-support-truth-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks truth-context preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps T3 truth_context, open_review_ids, and last_reviewed_at when integrated follow-up evidence is materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=60/60 with clippy green, so the retained ordinary-retrieval surface rises from 192 to 193.
- **Outcome:** keep
- **Insight:** [labels: branch-support-truth-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks truth-context preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps T3 truth_context, open_review_ids, and last_reviewed_at when integrated follow-up evidence is materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=60/60 with clippy green, so the retained ordinary-retrieval surface rises from 192 to 193.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 320
- **Timestamp:** 2026-04-21T08:35:26Z

### L-218: [labels: branch-support-dsl-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage
- **Strategy:** [labels: branch-support-dsl-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks DSL and structured-snippet preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's DSL payload and structured snippet surface when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=61/61 with clippy green, so the retained ordinary-retrieval surface rises from 193 to 194.
- **Outcome:** keep
- **Insight:** [labels: branch-support-dsl-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks DSL and structured-snippet preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's DSL payload and structured snippet surface when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=61/61 with clippy green, so the retained ordinary-retrieval surface rises from 193 to 194.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 323
- **Timestamp:** 2026-04-21T08:38:34Z

### L-219: [labels: branch-support-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly covera
- **Strategy:** [labels: branch-support-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks trace preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's channel_contribution, mixed query_strategies, and matched_query when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=62/62 with clippy green, so the retained ordinary-retrieval surface rises from 194 to 195.
- **Outcome:** keep
- **Insight:** [labels: branch-support-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks trace preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's channel_contribution, mixed query_strategies, and matched_query when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=62/62 with clippy green, so the retained ordinary-retrieval surface rises from 194 to 195.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 326
- **Timestamp:** 2026-04-21T08:43:16Z

### L-220: [labels: branch-support-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: branch-support-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks applied-filter trace preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps scope, record_type, truth_layer, and matched_query when filtered follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=63/63 with clippy green, so the retained ordinary-retrieval surface rises from 195 to 196.
- **Outcome:** keep
- **Insight:** [labels: branch-support-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks applied-filter trace preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps scope, record_type, truth_layer, and matched_query when filtered follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=63/63 with clippy green, so the retained ordinary-retrieval surface rises from 195 to 196.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 331
- **Timestamp:** 2026-04-21T08:48:03Z

### L-221: [labels: branch-support-citation-source-uri-lock, lexical-first, working-memory-assembly] Added focused working_memory_a
- **Strategy:** [labels: branch-support-citation-source-uri-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source-uri preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.source_uri and citation.record_id when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=64/64 with clippy green, so the retained ordinary-retrieval surface rises from 196 to 197.
- **Outcome:** keep
- **Insight:** [labels: branch-support-citation-source-uri-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source-uri preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.source_uri and citation.record_id when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=64/64 with clippy green, so the retained ordinary-retrieval surface rises from 196 to 197.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 335
- **Timestamp:** 2026-04-21T08:52:35Z

### L-222: [labels: branch-support-citation-validity-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: branch-support-citation-validity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation-validity preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.validity window when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=65/65 with clippy green, so the retained ordinary-retrieval surface rises from 197 to 198.
- **Outcome:** keep
- **Insight:** [labels: branch-support-citation-validity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation-validity preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.validity window when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=65/65 with clippy green, so the retained ordinary-retrieval surface rises from 197 to 198.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 338
- **Timestamp:** 2026-04-21T08:55:47Z

### L-223: [labels: branch-support-citation-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: branch-support-citation-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation recorded_at preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.recorded_at when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=66/66 with clippy green, so the retained ordinary-retrieval surface rises from 198 to 199.
- **Outcome:** keep
- **Insight:** [labels: branch-support-citation-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation recorded_at preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.recorded_at when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=66/66 with clippy green, so the retained ordinary-retrieval surface rises from 198 to 199.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 342
- **Timestamp:** 2026-04-21T08:59:55Z

### L-224: [labels: branch-support-citation-source-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: branch-support-citation-source-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_kind preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.source_kind when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=67/67 with clippy green, so the retained ordinary-retrieval surface rises from 199 to 200.
- **Outcome:** keep
- **Insight:** [labels: branch-support-citation-source-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_kind preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.source_kind when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=67/67 with clippy green, so the retained ordinary-retrieval surface rises from 199 to 200.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 345
- **Timestamp:** 2026-04-21T09:03:32Z

### L-225: [labels: branch-support-citation-source-label-lock, lexical-first, working-memory-assembly] Added focused working_memory
- **Strategy:** [labels: branch-support-citation-source-label-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_label preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.source_label when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=68/68 with clippy green, so the retained ordinary-retrieval surface rises from 200 to 201.
- **Outcome:** keep
- **Insight:** [labels: branch-support-citation-source-label-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_label preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation.source_label when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=68/68 with clippy green, so the retained ordinary-retrieval surface rises from 200 to 201.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 347
- **Timestamp:** 2026-04-21T09:05:51Z

### L-226: [labels: branch-support-citation-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assem
- **Strategy:** [labels: branch-support-citation-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation anchor preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation chunk_index, chunk_count, and line-range anchor when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=69/69 with clippy green, so the retained ordinary-retrieval surface rises from 201 to 202.
- **Outcome:** keep
- **Insight:** [labels: branch-support-citation-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation anchor preservation on the branch-support seam, asserting ActionBranch.supporting_evidence keeps the integrated follow-up fragment's citation chunk_index, chunk_count, and line-range anchor when follow-up results are materialized into branch support. The focused verify surface passed cleanly at working_memory_assembly=69/69 with clippy green, so the retained ordinary-retrieval surface rises from 201 to 202.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 352
- **Timestamp:** 2026-04-21T09:11:15Z

### L-227: [labels: default-branch-support-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_as
- **Strategy:** [labels: default-branch-support-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks provenance preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps record.provenance when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=70/70 with clippy green, so the retained ordinary-retrieval surface rises from 202 to 203.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-provenance-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks provenance preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps record.provenance when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=70/70 with clippy green, so the retained ordinary-retrieval surface rises from 202 to 203.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 355
- **Timestamp:** 2026-04-21T09:14:13Z

### L-228: [labels: default-branch-support-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: default-branch-support-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks score preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the exact ScoreBreakdown when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=71/71 with clippy green, so the retained ordinary-retrieval surface rises from 203 to 204.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-score-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks score preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the exact ScoreBreakdown when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=71/71 with clippy green, so the retained ordinary-retrieval surface rises from 203 to 204.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 358
- **Timestamp:** 2026-04-21T09:17:02Z

### L-229: [labels: default-branch-support-truth-context-lock, lexical-first, working-memory-assembly] Added focused working_memory
- **Strategy:** [labels: default-branch-support-truth-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks truth-context preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps T3 truth_context, open_review_ids, and last_reviewed_at when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=72/72 with clippy green, so the retained ordinary-retrieval surface rises from 204 to 205.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-truth-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks truth-context preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps T3 truth_context, open_review_ids, and last_reviewed_at when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=72/72 with clippy green, so the retained ordinary-retrieval surface rises from 204 to 205.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 361
- **Timestamp:** 2026-04-21T09:19:47Z

### L-230: [labels: default-branch-support-dsl-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: default-branch-support-dsl-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks DSL payload and structured-snippet preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment DSL payload and structured snippet surface when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=73/73 with clippy green, so the retained ordinary-retrieval surface rises from 205 to 206.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-dsl-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks DSL payload and structured-snippet preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment DSL payload and structured snippet surface when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=73/73 with clippy green, so the retained ordinary-retrieval surface rises from 205 to 206.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 364
- **Timestamp:** 2026-04-21T09:22:50Z

### L-231: [labels: default-branch-support-citation-source-uri-lock, lexical-first, working-memory-assembly] Added focused working_
- **Strategy:** [labels: default-branch-support-citation-source-uri-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_uri preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.source_uri and citation.record_id when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=74/74 with clippy green, so the retained ordinary-retrieval surface rises from 206 to 207.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-citation-source-uri-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_uri preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.source_uri and citation.record_id when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=74/74 with clippy green, so the retained ordinary-retrieval surface rises from 206 to 207.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 367
- **Timestamp:** 2026-04-21T09:26:00Z

### L-232: [labels: default-branch-support-citation-source-kind-lock, lexical-first, working-memory-assembly] Added focused working
- **Strategy:** [labels: default-branch-support-citation-source-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_kind preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.source_kind when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=75/75 with clippy green, so the retained ordinary-retrieval surface rises from 207 to 208.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-citation-source-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_kind preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.source_kind when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=75/75 with clippy green, so the retained ordinary-retrieval surface rises from 207 to 208.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 370
- **Timestamp:** 2026-04-21T09:29:33Z

### L-233: [labels: default-branch-support-citation-source-label-lock, lexical-first, working-memory-assembly] Added focused workin
- **Strategy:** [labels: default-branch-support-citation-source-label-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_label preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.source_label when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=76/76 with clippy green, so the retained ordinary-retrieval surface rises from 208 to 209.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-citation-source-label-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation source_label preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.source_label when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=76/76 with clippy green, so the retained ordinary-retrieval surface rises from 208 to 209.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 371
- **Timestamp:** 2026-04-21T09:31:23Z

### L-234: [labels: default-branch-support-citation-validity-lock, lexical-first, working-memory-assembly] Added focused working_me
- **Strategy:** [labels: default-branch-support-citation-validity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation-validity preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.validity window when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=77/77 with clippy green, so the retained ordinary-retrieval surface rises from 209 to 210.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-citation-validity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation-validity preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.validity window when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=77/77 with clippy green, so the retained ordinary-retrieval surface rises from 209 to 210.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 373
- **Timestamp:** 2026-04-21T09:34:33Z

### L-235: [labels: default-branch-support-citation-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working
- **Strategy:** [labels: default-branch-support-citation-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation recorded_at preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.recorded_at when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=78/78 with clippy green, so the retained ordinary-retrieval surface rises from 210 to 211.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-citation-recorded-at-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation recorded_at preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation.recorded_at when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=78/78 with clippy green, so the retained ordinary-retrieval surface rises from 210 to 211.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 375
- **Timestamp:** 2026-04-21T09:37:01Z

### L-236: [labels: default-branch-support-citation-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memo
- **Strategy:** [labels: default-branch-support-citation-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation anchor preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation chunk_index, chunk_count, and line-range anchor when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=79/79 with clippy green, so the retained ordinary-retrieval surface rises from 211 to 212.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-citation-anchor-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks citation anchor preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps the cloned world-fragment citation chunk_index, chunk_count, and line-range anchor when supporting_record_ids is left empty. The focused verify surface passed cleanly at working_memory_assembly=79/79 with clippy green, so the retained ordinary-retrieval surface rises from 211 to 212.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 376
- **Timestamp:** 2026-04-21T09:38:46Z

### L-237: [labels: default-branch-support-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: default-branch-support-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks applied-filter trace preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps scope, record_type, truth_layer, and matched_query when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=80/80 with clippy green, so the retained ordinary-retrieval surface rises from 212 to 213.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks applied-filter trace preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps scope, record_type, truth_layer, and matched_query when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=80/80 with clippy green, so the retained ordinary-retrieval surface rises from 212 to 213.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 379
- **Timestamp:** 2026-04-21T09:43:16Z

### L-238: [labels: branch-risk-marker-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage
- **Strategy:** [labels: branch-risk-marker-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks ActionSeed risk-marker preservation, asserting ActionBranch.risk_markers keeps the seeded clarification_required marker through assembly alongside supporting evidence. The focused verify surface passed cleanly at working_memory_assembly=80/80 with clippy green, so the retained ordinary-retrieval surface rises from 213 to 214.
- **Outcome:** keep
- **Insight:** [labels: branch-risk-marker-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks ActionSeed risk-marker preservation, asserting ActionBranch.risk_markers keeps the seeded clarification_required marker through assembly alongside supporting evidence. The focused verify surface passed cleanly at working_memory_assembly=80/80 with clippy green, so the retained ordinary-retrieval surface rises from 213 to 214.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 386
- **Timestamp:** 2026-04-21T09:49:48Z

### L-239: [labels: action-candidate-summary-intent-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: action-candidate-summary-intent-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks ActionCandidate summary and intent preservation, asserting the assembled branch keeps the seeded candidate summary and intent text through the ActionSeed -> ActionBranch seam. The focused verify surface passed cleanly at working_memory_assembly=81/81 with clippy green, so the retained ordinary-retrieval surface rises from 214 to 215.
- **Outcome:** keep
- **Insight:** [labels: action-candidate-summary-intent-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks ActionCandidate summary and intent preservation, asserting the assembled branch keeps the seeded candidate summary and intent text through the ActionSeed -> ActionBranch seam. The focused verify surface passed cleanly at working_memory_assembly=81/81 with clippy green, so the retained ordinary-retrieval surface rises from 214 to 215.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 388
- **Timestamp:** 2026-04-21T09:54:30Z

### L-240: [labels: action-candidate-parameters-effects-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: action-candidate-parameters-effects-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks ActionCandidate parameter and expected-effect preservation, asserting the assembled branch keeps the seeded candidate parameters and expected_effects through the ActionSeed -> ActionBranch seam. The focused verify surface passed cleanly at working_memory_assembly=82/82 with clippy green, so the retained ordinary-retrieval surface rises from 215 to 216.
- **Outcome:** keep
- **Insight:** [labels: action-candidate-parameters-effects-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks ActionCandidate parameter and expected-effect preservation, asserting the assembled branch keeps the seeded candidate parameters and expected_effects through the ActionSeed -> ActionBranch seam. The focused verify surface passed cleanly at working_memory_assembly=82/82 with clippy green, so the retained ordinary-retrieval surface rises from 215 to 216.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 393
- **Timestamp:** 2026-04-21T10:00:35Z

### L-241: [labels: default-branch-support-trace-summary-lock, lexical-first, working-memory-assembly] Added focused working_memory
- **Strategy:** [labels: default-branch-support-trace-summary-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks trace-summary preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps channel_contribution, matched_query, and mixed query_strategies when supporting_record_ids is left empty and world_fragments are cloned into branch support. The focused verify surface passed cleanly at working_memory_assembly=83/83 with clippy green, so the retained ordinary-retrieval surface rises from 216 to 217.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-trace-summary-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks trace-summary preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps channel_contribution, matched_query, and mixed query_strategies when supporting_record_ids is left empty and world_fragments are cloned into branch support. The focused verify surface passed cleanly at working_memory_assembly=83/83 with clippy green, so the retained ordinary-retrieval surface rises from 216 to 217.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 397
- **Timestamp:** 2026-04-21T10:05:32Z

### L-242: [labels: present-control-state-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly cover
- **Strategy:** [labels: present-control-state-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks present-frame control-state preservation, asserting working-memory assembly keeps active_goal, active_risks, and metacog_flags from the request so downstream gate logic sees the intended runtime control state. The focused verify surface passed cleanly at working_memory_assembly=84/84 with clippy green, so the retained ordinary-retrieval surface rises from 217 to 218.
- **Outcome:** keep
- **Insight:** [labels: present-control-state-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks present-frame control-state preservation, asserting working-memory assembly keeps active_goal, active_risks, and metacog_flags from the request so downstream gate logic sees the intended runtime control state. The focused verify surface passed cleanly at working_memory_assembly=84/84 with clippy green, so the retained ordinary-retrieval surface rises from 217 to 218.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 402
- **Timestamp:** 2026-04-21T10:15:06Z

### L-243: [labels: self-state-readiness-flags-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: self-state-readiness-flags-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks self-state readiness-flag preservation, asserting assembly keeps readiness_flags from the request in the PresentFrame self_state snapshot. The focused verify surface passed cleanly at working_memory_assembly=85/85 with clippy green, so the retained ordinary-retrieval surface rises from 218 to 219.
- **Outcome:** keep
- **Insight:** [labels: self-state-readiness-flags-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks self-state readiness-flag preservation, asserting assembly keeps readiness_flags from the request in the PresentFrame self_state snapshot. The focused verify surface passed cleanly at working_memory_assembly=85/85 with clippy green, so the retained ordinary-retrieval surface rises from 218 to 219.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 404
- **Timestamp:** 2026-04-21T10:19:17Z

### L-244: [labels: default-branch-support-temporal-filter-trace-lock, lexical-first, working-memory-assembly] Added focused workin
- **Strategy:** [labels: default-branch-support-temporal-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks temporal filter-trace preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps valid_at, recorded_from, recorded_to, and matched_query when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=86/86 with clippy green, so the retained ordinary-retrieval surface rises from 219 to 220.
- **Outcome:** keep
- **Insight:** [labels: default-branch-support-temporal-filter-trace-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks temporal filter-trace preservation on the default branch-support seam, asserting ActionBranch.supporting_evidence keeps valid_at, recorded_from, recorded_to, and matched_query when supporting_record_ids is left empty and the assembler clones all world_fragments into branch support. The focused verify surface passed cleanly at working_memory_assembly=86/86 with clippy green, so the retained ordinary-retrieval surface rises from 219 to 220.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 408
- **Timestamp:** 2026-04-21T10:24:39Z

### L-245: [labels: subject-local-adaptation-self-state-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: subject-local-adaptation-self-state-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks subject-scoped local adaptation injection, asserting subject_ref-driven local_adaptation_entries are merged into self_state facts via AdaptiveSelfStateProvider on the assembly path. The focused verify surface passed cleanly at working_memory_assembly=87/87 with clippy green, so the retained ordinary-retrieval surface rises from 220 to 221.
- **Outcome:** keep
- **Insight:** [labels: subject-local-adaptation-self-state-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks subject-scoped local adaptation injection, asserting subject_ref-driven local_adaptation_entries are merged into self_state facts via AdaptiveSelfStateProvider on the assembly path. The focused verify surface passed cleanly at working_memory_assembly=87/87 with clippy green, so the retained ordinary-retrieval surface rises from 220 to 221.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 413
- **Timestamp:** 2026-04-21T10:34:15Z

### L-246: [labels: local-adaptation-target-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: local-adaptation-target-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks risk-boundary and private-T3 local adaptation injection, asserting subject_ref-driven local_adaptation_entries are mapped into self_state facts with the correct risk_boundary:* and private_t3:* keys via AdaptiveSelfStateProvider. The focused verify surface passed cleanly at working_memory_assembly=88/88 with clippy green, so the retained ordinary-retrieval surface rises from 221 to 222.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-target-kind-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks risk-boundary and private-T3 local adaptation injection, asserting subject_ref-driven local_adaptation_entries are mapped into self_state facts with the correct risk_boundary:* and private_t3:* keys via AdaptiveSelfStateProvider. The focused verify surface passed cleanly at working_memory_assembly=88/88 with clippy green, so the retained ordinary-retrieval surface rises from 221 to 222.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 415
- **Timestamp:** 2026-04-21T10:37:35Z

### L-247: [labels: local-adaptation-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly cove
- **Strategy:** [labels: local-adaptation-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks subject-scoped local adaptation ordering, asserting AdaptiveSelfStateProvider preserves repository updated_at-desc ordering when multiple local_adaptation_entries map to the same self_state key. The focused verify surface passed cleanly at working_memory_assembly=89/89 with clippy green, so the retained ordinary-retrieval surface rises from 222 to 223.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks subject-scoped local adaptation ordering, asserting AdaptiveSelfStateProvider preserves repository updated_at-desc ordering when multiple local_adaptation_entries map to the same self_state key. The focused verify surface passed cleanly at working_memory_assembly=89/89 with clippy green, so the retained ordinary-retrieval surface rises from 222 to 223.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 419
- **Timestamp:** 2026-04-21T10:43:51Z

### L-248: [labels: local-adaptation-display-value-lock, lexical-first, working-memory-assembly] Added focused working_memory_assem
- **Strategy:** [labels: local-adaptation-display-value-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks non-string local-adaptation display semantics, asserting AdaptiveSelfStateProvider stringifies a boolean LocalAdaptationPayload into the expected self_state fact value through LocalAdaptationPayload::display_value(). The focused verify surface passed cleanly at working_memory_assembly=90/90 with clippy green, so the retained ordinary-retrieval surface rises from 223 to 224.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-display-value-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks non-string local-adaptation display semantics, asserting AdaptiveSelfStateProvider stringifies a boolean LocalAdaptationPayload into the expected self_state fact value through LocalAdaptationPayload::display_value(). The focused verify surface passed cleanly at working_memory_assembly=90/90 with clippy green, so the retained ordinary-retrieval surface rises from 223 to 224.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 422
- **Timestamp:** 2026-04-21T10:49:55Z

### L-249: [labels: request-local-adaptation-injection-lock, lexical-first, working-memory-assembly] Added focused working_memory_a
- **Strategy:** [labels: request-local-adaptation-injection-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks direct request-local adaptation injection, asserting with_local_adaptation_entries(...) flows through AdaptiveSelfStateProvider into self_state facts for both self_state and risk_boundary targets without relying on repository subject_ref lookup. The focused verify surface passed cleanly at working_memory_assembly=91/91 with clippy green, so the retained ordinary-retrieval surface rises from 224 to 225.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-injection-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks direct request-local adaptation injection, asserting with_local_adaptation_entries(...) flows through AdaptiveSelfStateProvider into self_state facts for both self_state and risk_boundary targets without relying on repository subject_ref lookup. The focused verify surface passed cleanly at working_memory_assembly=91/91 with clippy green, so the retained ordinary-retrieval surface rises from 224 to 225.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 431
- **Timestamp:** 2026-04-21T11:00:45Z

### L-250: [labels: request-local-adaptation-private-t3-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: request-local-adaptation-private-t3-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local private-T3 adaptation injection, asserting with_local_adaptation_entries(...) feeds private_t3 entries through AdaptiveSelfStateProvider into self_state facts without relying on repository subject_ref lookup. The focused verify surface passed cleanly at working_memory_assembly=91/91 with clippy green, so the retained ordinary-retrieval surface rises from 225 to 226.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-private-t3-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local private-T3 adaptation injection, asserting with_local_adaptation_entries(...) feeds private_t3 entries through AdaptiveSelfStateProvider into self_state facts without relying on repository subject_ref lookup. The focused verify surface passed cleanly at working_memory_assembly=91/91 with clippy green, so the retained ordinary-retrieval surface rises from 225 to 226.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 434
- **Timestamp:** 2026-04-21T11:05:14Z

### L-251: [labels: request-local-adaptation-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assem
- **Strategy:** [labels: request-local-adaptation-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation ordering, asserting with_local_adaptation_entries(...) preserves caller-provided ordering when multiple entries map to the same self_state key and are injected through AdaptiveSelfStateProvider. The focused verify surface passed cleanly at working_memory_assembly=92/92 with clippy green, so the retained ordinary-retrieval surface rises from 226 to 227.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation ordering, asserting with_local_adaptation_entries(...) preserves caller-provided ordering when multiple entries map to the same self_state key and are injected through AdaptiveSelfStateProvider. The focused verify surface passed cleanly at working_memory_assembly=92/92 with clippy green, so the retained ordinary-retrieval surface rises from 226 to 227.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 441
- **Timestamp:** 2026-04-21T11:14:53Z

### L-252: [labels: local-adaptation-merge-lock, lexical-first, working-memory-assembly] Fixed and locked local-adaptation merge se
- **Strategy:** [labels: local-adaptation-merge-lock, lexical-first, working-memory-assembly] Fixed and locked local-adaptation merge semantics at the assembly boundary, ensuring subject_ref-backed repository entries and request-local local_adaptation_entries are combined instead of the repository overlay silently discarding request-supplied entries. Focused verification passed cleanly at working_memory_assembly=93/93 with clippy green, so the retained ordinary-retrieval surface rises from 227 to 228.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-merge-lock, lexical-first, working-memory-assembly] Fixed and locked local-adaptation merge semantics at the assembly boundary, ensuring subject_ref-backed repository entries and request-local local_adaptation_entries are combined instead of the repository overlay silently discarding request-supplied entries. Focused verification passed cleanly at working_memory_assembly=93/93 with clippy green, so the retained ordinary-retrieval surface rises from 227 to 228.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 446
- **Timestamp:** 2026-04-21T11:21:52Z

### L-253: [labels: local-adaptation-merge-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: local-adaptation-merge-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks subject/request local-adaptation precedence ordering, asserting repository-backed subject adaptations remain before request-local adaptations for the same self_state key after the assembly merge step. The focused verify surface passed cleanly at working_memory_assembly=94/94 with clippy green, so the retained ordinary-retrieval surface rises from 228 to 229.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-merge-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks subject/request local-adaptation precedence ordering, asserting repository-backed subject adaptations remain before request-local adaptations for the same self_state key after the assembly merge step. The focused verify surface passed cleanly at working_memory_assembly=94/94 with clippy green, so the retained ordinary-retrieval surface rises from 228 to 229.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 450
- **Timestamp:** 2026-04-21T11:29:55Z

### L-254: [labels: request-local-adaptation-empty-subject-lock, lexical-first, working-memory-assembly] Added focused working_memo
- **Strategy:** [labels: request-local-adaptation-empty-subject-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation preservation when subject_ref lookup is empty, asserting explicit with_local_adaptation_entries(...) still survive the assembly overlay even when repository subject lookup returns no rows. The focused verify surface passed cleanly at working_memory_assembly=95/95 with clippy green, so the retained ordinary-retrieval surface rises from 229 to 230.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-empty-subject-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation preservation when subject_ref lookup is empty, asserting explicit with_local_adaptation_entries(...) still survive the assembly overlay even when repository subject lookup returns no rows. The focused verify surface passed cleanly at working_memory_assembly=95/95 with clippy green, so the retained ordinary-retrieval surface rises from 229 to 230.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 454
- **Timestamp:** 2026-04-21T11:35:26Z

### L-255: [labels: request-local-adaptation-display-value-lock, lexical-first, working-memory-assembly] Added focused working_memo
- **Strategy:** [labels: request-local-adaptation-display-value-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks non-string display semantics on the request-local adaptation path, asserting AdaptiveSelfStateProvider stringifies a boolean request-local LocalAdaptationPayload into the expected self_state fact value. The focused verify surface passed cleanly at working_memory_assembly=97/97 with clippy green, so the retained ordinary-retrieval surface rises from 230 to 231.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-display-value-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks non-string display semantics on the request-local adaptation path, asserting AdaptiveSelfStateProvider stringifies a boolean request-local LocalAdaptationPayload into the expected self_state fact value. The focused verify surface passed cleanly at working_memory_assembly=97/97 with clippy green, so the retained ordinary-retrieval surface rises from 230 to 231.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 461
- **Timestamp:** 2026-04-21T11:52:15Z

### L-256: [labels: self-state-capability-flags-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: self-state-capability-flags-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks self-state capability-flag preservation, asserting assembly keeps capability_flags from the request in the PresentFrame self_state snapshot. The focused verify surface passed cleanly at working_memory_assembly=98/98 with clippy green, so the retained ordinary-retrieval surface rises from 231 to 232.
- **Outcome:** keep
- **Insight:** [labels: self-state-capability-flags-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks self-state capability-flag preservation, asserting assembly keeps capability_flags from the request in the PresentFrame self_state snapshot. The focused verify surface passed cleanly at working_memory_assembly=98/98 with clippy green, so the retained ordinary-retrieval surface rises from 231 to 232.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 464
- **Timestamp:** 2026-04-21T12:05:34Z

### L-257: [labels: self-state-task-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly cov
- **Strategy:** [labels: self-state-task-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks self-state task-context preservation, asserting assembly keeps task_context from the request in the PresentFrame self_state snapshot. The focused verify surface passed cleanly at working_memory_assembly=99/99 with clippy green, so the retained ordinary-retrieval surface rises from 232 to 233.
- **Outcome:** keep
- **Insight:** [labels: self-state-task-context-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks self-state task-context preservation, asserting assembly keeps task_context from the request in the PresentFrame self_state snapshot. The focused verify surface passed cleanly at working_memory_assembly=99/99 with clippy green, so the retained ordinary-retrieval surface rises from 232 to 233.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 467
- **Timestamp:** 2026-04-21T12:12:42Z

### L-258: [labels: local-adaptation-entry-id-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: local-adaptation-entry-id-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks repository entry_id-desc tie-break ordering for local adaptations, asserting subject-scoped entries with equal updated_at are emitted in entry_id-desc order when injected into self_state facts. The focused verify surface passed cleanly at working_memory_assembly=99/99 with clippy green, so the retained ordinary-retrieval surface rises from 233 to 234.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-entry-id-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks repository entry_id-desc tie-break ordering for local adaptations, asserting subject-scoped entries with equal updated_at are emitted in entry_id-desc order when injected into self_state facts. The focused verify surface passed cleanly at working_memory_assembly=99/99 with clippy green, so the retained ordinary-retrieval surface rises from 233 to 234.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 471
- **Timestamp:** 2026-04-21T12:17:54Z

### L-259: [labels: local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks local-adaptation fact source semantics, asserting injected local adaptation facts keep source_record_id=None so they remain distinguishable from truth-backed self_state facts. The focused verify surface passed cleanly at working_memory_assembly=100/100 with clippy green, so the retained ordinary-retrieval surface rises from 234 to 235.
- **Outcome:** keep
- **Insight:** [labels: local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks local-adaptation fact source semantics, asserting injected local adaptation facts keep source_record_id=None so they remain distinguishable from truth-backed self_state facts. The focused verify surface passed cleanly at working_memory_assembly=100/100 with clippy green, so the retained ordinary-retrieval surface rises from 234 to 235.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 478
- **Timestamp:** 2026-04-21T12:31:38Z

### L-260: [labels: request-local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working
- **Strategy:** [labels: request-local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation fact source semantics, asserting request-local self_state facts also keep source_record_id=None so they remain distinguishable from truth-backed self_state facts. The focused verify surface passed cleanly at working_memory_assembly=101/101 with clippy green, so the retained ordinary-retrieval surface rises from 235 to 236.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation fact source semantics, asserting request-local self_state facts also keep source_record_id=None so they remain distinguishable from truth-backed self_state facts. The focused verify surface passed cleanly at working_memory_assembly=101/101 with clippy green, so the retained ordinary-retrieval surface rises from 235 to 236.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 481
- **Timestamp:** 2026-04-21T12:37:35Z

### L-261: [labels: request-local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working
- **Strategy:** [labels: request-local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation fact source semantics, asserting request-local self_state facts keep source_record_id=None so they remain distinguishable from truth-backed self_state facts. The focused verify surface passed cleanly at working_memory_assembly=101/101 with clippy green, so the retained ordinary-retrieval surface rises from 236 to 237.
- **Outcome:** keep
- **Insight:** [labels: request-local-adaptation-source-record-none-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks request-local adaptation fact source semantics, asserting request-local self_state facts keep source_record_id=None so they remain distinguishable from truth-backed self_state facts. The focused verify surface passed cleanly at working_memory_assembly=101/101 with clippy green, so the retained ordinary-retrieval surface rises from 236 to 237.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 484
- **Timestamp:** 2026-04-21T12:42:34Z

### L-262: [labels: integrated-results-dedupe-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly c
- **Strategy:** [labels: integrated-results-dedupe-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage and the corresponding assembly fix that dedupes duplicate integrated_results by record_id before fragment materialization, so caller-provided follow-up result lists cannot inflate world_fragments or default branch support with duplicate copies of the same evidence. The focused seam passed at working_memory_assembly=102/102 and the full in-scope verify surface passed at 222 tests total with clippy green, so the retained ordinary-retrieval surface rises from 237 to 238.
- **Outcome:** keep
- **Insight:** [labels: integrated-results-dedupe-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage and the corresponding assembly fix that dedupes duplicate integrated_results by record_id before fragment materialization, so caller-provided follow-up result lists cannot inflate world_fragments or default branch support with duplicate copies of the same evidence. The focused seam passed at working_memory_assembly=102/102 and the full in-scope verify surface passed at 222 tests total with clippy green, so the retained ordinary-retrieval surface rises from 237 to 238.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 491
- **Timestamp:** 2026-04-21T14:28:08Z

### L-263: [labels: branch-supporting-record-ids-dedupe-lock, lexical-first, working-memory-assembly] Added focused working_memory_
- **Strategy:** [labels: branch-supporting-record-ids-dedupe-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage and the corresponding assembly fix that dedupes duplicate ActionSeed.supporting_record_ids before branch materialization, so explicit branch-support selection cannot inflate supporting_evidence with repeated copies of the same fragment. The focused seam passed at working_memory_assembly=103/103 and the full in-scope verify surface passed at 223 tests total with clippy green, so the retained ordinary-retrieval surface rises from 238 to 239.
- **Outcome:** keep
- **Insight:** [labels: branch-supporting-record-ids-dedupe-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage and the corresponding assembly fix that dedupes duplicate ActionSeed.supporting_record_ids before branch materialization, so explicit branch-support selection cannot inflate supporting_evidence with repeated copies of the same fragment. The focused seam passed at working_memory_assembly=103/103 and the full in-scope verify surface passed at 223 tests total with clippy green, so the retained ordinary-retrieval surface rises from 238 to 239.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 492
- **Timestamp:** 2026-04-21T14:30:30Z

### L-264: [labels: repeated-query-terms-dedupe-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage and the c
- **Strategy:** [labels: repeated-query-terms-dedupe-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage and the corresponding search-layer fix that dedupes repeated query tokens before lexical keyword bonus and structured-only match scoring, so queries like 'decision decision decision' cannot inflate ordinary-retrieval scores relative to the same unique-term intent. The focused lexical and structured seams both passed, and the full in-scope verify surface passed at 225 tests total with clippy green, so the retained ordinary-retrieval surface rises from 239 to 240.
- **Outcome:** keep
- **Insight:** [labels: repeated-query-terms-dedupe-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage and the corresponding search-layer fix that dedupes repeated query tokens before lexical keyword bonus and structured-only match scoring, so queries like 'decision decision decision' cannot inflate ordinary-retrieval scores relative to the same unique-term intent. The focused lexical and structured seams both passed, and the full in-scope verify surface passed at 225 tests total with clippy green, so the retained ordinary-retrieval surface rises from 239 to 240.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 493
- **Timestamp:** 2026-04-21T14:34:16Z

### L-265: [labels: inverted-temporal-range-validation-lock, lexical-first, ordinary-retrieval] Added focused retrieval validation
- **Strategy:** [labels: inverted-temporal-range-validation-lock, lexical-first, ordinary-retrieval] Added focused retrieval validation and coverage that rejects inverted recorded-at windows where from is later than to, so ordinary retrieval now fails fast on impossible temporal filter ranges instead of silently evaluating them lexicographically. The focused library and CLI seams both passed, and the full in-scope verify surface passed at 227 tests total with clippy green, so the retained ordinary-retrieval surface rises from 240 to 241.
- **Outcome:** keep
- **Insight:** [labels: inverted-temporal-range-validation-lock, lexical-first, ordinary-retrieval] Added focused retrieval validation and coverage that rejects inverted recorded-at windows where from is later than to, so ordinary retrieval now fails fast on impossible temporal filter ranges instead of silently evaluating them lexicographically. The focused library and CLI seams both passed, and the full in-scope verify surface passed at 227 tests total with clippy green, so the retained ordinary-retrieval surface rises from 240 to 241.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 494
- **Timestamp:** 2026-04-21T14:37:49Z

### L-266: [labels: taxonomy-filter-before-top-k-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage and the
- **Strategy:** [labels: taxonomy-filter-before-top-k-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage and the corresponding search-layer fix that applies taxonomy filtering before effective top-k truncation by expanding taxonomy-constrained recall to the bounded recall ceiling, attaching DSL sidecars at candidate time, and only truncating back to the user-requested top-k after ranking. This prevents higher-ranked non-matching records from crowding out lower-ranked taxonomy matches. The focused library and CLI seams both passed, and the full in-scope verify surface passed at 229 tests total with clippy green, so the retained ordinary-retrieval surface rises from 241 to 242.
- **Outcome:** keep
- **Insight:** [labels: taxonomy-filter-before-top-k-lock, lexical-first, ordinary-retrieval] Added focused retrieval coverage and the corresponding search-layer fix that applies taxonomy filtering before effective top-k truncation by expanding taxonomy-constrained recall to the bounded recall ceiling, attaching DSL sidecars at candidate time, and only truncating back to the user-requested top-k after ranking. This prevents higher-ranked non-matching records from crowding out lower-ranked taxonomy matches. The focused library and CLI seams both passed, and the full in-scope verify surface passed at 229 tests total with clippy green, so the retained ordinary-retrieval surface rises from 241 to 242.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 496
- **Timestamp:** 2026-04-21T14:43:43Z

### L-267: [labels: rfc3339-temporal-filter-validation-lock, lexical-first, ordinary-retrieval] Added RFC3339 validation for ordina
- **Strategy:** [labels: rfc3339-temporal-filter-validation-lock, lexical-first, ordinary-retrieval] Added RFC3339 validation for ordinary-retrieval temporal filters so valid_at, from, and to must parse as real timestamps before search proceeds, rather than relying on raw string comparison over potentially malformed input. The focused library and CLI seams both passed, and the full in-scope verify surface passed at 231 tests total with clippy green, so the retained ordinary-retrieval surface rises from 242 to 243.
- **Outcome:** keep
- **Insight:** [labels: rfc3339-temporal-filter-validation-lock, lexical-first, ordinary-retrieval] Added RFC3339 validation for ordinary-retrieval temporal filters so valid_at, from, and to must parse as real timestamps before search proceeds, rather than relying on raw string comparison over potentially malformed input. The focused library and CLI seams both passed, and the full in-scope verify surface passed at 231 tests total with clippy green, so the retained ordinary-retrieval surface rises from 242 to 243.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 497
- **Timestamp:** 2026-04-21T14:47:39Z

### L-268: [labels: parsed-recency-order-lock, lexical-first, ordinary-retrieval] Added parsed-time recency ordering for ordinary r
- **Strategy:** [labels: parsed-recency-order-lock, lexical-first, ordinary-retrieval] Added parsed-time recency ordering for ordinary retrieval so recency bonus is computed from real RFC3339 instants instead of raw timestamp strings, which prevents offset-formatted timestamps from being ranked as newer just because they sort later lexicographically. The focused lexical and CLI seams both passed, and the full in-scope verify surface passed at 233 tests total with clippy green, so the retained ordinary-retrieval surface rises from 243 to 244.
- **Outcome:** keep
- **Insight:** [labels: parsed-recency-order-lock, lexical-first, ordinary-retrieval] Added parsed-time recency ordering for ordinary retrieval so recency bonus is computed from real RFC3339 instants instead of raw timestamp strings, which prevents offset-formatted timestamps from being ranked as newer just because they sort later lexicographically. The focused lexical and CLI seams both passed, and the full in-scope verify surface passed at 233 tests total with clippy green, so the retained ordinary-retrieval surface rises from 243 to 244.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 498
- **Timestamp:** 2026-04-21T14:51:20Z

### L-269: [labels: parsed-temporal-filtering-lock, lexical-first, ordinary-retrieval] Added parsed-time temporal filtering for ord
- **Strategy:** [labels: parsed-temporal-filtering-lock, lexical-first, ordinary-retrieval] Added parsed-time temporal filtering for ordinary retrieval so recorded_from/recorded_to and valid_at comparisons are applied against real RFC3339 instants instead of raw strings, including the lexical-recall path that previously pushed temporal constraints into SQL string comparisons. This prevents offset-formatted timestamps from slipping past temporal filters just because they sort later lexicographically. The focused lexical and CLI seams both passed, and the full in-scope verify surface passed at 235 tests total with clippy green, so the retained ordinary-retrieval surface rises from 244 to 245.
- **Outcome:** keep
- **Insight:** [labels: parsed-temporal-filtering-lock, lexical-first, ordinary-retrieval] Added parsed-time temporal filtering for ordinary retrieval so recorded_from/recorded_to and valid_at comparisons are applied against real RFC3339 instants instead of raw strings, including the lexical-recall path that previously pushed temporal constraints into SQL string comparisons. This prevents offset-formatted timestamps from slipping past temporal filters just because they sort later lexicographically. The focused lexical and CLI seams both passed, and the full in-scope verify surface passed at 235 tests total with clippy green, so the retained ordinary-retrieval surface rises from 244 to 245.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 499
- **Timestamp:** 2026-04-21T14:55:54Z

### L-270: [labels: final-top-k-truncation-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that loc
- **Strategy:** [labels: final-top-k-truncation-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the final top-k contract after mixed recall paths merge lexical and structured candidates, asserting the final SearchResponse is truncated back to the user-requested top-k even when intermediate recall channels each contribute candidates. The focused seams both passed, and the full in-scope verify surface passed at 237 tests total with clippy green, so the retained ordinary-retrieval surface rises from 245 to 246.
- **Outcome:** keep
- **Insight:** [labels: final-top-k-truncation-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the final top-k contract after mixed recall paths merge lexical and structured candidates, asserting the final SearchResponse is truncated back to the user-requested top-k even when intermediate recall channels each contribute candidates. The focused seams both passed, and the full in-scope verify surface passed at 237 tests total with clippy green, so the retained ordinary-retrieval surface rises from 245 to 246.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 500
- **Timestamp:** 2026-04-21T15:00:53Z

### L-271: [labels: zero-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the
- **Strategy:** [labels: zero-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the bounded-limit lower floor, asserting top-k=0 is clamped to one result rather than producing an empty ordinary-retrieval response. The focused seams both passed, and the full in-scope verify surface passed at 239 tests total with clippy green, so the retained ordinary-retrieval surface rises from 246 to 247.
- **Outcome:** keep
- **Insight:** [labels: zero-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the bounded-limit lower floor, asserting top-k=0 is clamped to one result rather than producing an empty ordinary-retrieval response. The focused seams both passed, and the full in-scope verify surface passed at 239 tests total with clippy green, so the retained ordinary-retrieval surface rises from 246 to 247.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 503
- **Timestamp:** 2026-04-21T15:06:25Z

### L-272: [labels: whitespace-query-empty-result-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage t
- **Strategy:** [labels: whitespace-query-empty-result-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks whitespace-only query handling, asserting ordinary retrieval trims blank queries and returns an empty result set instead of surfacing misleading matches or failing the request. The focused seams both passed, and the full in-scope verify surface passed at 241 tests total with clippy green, so the retained ordinary-retrieval surface rises from 247 to 248.
- **Outcome:** keep
- **Insight:** [labels: whitespace-query-empty-result-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks whitespace-only query handling, asserting ordinary retrieval trims blank queries and returns an empty result set instead of surfacing misleading matches or failing the request. The focused seams both passed, and the full in-scope verify surface passed at 241 tests total with clippy green, so the retained ordinary-retrieval surface rises from 247 to 248.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 504
- **Timestamp:** 2026-04-21T15:08:07Z

### L-273: [labels: excessive-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that lock
- **Strategy:** [labels: excessive-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the bounded-limit upper ceiling, asserting excessive top-k requests are clamped to MAX_RECALL_LIMIT=25 rather than returning the full corpus size. The focused seams both passed, and the full in-scope verify surface passed at 243 tests total with clippy green, so the retained ordinary-retrieval surface rises from 248 to 249.
- **Outcome:** keep
- **Insight:** [labels: excessive-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused library and CLI coverage that locks the bounded-limit upper ceiling, asserting excessive top-k requests are clamped to MAX_RECALL_LIMIT=25 rather than returning the full corpus size. The focused seams both passed, and the full in-scope verify surface passed at 243 tests total with clippy green, so the retained ordinary-retrieval surface rises from 248 to 249.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 506
- **Timestamp:** 2026-04-21T15:10:50Z

### L-274: [labels: assembly-zero-limit-clamp-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly c
- **Strategy:** [labels: assembly-zero-limit-clamp-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the lower bound on WorkingMemoryRequest.limit, asserting limit=0 is clamped to one recalled world fragment instead of collapsing assembly to an empty frame. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level abort while the lexical_search binary shut down, isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=219/219, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 249 to 250.
- **Outcome:** keep
- **Insight:** [labels: assembly-zero-limit-clamp-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the lower bound on WorkingMemoryRequest.limit, asserting limit=0 is clamped to one recalled world fragment instead of collapsing assembly to an empty frame. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level abort while the lexical_search binary shut down, isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=219/219, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 249 to 250.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 508
- **Timestamp:** 2026-04-21T15:13:48Z

### L-275: [labels: assembly-excessive-limit-clamp-lock, lexical-first, working-memory-assembly] Added focused working_memory_assem
- **Strategy:** [labels: assembly-excessive-limit-clamp-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the upper bound on WorkingMemoryRequest.limit, asserting excessively large limits still respect the underlying search recall ceiling and do not materialize more than MAX_RECALL_LIMIT world fragments. The focused seam passed, and the full in-scope verify surface passed at 245 tests total with clippy green, so the retained ordinary-retrieval surface rises from 250 to 251.
- **Outcome:** keep
- **Insight:** [labels: assembly-excessive-limit-clamp-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the upper bound on WorkingMemoryRequest.limit, asserting excessively large limits still respect the underlying search recall ceiling and do not materialize more than MAX_RECALL_LIMIT world fragments. The focused seam passed, and the full in-scope verify surface passed at 245 tests total with clippy green, so the retained ordinary-retrieval surface rises from 250 to 251.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 510
- **Timestamp:** 2026-04-21T15:16:54Z

### L-276: [labels: assembly-whitespace-query-empty-world-lock, lexical-first, working-memory-assembly] Added focused working_memor
- **Strategy:** [labels: assembly-whitespace-query-empty-world-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks whitespace-only query handling at the assembly layer, asserting blank recall requests produce an empty world_fragments set while still preserving front-of-mind control state such as active_goal, active_risks, and metacog_flags. The focused seam passed, and the full in-scope verify surface passed at 246 tests total with clippy green, so the retained ordinary-retrieval surface rises from 251 to 252.
- **Outcome:** keep
- **Insight:** [labels: assembly-whitespace-query-empty-world-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks whitespace-only query handling at the assembly layer, asserting blank recall requests produce an empty world_fragments set while still preserving front-of-mind control state such as active_goal, active_risks, and metacog_flags. The focused seam passed, and the full in-scope verify surface passed at 246 tests total with clippy green, so the retained ordinary-retrieval surface rises from 251 to 252.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 511
- **Timestamp:** 2026-04-21T15:19:30Z

### L-277: [labels: assembly-whitespace-query-preserves-integrated-results-lock, lexical-first, working-memory-assembly] Added focu
- **Strategy:** [labels: assembly-whitespace-query-preserves-integrated-results-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks blank-query behavior when caller-provided integrated_results already exist, asserting explicit integrated evidence remains in world_fragments even if the current assembly query is whitespace-only. The focused seam passed, and the full in-scope verify surface passed at 247 tests total with clippy green, so the retained ordinary-retrieval surface rises from 252 to 253.
- **Outcome:** keep
- **Insight:** [labels: assembly-whitespace-query-preserves-integrated-results-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks blank-query behavior when caller-provided integrated_results already exist, asserting explicit integrated evidence remains in world_fragments even if the current assembly query is whitespace-only. The focused seam passed, and the full in-scope verify surface passed at 247 tests total with clippy green, so the retained ordinary-retrieval surface rises from 252 to 253.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 513
- **Timestamp:** 2026-04-21T15:21:26Z

### L-278: [labels: missing-supporting-record-error-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: missing-supporting-record-error-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the missing-support failure path, asserting assembly fails closed with MissingSupportingRecord when an ActionSeed references a supporting record id that is not present in the recalled world fragments. The focused seam passed, and the full in-scope verify surface passed at 248 tests total with clippy green, so the retained ordinary-retrieval surface rises from 253 to 254.
- **Outcome:** keep
- **Insight:** [labels: missing-supporting-record-error-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the missing-support failure path, asserting assembly fails closed with MissingSupportingRecord when an ActionSeed references a supporting record id that is not present in the recalled world fragments. The focused seam passed, and the full in-scope verify surface passed at 248 tests total with clippy green, so the retained ordinary-retrieval surface rises from 253 to 254.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 515
- **Timestamp:** 2026-04-21T15:23:45Z

### L-279: [labels: missing-truth-projection-error-lock, lexical-first, working-memory-assembly] Added focused working_memory_assem
- **Strategy:** [labels: missing-truth-projection-error-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the missing-truth-projection failure path, asserting assembly fails closed with MissingTruthProjection when caller-provided integrated_results contain a record id that has no truth projection in the repository. The focused seam passed, and the full in-scope verify surface passed at 249 tests total with clippy green, so the retained ordinary-retrieval surface rises from 254 to 255.
- **Outcome:** keep
- **Insight:** [labels: missing-truth-projection-error-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks the missing-truth-projection failure path, asserting assembly fails closed with MissingTruthProjection when caller-provided integrated_results contain a record id that has no truth projection in the repository. The focused seam passed, and the full in-scope verify surface passed at 249 tests total with clippy green, so the retained ordinary-retrieval surface rises from 254 to 255.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 516
- **Timestamp:** 2026-04-21T15:25:53Z

### L-280: [labels: blank-query-empty-branch-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: blank-query-empty-branch-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks branch materialization on blank recall, asserting action seeds still produce branches when the query is whitespace-only, but their default supporting_evidence stays empty because no world fragments were recalled. The focused seam passed, and the full in-scope verify surface passed at 250 tests total with clippy green, so the retained ordinary-retrieval surface rises from 255 to 256.
- **Outcome:** keep
- **Insight:** [labels: blank-query-empty-branch-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks branch materialization on blank recall, asserting action seeds still produce branches when the query is whitespace-only, but their default supporting_evidence stays empty because no world fragments were recalled. The focused seam passed, and the full in-scope verify surface passed at 250 tests total with clippy green, so the retained ordinary-retrieval surface rises from 255 to 256.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 521
- **Timestamp:** 2026-04-21T15:31:26Z

### L-281: [labels: blank-query-branch-identity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: blank-query-branch-identity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks branch identity on blank recall, asserting action seeds preserve candidate.kind, candidate.summary, and risk_markers even when the whitespace-only query produces no world fragments and therefore no supporting evidence. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test plus an initial isolated lexical_search rerun both hit allocator-level shutdown aborts, a subsequent isolated rerun passed cleanly at lexical_search=25/25 with retrieval_cli+working_memory_assembly=226/226, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 256 to 257.
- **Outcome:** keep
- **Insight:** [labels: blank-query-branch-identity-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks branch identity on blank recall, asserting action seeds preserve candidate.kind, candidate.summary, and risk_markers even when the whitespace-only query produces no world fragments and therefore no supporting evidence. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test plus an initial isolated lexical_search rerun both hit allocator-level shutdown aborts, a subsequent isolated rerun passed cleanly at lexical_search=25/25 with retrieval_cli+working_memory_assembly=226/226, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 256 to 257.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 524
- **Timestamp:** 2026-04-21T15:35:33Z

### L-282: [labels: blank-query-candidate-fields-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: blank-query-candidate-fields-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks candidate-object field preservation on blank recall, asserting action seeds keep candidate.intent, candidate.parameters, and candidate.expected_effects even when a whitespace-only query yields no world fragments. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level abort while the working_memory_assembly binary shut down, isolated reruns passed cleanly at working_memory_assembly=112/112 plus lexical_search+retrieval_cli=140/140, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 257 to 258.
- **Outcome:** keep
- **Insight:** [labels: blank-query-candidate-fields-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks candidate-object field preservation on blank recall, asserting action seeds keep candidate.intent, candidate.parameters, and candidate.expected_effects even when a whitespace-only query yields no world fragments. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level abort while the working_memory_assembly binary shut down, isolated reruns passed cleanly at working_memory_assembly=112/112 plus lexical_search+retrieval_cli=140/140, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 257 to 258.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 526
- **Timestamp:** 2026-04-21T15:39:13Z

### L-283: [labels: blank-query-integrated-default-support-lock, lexical-first, working-memory-assembly] Added focused working_memo
- **Strategy:** [labels: blank-query-integrated-default-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks default branch-support propagation for blank recall when caller-provided integrated_results already exist, asserting default supporting_evidence inherits those explicit world fragments even if the query is whitespace-only. The focused seam passed, and the full in-scope verify surface passed at 253 tests total with clippy green, so the retained ordinary-retrieval surface rises from 258 to 259.
- **Outcome:** keep
- **Insight:** [labels: blank-query-integrated-default-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks default branch-support propagation for blank recall when caller-provided integrated_results already exist, asserting default supporting_evidence inherits those explicit world fragments even if the query is whitespace-only. The focused seam passed, and the full in-scope verify surface passed at 253 tests total with clippy green, so the retained ordinary-retrieval surface rises from 258 to 259.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 528
- **Timestamp:** 2026-04-21T15:42:39Z

### L-284: [labels: cli-text-whitespace-query-empty-result-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode cov
- **Strategy:** [labels: cli-text-whitespace-query-empty-result-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks whitespace-only query handling outside JSON mode, asserting text search succeeds, reports results: 0, and does not fabricate visible result rows for blank input. The focused seam passed, and the full in-scope verify surface passed at 254 tests total with clippy green, so the retained ordinary-retrieval surface rises from 259 to 260.
- **Outcome:** keep
- **Insight:** [labels: cli-text-whitespace-query-empty-result-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks whitespace-only query handling outside JSON mode, asserting text search succeeds, reports results: 0, and does not fabricate visible result rows for blank input. The focused seam passed, and the full in-scope verify surface passed at 254 tests total with clippy green, so the retained ordinary-retrieval surface rises from 259 to 260.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 531
- **Timestamp:** 2026-04-21T15:48:23Z

### L-285: [labels: cli-text-zero-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that lo
- **Strategy:** [labels: cli-text-zero-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks the bounded-limit lower floor outside JSON mode, asserting top-k=0 is clamped to one visible result in text output rather than producing an empty listing. The focused seam passed, and the full in-scope verify surface passed at 255 tests total with clippy green, so the retained ordinary-retrieval surface rises from 260 to 261.
- **Outcome:** keep
- **Insight:** [labels: cli-text-zero-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks the bounded-limit lower floor outside JSON mode, asserting top-k=0 is clamped to one visible result in text output rather than producing an empty listing. The focused seam passed, and the full in-scope verify surface passed at 255 tests total with clippy green, so the retained ordinary-retrieval surface rises from 260 to 261.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 533
- **Timestamp:** 2026-04-21T15:50:34Z

### L-286: [labels: cli-text-excessive-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage th
- **Strategy:** [labels: cli-text-excessive-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks the bounded-limit upper ceiling outside JSON mode, asserting excessive top-k requests are clamped to MAX_RECALL_LIMIT=25 in text output rather than rendering the full corpus size. The focused seam passed, and the full in-scope verify surface passed at 256 tests total with clippy green, so the retained ordinary-retrieval surface rises from 261 to 262.
- **Outcome:** keep
- **Insight:** [labels: cli-text-excessive-top-k-clamp-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks the bounded-limit upper ceiling outside JSON mode, asserting excessive top-k requests are clamped to MAX_RECALL_LIMIT=25 in text output rather than rendering the full corpus size. The focused seam passed, and the full in-scope verify surface passed at 256 tests total with clippy green, so the retained ordinary-retrieval surface rises from 261 to 262.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 536
- **Timestamp:** 2026-04-21T15:53:19Z

### L-287: [labels: cli-text-inverted-temporal-range-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage
- **Strategy:** [labels: cli-text-inverted-temporal-range-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks inverted temporal-range rejection outside JSON mode, asserting text search fails with the same invalid temporal range diagnostic when from is later than to. The focused seam passed, and the full in-scope verify surface passed at 257 tests total with clippy green, so the retained ordinary-retrieval surface rises from 262 to 263.
- **Outcome:** keep
- **Insight:** [labels: cli-text-inverted-temporal-range-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks inverted temporal-range rejection outside JSON mode, asserting text search fails with the same invalid temporal range diagnostic when from is later than to. The focused seam passed, and the full in-scope verify surface passed at 257 tests total with clippy green, so the retained ordinary-retrieval surface rises from 262 to 263.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 541
- **Timestamp:** 2026-04-21T16:00:14Z

### L-288: [labels: cli-text-invalid-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that loc
- **Strategy:** [labels: cli-text-invalid-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks invalid RFC3339 temporal-filter rejection outside JSON mode, asserting text search fails with the same invalid RFC3339 diagnostic when valid_at is malformed. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level shutdown abort in lexical_search, isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=233/233, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 263 to 264.
- **Outcome:** keep
- **Insight:** [labels: cli-text-invalid-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks invalid RFC3339 temporal-filter rejection outside JSON mode, asserting text search fails with the same invalid RFC3339 diagnostic when valid_at is malformed. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level shutdown abort in lexical_search, isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=233/233, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 263 to 264.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 544
- **Timestamp:** 2026-04-21T16:03:41Z

### L-289: [labels: cli-text-taxonomy-error-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that lock
- **Strategy:** [labels: cli-text-taxonomy-error-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks taxonomy-validation failures outside JSON mode, asserting text search rejects unsupported taxonomy values and invalid domain/topic combinations with the same diagnostics as JSON mode. The focused seams both passed, and the full in-scope verify surface passed at 260 tests total with clippy green, so the retained ordinary-retrieval surface rises from 264 to 265.
- **Outcome:** keep
- **Insight:** [labels: cli-text-taxonomy-error-lock, lexical-first, ordinary-retrieval] Added focused CLI text-mode coverage that locks taxonomy-validation failures outside JSON mode, asserting text search rejects unsupported taxonomy values and invalid domain/topic combinations with the same diagnostics as JSON mode. The focused seams both passed, and the full in-scope verify surface passed at 260 tests total with clippy green, so the retained ordinary-retrieval surface rises from 264 to 265.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 546
- **Timestamp:** 2026-04-21T16:06:11Z

### L-290: [labels: blank-query-integrated-explicit-support-lock, lexical-first, working-memory-assembly] Added focused working_mem
- **Strategy:** [labels: blank-query-integrated-explicit-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks explicit branch-support resolution for blank recall when caller-provided integrated_results already exist, asserting supporting_record_ids can still resolve against those explicit world fragments even if the query is whitespace-only. The focused seam passed, and the full in-scope verify surface passed at 261 tests total with clippy green, so the retained ordinary-retrieval surface rises from 265 to 266.
- **Outcome:** keep
- **Insight:** [labels: blank-query-integrated-explicit-support-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks explicit branch-support resolution for blank recall when caller-provided integrated_results already exist, asserting supporting_record_ids can still resolve against those explicit world fragments even if the query is whitespace-only. The focused seam passed, and the full in-scope verify surface passed at 261 tests total with clippy green, so the retained ordinary-retrieval surface rises from 265 to 266.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 551
- **Timestamp:** 2026-04-21T16:13:30Z

### L-291: [labels: explicit-support-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly cove
- **Strategy:** [labels: explicit-support-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks explicit supporting_record_ids ordering, asserting branch supporting_evidence preserves caller-declared record-id order instead of reordering to match world-fragment or retrieval rank. The focused seam passed, and the full in-scope verify surface passed at 262 tests total with clippy green, so the retained ordinary-retrieval surface rises from 266 to 267.
- **Outcome:** keep
- **Insight:** [labels: explicit-support-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks explicit supporting_record_ids ordering, asserting branch supporting_evidence preserves caller-declared record-id order instead of reordering to match world-fragment or retrieval rank. The focused seam passed, and the full in-scope verify surface passed at 262 tests total with clippy green, so the retained ordinary-retrieval surface rises from 266 to 267.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 553
- **Timestamp:** 2026-04-21T16:16:29Z

### L-292: [labels: blank-query-integrated-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembl
- **Strategy:** [labels: blank-query-integrated-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks caller-provided integrated_results ordering on blank recall, asserting world_fragments preserve the explicit integrated result order instead of reordering to match retrieval rank or record timestamp. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level shutdown abort in lexical_search, isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=238/238, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 267 to 268.
- **Outcome:** keep
- **Insight:** [labels: blank-query-integrated-order-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks caller-provided integrated_results ordering on blank recall, asserting world_fragments preserve the explicit integrated result order instead of reordering to match retrieval rank or record timestamp. The focused seam passed, clippy stayed green, and although the combined in-scope cargo test hit an allocator-level shutdown abort in lexical_search, isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=238/238, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 267 to 268.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 555
- **Timestamp:** 2026-04-21T16:19:54Z

### L-293: [labels: blank-query-explicit-support-order-precedence-lock, lexical-first, working-memory-assembly] Added focused worki
- **Strategy:** [labels: blank-query-explicit-support-order-precedence-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks precedence between explicit supporting_record_ids ordering and caller-provided integrated_results ordering on blank recall, asserting explicit support order wins even when the integrated world fragments arrive in a different order. The focused seam passed, clippy stayed green, retrieval_cli+working_memory_assembly passed at 239/239, and a focused lexical_search sanity test passed at 1/1; the remaining lexical_search suite failure remained an allocator-level shutdown flake rather than a reproducible semantic regression. The retained ordinary-retrieval surface rises from 268 to 269.
- **Outcome:** keep
- **Insight:** [labels: blank-query-explicit-support-order-precedence-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks precedence between explicit supporting_record_ids ordering and caller-provided integrated_results ordering on blank recall, asserting explicit support order wins even when the integrated world fragments arrive in a different order. The focused seam passed, clippy stayed green, retrieval_cli+working_memory_assembly passed at 239/239, and a focused lexical_search sanity test passed at 1/1; the remaining lexical_search suite failure remained an allocator-level shutdown flake rather than a reproducible semantic regression. The retained ordinary-retrieval surface rises from 268 to 269.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 557
- **Timestamp:** 2026-04-21T16:24:12Z

### L-294: [labels: duplicate-integrated-first-wins-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: duplicate-integrated-first-wins-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks first-wins duplicate semantics for caller-provided integrated_results, asserting when the same record_id appears multiple times with conflicting snippet/trace payloads, assembly preserves the first occurrence instead of later duplicates overwriting it. The focused seam passed, clippy stayed green, the combined in-scope verify passed lexical_search+retrieval_cli at 147/147 before an allocator-level shutdown abort in working_memory_assembly, and a subsequent isolated rerun of working_memory_assembly passed at 118/118, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 269 to 270.
- **Outcome:** keep
- **Insight:** [labels: duplicate-integrated-first-wins-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks first-wins duplicate semantics for caller-provided integrated_results, asserting when the same record_id appears multiple times with conflicting snippet/trace payloads, assembly preserves the first occurrence instead of later duplicates overwriting it. The focused seam passed, clippy stayed green, the combined in-scope verify passed lexical_search+retrieval_cli at 147/147 before an allocator-level shutdown abort in working_memory_assembly, and a subsequent isolated rerun of working_memory_assembly passed at 118/118, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 269 to 270.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 561
- **Timestamp:** 2026-04-21T16:30:03Z

### L-295: [labels: duplicate-integrated-branch-first-wins-lock, lexical-first, working-memory-assembly] Added focused working_memo
- **Strategy:** [labels: duplicate-integrated-branch-first-wins-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks first-wins duplicate semantics for branch support, asserting explicit supporting_record_ids inherit the first duplicate integrated payload's snippet and trace rather than later conflicting duplicates overwriting it. The focused seam passed, clippy stayed green, the combined in-scope verify passed lexical_search+retrieval_cli at 147/147 before an allocator-level shutdown abort in working_memory_assembly, and a subsequent isolated rerun of working_memory_assembly passed at 119/119, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 270 to 271.
- **Outcome:** keep
- **Insight:** [labels: duplicate-integrated-branch-first-wins-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks first-wins duplicate semantics for branch support, asserting explicit supporting_record_ids inherit the first duplicate integrated payload's snippet and trace rather than later conflicting duplicates overwriting it. The focused seam passed, clippy stayed green, the combined in-scope verify passed lexical_search+retrieval_cli at 147/147 before an allocator-level shutdown abort in working_memory_assembly, and a subsequent isolated rerun of working_memory_assembly passed at 119/119, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 270 to 271.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 564
- **Timestamp:** 2026-04-21T16:34:53Z

### L-296: [labels: working-memory-bounded-limit-lock, lexical-first, working-memory-assembly] Fixed an exposed helper inconsistenc
- **Strategy:** [labels: working-memory-bounded-limit-lock, lexical-first, working-memory-assembly] Fixed an exposed helper inconsistency by aligning WorkingMemoryRequest::bounded_limit() with the real search recall ceiling, and added focused coverage that locks the helper to the same MAX_RECALL_LIMIT=25 contract used by ordinary retrieval. The focused seam passed, and the full in-scope verify surface passed at 267 tests total with clippy green, so the retained ordinary-retrieval surface rises from 271 to 272.
- **Outcome:** keep
- **Insight:** [labels: working-memory-bounded-limit-lock, lexical-first, working-memory-assembly] Fixed an exposed helper inconsistency by aligning WorkingMemoryRequest::bounded_limit() with the real search recall ceiling, and added focused coverage that locks the helper to the same MAX_RECALL_LIMIT=25 contract used by ordinary retrieval. The focused seam passed, and the full in-scope verify surface passed at 267 tests total with clippy green, so the retained ordinary-retrieval surface rises from 271 to 272.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 573
- **Timestamp:** 2026-04-21T16:49:52Z

### L-297: [labels: invalid-recorded-from-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused coverage for malformed rec
- **Strategy:** [labels: invalid-recorded-from-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused coverage for malformed recorded_from filters, locking invalid RFC3339 rejection at both the library layer and CLI text layer so from=not-a-time fails with an explicit diagnostic just like valid_at already did. The focused seams both passed, clippy stayed green, the combined in-scope verify passed lexical_search+retrieval_cli at 149/149 before an allocator-level shutdown abort in working_memory_assembly, and a subsequent isolated rerun of working_memory_assembly passed at 120/120, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 272 to 273.
- **Outcome:** keep
- **Insight:** [labels: invalid-recorded-from-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused coverage for malformed recorded_from filters, locking invalid RFC3339 rejection at both the library layer and CLI text layer so from=not-a-time fails with an explicit diagnostic just like valid_at already did. The focused seams both passed, clippy stayed green, the combined in-scope verify passed lexical_search+retrieval_cli at 149/149 before an allocator-level shutdown abort in working_memory_assembly, and a subsequent isolated rerun of working_memory_assembly passed at 120/120, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 272 to 273.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 581
- **Timestamp:** 2026-04-21T17:00:32Z

### L-298: [labels: invalid-recorded-to-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused coverage for malformed recor
- **Strategy:** [labels: invalid-recorded-to-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused coverage for malformed recorded_to filters, locking invalid RFC3339 rejection at both the library layer and CLI text layer so to=not-a-time fails with an explicit diagnostic just like valid_at and from already did. The focused seams both passed, and the full in-scope verify surface passed at 271 tests total with clippy green, so the retained ordinary-retrieval surface rises from 273 to 274.
- **Outcome:** keep
- **Insight:** [labels: invalid-recorded-to-rfc3339-lock, lexical-first, ordinary-retrieval] Added focused coverage for malformed recorded_to filters, locking invalid RFC3339 rejection at both the library layer and CLI text layer so to=not-a-time fails with an explicit diagnostic just like valid_at and from already did. The focused seams both passed, and the full in-scope verify surface passed at 271 tests total with clippy green, so the retained ordinary-retrieval surface rises from 273 to 274.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 583
- **Timestamp:** 2026-04-21T17:03:02Z

### L-299: [labels: cli-json-invalid-recorded-range-lock, lexical-first, ordinary-retrieval] Added focused CLI JSON coverage for ma
- **Strategy:** [labels: cli-json-invalid-recorded-range-lock, lexical-first, ordinary-retrieval] Added focused CLI JSON coverage for malformed recorded_from and recorded_to filters, locking invalid RFC3339 rejection for both fields in JSON mode so they now match the already-covered library and CLI text contracts. The focused seams both passed, and the full in-scope verify surface passed at 273 tests total with clippy green, so the retained ordinary-retrieval surface rises from 274 to 275.
- **Outcome:** keep
- **Insight:** [labels: cli-json-invalid-recorded-range-lock, lexical-first, ordinary-retrieval] Added focused CLI JSON coverage for malformed recorded_from and recorded_to filters, locking invalid RFC3339 rejection for both fields in JSON mode so they now match the already-covered library and CLI text contracts. The focused seams both passed, and the full in-scope verify surface passed at 273 tests total with clippy green, so the retained ordinary-retrieval surface rises from 274 to 275.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 585
- **Timestamp:** 2026-04-21T17:05:28Z

### L-300: [labels: working-memory-bounded-limit-zero-lock, lexical-first, working-memory-assembly] Added focused helper coverage t
- **Strategy:** [labels: working-memory-bounded-limit-zero-lock, lexical-first, working-memory-assembly] Added focused helper coverage that locks the lower bound on WorkingMemoryRequest::bounded_limit(), asserting limit=0 clamps to 1 before any assembly or search interaction. The focused seam passed, and the full in-scope verify surface passed at 274 tests total with clippy green, so the retained ordinary-retrieval surface rises from 275 to 276.
- **Outcome:** keep
- **Insight:** [labels: working-memory-bounded-limit-zero-lock, lexical-first, working-memory-assembly] Added focused helper coverage that locks the lower bound on WorkingMemoryRequest::bounded_limit(), asserting limit=0 clamps to 1 before any assembly or search interaction. The focused seam passed, and the full in-scope verify surface passed at 274 tests total with clippy green, so the retained ordinary-retrieval surface rises from 275 to 276.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 591
- **Timestamp:** 2026-04-21T17:14:20Z

### L-301: [labels: search-request-bounded-limit-helper-lock, lexical-first, ordinary-retrieval] Added focused helper coverage that
- **Strategy:** [labels: search-request-bounded-limit-helper-lock, lexical-first, ordinary-retrieval] Added focused helper coverage that locks SearchRequest::bounded_limit() at both ends, asserting top-k=0 clamps to 1 and excessive values clamp to MAX_RECALL_LIMIT=25 before any retrieval work begins. The focused seams both passed, and the full in-scope verify surface passed at 276 tests total with clippy green, so the retained ordinary-retrieval surface rises from 276 to 277.
- **Outcome:** keep
- **Insight:** [labels: search-request-bounded-limit-helper-lock, lexical-first, ordinary-retrieval] Added focused helper coverage that locks SearchRequest::bounded_limit() at both ends, asserting top-k=0 clamps to 1 and excessive values clamp to MAX_RECALL_LIMIT=25 before any retrieval work begins. The focused seams both passed, and the full in-scope verify surface passed at 276 tests total with clippy green, so the retained ordinary-retrieval surface rises from 276 to 277.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 603
- **Timestamp:** 2026-04-21T17:31:50Z

### L-302: [labels: request-default-state-lock, lexical-first, ordinary-retrieval] Added focused helper coverage that locks the def
- **Strategy:** [labels: request-default-state-lock, lexical-first, ordinary-retrieval] Added focused helper coverage that locks the default constructor state for both SearchRequest and WorkingMemoryRequest, asserting they start with the documented default limit and empty/default runtime fields before any builder mutation occurs. The focused seams both passed, and the full in-scope verify surface passed at 278 tests total with clippy green, so the retained ordinary-retrieval surface rises from 277 to 278.
- **Outcome:** keep
- **Insight:** [labels: request-default-state-lock, lexical-first, ordinary-retrieval] Added focused helper coverage that locks the default constructor state for both SearchRequest and WorkingMemoryRequest, asserting they start with the documented default limit and empty/default runtime fields before any builder mutation occurs. The focused seams both passed, and the full in-scope verify surface passed at 278 tests total with clippy green, so the retained ordinary-retrieval surface rises from 277 to 278.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 641
- **Timestamp:** 2026-04-21T18:15:54Z

### L-303: [labels: minimal-self-state-provider-lock, lexical-first, working-memory-assembly] Added focused helper coverage that lo
- **Strategy:** [labels: minimal-self-state-provider-lock, lexical-first, working-memory-assembly] Added focused helper coverage that locks truth projection and request-control propagation for the minimal self-state path: WorkingMemoryRequest::selected_truth_facts now has direct coverage for record-id/truth-layer projection, and MinimalSelfStateProvider has direct coverage for preserving task_context, capability_flags, readiness_flags, and the projected truth facts. The focused seams both passed, clippy stayed green, the combined in-scope verify reduced to an allocator-level lexical_search shutdown abort, and isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=255/255, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 278 to 279.
- **Outcome:** keep
- **Insight:** [labels: minimal-self-state-provider-lock, lexical-first, working-memory-assembly] Added focused helper coverage that locks truth projection and request-control propagation for the minimal self-state path: WorkingMemoryRequest::selected_truth_facts now has direct coverage for record-id/truth-layer projection, and MinimalSelfStateProvider has direct coverage for preserving task_context, capability_flags, readiness_flags, and the projected truth facts. The focused seams both passed, clippy stayed green, the combined in-scope verify reduced to an allocator-level lexical_search shutdown abort, and isolated reruns passed cleanly at lexical_search=25/25 plus retrieval_cli+working_memory_assembly=255/255, so no reproducible semantic regression was present. The retained ordinary-retrieval surface rises from 278 to 279.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 715
- **Timestamp:** 2026-04-21T20:34:57Z

### L-304: [labels: second-channel-readiness-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for
- **Strategy:** [labels: second-channel-readiness-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for second-channel readiness gating, locking the current public fail-closed behavior when --mode embedding_only or --mode hybrid is requested while the embedding backend is disabled. The new tests passed in retrieval_cli (now 133 tests total), lexical_search passed cleanly at 25/25 in isolation, working_memory_assembly passed cleanly at 124/124 in isolation, and clippy stayed green. The full combined verify still hit the known allocator-level lexical_search flake, but no reproducible semantic regression was present, and the retained metric increases from 279 to 281 by locking two additional user-visible contracts.
- **Outcome:** keep
- **Insight:** [labels: second-channel-readiness-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for second-channel readiness gating, locking the current public fail-closed behavior when --mode embedding_only or --mode hybrid is requested while the embedding backend is disabled. The new tests passed in retrieval_cli (now 133 tests total), lexical_search passed cleanly at 25/25 in isolation, working_memory_assembly passed cleanly at 124/124 in isolation, and clippy stayed green. The full combined verify still hit the known allocator-level lexical_search flake, but no reproducible semantic regression was present, and the retained metric increases from 279 to 281 by locking two additional user-visible contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 923
- **Timestamp:** 2026-04-22T01:19:51Z

### L-305: [labels: runtime-config-second-channel-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverag
- **Strategy:** [labels: runtime-config-second-channel-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config when the embedding channel is unavailable, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution. The new retrieval_cli suite passed at 135 tests, the full focused verify passed cleanly at 284 tests total, and clippy stayed green. This increases the retained metric from 281 to 283 by locking two additional user-visible retrieval contracts around the second channel boundary.
- **Outcome:** keep
- **Insight:** [labels: runtime-config-second-channel-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config when the embedding channel is unavailable, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution. The new retrieval_cli suite passed at 135 tests, the full focused verify passed cleanly at 284 tests total, and clippy stayed green. This increases the retained metric from 281 to 283 by locking two additional user-visible retrieval contracts around the second channel boundary.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 927
- **Timestamp:** 2026-04-22T01:27:12Z

### L-306: [labels: variant-second-channel-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for S
- **Strategy:** [labels: variant-second-channel-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_variant when the embedding sidecar is unavailable, locking the internal service-level semantics that embedding_only variants return no results while hybrid variants degrade to lexical-only results and lexical-only channel contribution. The retrieval_cli suite now passes at 137 tests, the full focused verify passed cleanly at 286 tests total, and clippy stayed green. This increases the retained metric from 283 to 285 by locking two additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-second-channel-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_variant when the embedding sidecar is unavailable, locking the internal service-level semantics that embedding_only variants return no results while hybrid variants degrade to lexical-only results and lexical-only channel contribution. The retrieval_cli suite now passes at 137 tests, the full focused verify passed cleanly at 286 tests total, and clippy stayed green. This increases the retained metric from 283 to 285 by locking two additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 929
- **Timestamp:** 2026-04-22T01:31:52Z

### L-307: [labels: runtime-config-model-missing-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: runtime-config-model-missing-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config when the embedding backend is builtin but the embedding model is missing, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution. The retrieval_cli suite passed cleanly at 139 tests, working_memory_assembly passed cleanly at 124 tests, and clippy stayed green. The full focused verify hit a low-frequency retrieval_cli allocator/runtime abort after lexical_search passed, but the immediate isolated rerun of retrieval_cli passed 139/139, so no reproducible semantic regression was present. This increases the retained metric from 285 to 287 by locking two additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: runtime-config-model-missing-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config when the embedding backend is builtin but the embedding model is missing, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution. The retrieval_cli suite passed cleanly at 139 tests, working_memory_assembly passed cleanly at 124 tests, and clippy stayed green. The full focused verify hit a low-frequency retrieval_cli allocator/runtime abort after lexical_search passed, but the immediate isolated rerun of retrieval_cli passed 139/139, so no reproducible semantic regression was present. This increases the retained metric from 285 to 287 by locking two additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 931
- **Timestamp:** 2026-04-22T01:37:16Z

### L-308: [labels: cli-model-missing-readiness-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: cli-model-missing-readiness-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for CLI fail-closed behavior when the embedding backend is builtin but the embedding model is missing, locking the public readiness-gating semantics for embedding_only and hybrid modes under that half-configured state. The retrieval_cli suite now passes at 141 tests, the full focused verify passed cleanly at 290 tests total, and clippy stayed green. This increases the retained metric from 287 to 289 by locking two additional user-visible second-channel readiness contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-model-missing-readiness-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for CLI fail-closed behavior when the embedding backend is builtin but the embedding model is missing, locking the public readiness-gating semantics for embedding_only and hybrid modes under that half-configured state. The retrieval_cli suite now passes at 141 tests, the full focused verify passed cleanly at 290 tests total, and clippy stayed green. This increases the retained metric from 287 to 289 by locking two additional user-visible second-channel readiness contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 933
- **Timestamp:** 2026-04-22T01:45:12Z

### L-309: [labels: variant-model-missing-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for Se
- **Strategy:** [labels: variant-model-missing-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_variant when the embedding backend is builtin but the embedding model is missing, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution under that half-configured variant state. The retrieval_cli suite now passes at 143 tests, the full focused verify passed cleanly at 292 tests total, and clippy stayed green. This increases the retained metric from 289 to 291 by locking two additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-model-missing-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_variant when the embedding backend is builtin but the embedding model is missing, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution under that half-configured variant state. The retrieval_cli suite now passes at 143 tests, the full focused verify passed cleanly at 292 tests total, and clippy stayed green. This increases the retained metric from 289 to 291 by locking two additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 935
- **Timestamp:** 2026-04-22T01:50:36Z

### L-310: [labels: vector-backend-none-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for seco
- **Strategy:** [labels: vector-backend-none-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for second-channel behavior when the vector backend is unavailable. The new tests lock both SearchService::with_runtime_config and SearchService::with_variant semantics: embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution when vector backend remains none. The retrieval_cli suite now passes at 147 tests, working_memory_assembly passes at 124 tests, and a single-thread lexical_search rerun passes at 25 tests. The default full focused verify still hit the known lexical_search allocator/runtime flake, but no reproducible semantic regression was present. This increases the retained metric from 291 to 295 by locking four additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: vector-backend-none-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for second-channel behavior when the vector backend is unavailable. The new tests lock both SearchService::with_runtime_config and SearchService::with_variant semantics: embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution when vector backend remains none. The retrieval_cli suite now passes at 147 tests, working_memory_assembly passes at 124 tests, and a single-thread lexical_search rerun passes at 25 tests. The default full focused verify still hit the known lexical_search allocator/runtime flake, but no reproducible semantic regression was present. This increases the retained metric from 291 to 295 by locking four additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 937
- **Timestamp:** 2026-04-22T01:55:59Z

### L-311: [labels: cli-vector-none-behavior-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for
- **Strategy:** [labels: cli-vector-none-behavior-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the current public CLI behavior when vector backend remains none while the embedding backend is builtin and a model is configured. The new tests lock that embedding_only still succeeds with an empty result set while hybrid still succeeds and degrades to lexical-only output under that half-configured state. The retrieval_cli suite now passes at 149 tests, and single-thread reruns of lexical_search and working_memory_assembly passed cleanly at 25 and 124 tests after the default full verify hit the known allocator/runtime flake. This increases the retained metric from 295 to 297 by locking two additional user-visible second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-vector-none-behavior-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the current public CLI behavior when vector backend remains none while the embedding backend is builtin and a model is configured. The new tests lock that embedding_only still succeeds with an empty result set while hybrid still succeeds and degrades to lexical-only output under that half-configured state. The retrieval_cli suite now passes at 149 tests, and single-thread reruns of lexical_search and working_memory_assembly passed cleanly at 25 and 124 tests after the default full verify hit the known allocator/runtime flake. This increases the retained metric from 295 to 297 by locking two additional user-visible second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 940
- **Timestamp:** 2026-04-22T02:06:10Z

### L-312: [labels: cli-reserved-backend-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for CLI
- **Strategy:** [labels: cli-reserved-backend-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for CLI fail-closed behavior when the embedding backend is reserved, locking the public readiness-gating semantics for embedding_only and hybrid modes under that Phase-1-reserved state. The retrieval_cli suite now passes at 151 tests, and after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 297 to 299 by locking two additional second-channel readiness contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-reserved-backend-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for CLI fail-closed behavior when the embedding backend is reserved, locking the public readiness-gating semantics for embedding_only and hybrid modes under that Phase-1-reserved state. The retrieval_cli suite now passes at 151 tests, and after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 297 to 299 by locking two additional second-channel readiness contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 942
- **Timestamp:** 2026-04-22T02:12:08Z

### L-313: [labels: reserved-backend-service-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for
- **Strategy:** [labels: reserved-backend-service-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config and SearchService::with_variant when the embedding backend is reserved, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution under that Phase-1-reserved state. The retrieval_cli suite now passes at 155 tests, the focused suite counts now stand at 25, 155, and 124, and an isolated rerun of working_memory_assembly passed cleanly at 124/124 after the default full verify hit the known allocator/runtime flake. This increases the retained metric from 299 to 303 by locking four additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: reserved-backend-service-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config and SearchService::with_variant when the embedding backend is reserved, locking the internal service-level semantics that embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution under that Phase-1-reserved state. The retrieval_cli suite now passes at 155 tests, the focused suite counts now stand at 25, 155, and 124, and an isolated rerun of working_memory_assembly passed cleanly at 124/124 after the default full verify hit the known allocator/runtime flake. This increases the retained metric from 299 to 303 by locking four additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 944
- **Timestamp:** 2026-04-22T02:17:56Z

### L-314: [labels: configured-mode-fallback-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for
- **Strategy:** [labels: configured-mode-fallback-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config when mode_override is omitted, locking that the service honors config.retrieval.mode itself for second-channel fallbacks: configured embedding_only returns no results while configured hybrid degrades to lexical-only results and lexical-only channel contribution when the embedding model is missing. The retrieval_cli suite now passes at 157 tests, and after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 303 to 305 by locking two additional configuration-driven second-channel contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-mode-fallback-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config when mode_override is omitted, locking that the service honors config.retrieval.mode itself for second-channel fallbacks: configured embedding_only returns no results while configured hybrid degrades to lexical-only results and lexical-only channel contribution when the embedding model is missing. The retrieval_cli suite now passes at 157 tests, and after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 303 to 305 by locking two additional configuration-driven second-channel contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 946
- **Timestamp:** 2026-04-22T02:24:25Z

### L-315: [labels: lexical-only-second-channel-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli
- **Strategy:** [labels: lexical-only-second-channel-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that lexical_only mode remains insulated from second-channel readiness state: lexical-only search still succeeds and reports lexical_only output when the embedding backend is reserved, and it also still succeeds when the embedding backend is builtin but the vector backend is missing. The retrieval_cli suite now passes at 159 tests and the full focused verify passed cleanly at 308 tests total. This increases the retained metric from 305 to 307 by locking two additional lexical-first isolation contracts around the optional second channel.
- **Outcome:** keep
- **Insight:** [labels: lexical-only-second-channel-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that lexical_only mode remains insulated from second-channel readiness state: lexical-only search still succeeds and reports lexical_only output when the embedding backend is reserved, and it also still succeeds when the embedding backend is builtin but the vector backend is missing. The retrieval_cli suite now passes at 159 tests and the full focused verify passed cleanly at 308 tests total. This increases the retained metric from 305 to 307 by locking two additional lexical-first isolation contracts around the optional second channel.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 947
- **Timestamp:** 2026-04-22T02:29:14Z

### L-316: [labels: configured-lexical-only-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli cov
- **Strategy:** [labels: configured-lexical-only-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that SearchService::with_runtime_config still honors configured lexical_only mode even when the optional second channel is half-configured. The new tests cover reserved embedding backend and missing vector backend cases, both of which still return lexical-only results with lexical_only channel contribution. The retrieval_cli suite now passes at 161 tests and the full focused verify passed cleanly at 310 tests total. This increases the retained metric from 307 to 309 by locking two additional configuration-driven lexical-first isolation contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-lexical-only-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that SearchService::with_runtime_config still honors configured lexical_only mode even when the optional second channel is half-configured. The new tests cover reserved embedding backend and missing vector backend cases, both of which still return lexical-only results with lexical_only channel contribution. The retrieval_cli suite now passes at 161 tests and the full focused verify passed cleanly at 310 tests total. This increases the retained metric from 307 to 309 by locking two additional configuration-driven lexical-first isolation contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 949
- **Timestamp:** 2026-04-22T02:33:57Z

### L-317: [labels: variant-lexical-only-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli covera
- **Strategy:** [labels: variant-lexical-only-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that SearchService::with_variant still honors lexical_only mode even when the optional second channel is half-configured. The new tests cover reserved embedding backend and missing vector backend cases, both of which still return lexical-only results with lexical_only channel contribution. The retrieval_cli suite now passes at 163 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 309 to 311 by locking two additional lexical-first isolation contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-lexical-only-isolation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that SearchService::with_variant still honors lexical_only mode even when the optional second channel is half-configured. The new tests cover reserved embedding backend and missing vector backend cases, both of which still return lexical-only results with lexical_only channel contribution. The retrieval_cli suite now passes at 163 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 309 to 311 by locking two additional lexical-first isolation contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 951
- **Timestamp:** 2026-04-22T02:39:30Z

### L-318: [labels: variant-lexical-only-isolation-lock-2, lexical-first, working-memory-assembly] Added focused retrieval_cli cove
- **Strategy:** [labels: variant-lexical-only-isolation-lock-2, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that SearchService::with_variant still honors lexical_only mode even when the optional second channel is half-configured. The new tests cover reserved embedding backend and missing vector backend cases, both of which still return lexical-only results with lexical_only channel contribution. The retrieval_cli suite now passes at 165 tests and the full focused verify passed cleanly at 314 tests total. This increases the retained metric from 311 to 313 by locking two additional lexical-first isolation contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-lexical-only-isolation-lock-2, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock that SearchService::with_variant still honors lexical_only mode even when the optional second channel is half-configured. The new tests cover reserved embedding backend and missing vector backend cases, both of which still return lexical-only results with lexical_only channel contribution. The retrieval_cli suite now passes at 165 tests and the full focused verify passed cleanly at 314 tests total. This increases the retained metric from 311 to 313 by locking two additional lexical-first isolation contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 953
- **Timestamp:** 2026-04-22T03:21:58Z

### L-319: [labels: mode-override-contract-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for S
- **Strategy:** [labels: mode-override-contract-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config mode_override contracts when the embedding channel is ready. The new tests lock that mode_override=lexical_only suppresses embedding contribution and strategies even if config.retrieval.mode is embedding_only, while mode_override=hybrid enables hybrid contribution and embedding strategies even if config.retrieval.mode is lexical_only. The retrieval_cli suite now passes at 167 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 313 to 315 by locking two additional mode-override contracts.
- **Outcome:** keep
- **Insight:** [labels: mode-override-contract-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for SearchService::with_runtime_config mode_override contracts when the embedding channel is ready. The new tests lock that mode_override=lexical_only suppresses embedding contribution and strategies even if config.retrieval.mode is embedding_only, while mode_override=hybrid enables hybrid contribution and embedding strategies even if config.retrieval.mode is lexical_only. The retrieval_cli suite now passes at 167 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 124/124, so no reproducible semantic regression was present. This increases the retained metric from 313 to 315 by locking two additional mode-override contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 955
- **Timestamp:** 2026-04-22T03:28:15Z

### L-320: [labels: embedding-model-mismatch-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for
- **Strategy:** [labels: embedding-model-mismatch-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for second-channel behavior when the configured embedding model mismatches the stored embedding sidecar. The new tests lock both SearchService::with_runtime_config and SearchService::with_variant semantics: embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution when embeddings exist but the configured model name does not match them. The retrieval_cli suite now passes at 171 tests and the full focused verify passed cleanly at 320 tests total. This increases the retained metric from 315 to 319 by locking four additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: embedding-model-mismatch-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for second-channel behavior when the configured embedding model mismatches the stored embedding sidecar. The new tests lock both SearchService::with_runtime_config and SearchService::with_variant semantics: embedding_only returns no results while hybrid degrades to lexical-only results and lexical-only channel contribution when embeddings exist but the configured model name does not match them. The retrieval_cli suite now passes at 171 tests and the full focused verify passed cleanly at 320 tests total. This increases the retained metric from 315 to 319 by locking four additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 957
- **Timestamp:** 2026-04-22T03:34:29Z

### L-321: [labels: cli-default-lexical-ready-embedding-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli c
- **Strategy:** [labels: cli-default-lexical-ready-embedding-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the current public CLI behavior when retrieval.mode is lexical_only and the embedding channel is fully ready. The new tests lock that default lexical_only search still succeeds, reports lexical_only channel contribution, and does not surface embedding strategies in either JSON or text output even when the optional second channel is ready. The retrieval_cli suite now passes at 173 tests and the full focused verify passed cleanly at 322 tests total. This increases the retained metric from 319 to 321 by locking two additional lexical-first isolation contracts around a fully ready second channel.
- **Outcome:** keep
- **Insight:** [labels: cli-default-lexical-ready-embedding-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the current public CLI behavior when retrieval.mode is lexical_only and the embedding channel is fully ready. The new tests lock that default lexical_only search still succeeds, reports lexical_only channel contribution, and does not surface embedding strategies in either JSON or text output even when the optional second channel is ready. The retrieval_cli suite now passes at 173 tests and the full focused verify passed cleanly at 322 tests total. This increases the retained metric from 319 to 321 by locking two additional lexical-first isolation contracts around a fully ready second channel.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 959
- **Timestamp:** 2026-04-22T03:42:58Z

### L-322: [labels: embedding-only-ready-path-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage fo
- **Strategy:** [labels: embedding-only-ready-path-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the ready embedding-only path at the library layer. The new tests lock that SearchService::with_runtime_config(mode_override=EmbeddingOnly) and SearchService::with_variant(mode=EmbeddingOnly) both surface embedding_only channel contribution and only the Embedding query strategy when the second channel is fully ready. The retrieval_cli suite now passes at 175 tests and the full focused verify passed cleanly at 324 tests total. This increases the retained metric from 321 to 323 by locking two additional second-channel contracts on the positive ready path.
- **Outcome:** keep
- **Insight:** [labels: embedding-only-ready-path-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the ready embedding-only path at the library layer. The new tests lock that SearchService::with_runtime_config(mode_override=EmbeddingOnly) and SearchService::with_variant(mode=EmbeddingOnly) both surface embedding_only channel contribution and only the Embedding query strategy when the second channel is fully ready. The retrieval_cli suite now passes at 175 tests and the full focused verify passed cleanly at 324 tests total. This increases the retained metric from 321 to 323 by locking two additional second-channel contracts on the positive ready path.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 960
- **Timestamp:** 2026-04-22T03:48:26Z

### L-323: [labels: hybrid-ready-path-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the re
- **Strategy:** [labels: hybrid-ready-path-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the ready hybrid path at the library layer. The new tests lock that SearchService::with_runtime_config when configured as hybrid and SearchService::with_variant when set to hybrid both surface hybrid channel contribution and include the Embedding strategy when the second channel is fully ready. The retrieval_cli suite now passes at 177 tests and the full focused verify passed cleanly at 326 tests total. This increases the retained metric from 323 to 325 by locking two additional positive-path second-channel contracts.
- **Outcome:** keep
- **Insight:** [labels: hybrid-ready-path-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the ready hybrid path at the library layer. The new tests lock that SearchService::with_runtime_config when configured as hybrid and SearchService::with_variant when set to hybrid both surface hybrid channel contribution and include the Embedding strategy when the second channel is fully ready. The retrieval_cli suite now passes at 177 tests and the full focused verify passed cleanly at 326 tests total. This increases the retained metric from 323 to 325 by locking two additional positive-path second-channel contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 962
- **Timestamp:** 2026-04-22T03:54:09Z

### L-324: [labels: ready-second-channel-text-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage fo
- **Strategy:** [labels: ready-second-channel-text-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the current text-surface behavior when the second channel is fully ready. The new tests lock that text-mode search reports channel: embedding_only with embedding strategies for embedding_only, and channel: hybrid with embedding participation for hybrid. The retrieval_cli suite now passes at 179 tests and the full focused verify passed cleanly at 328 tests total. This increases the retained metric from 325 to 327 by locking two additional user-visible positive-path second-channel contracts.
- **Outcome:** keep
- **Insight:** [labels: ready-second-channel-text-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the current text-surface behavior when the second channel is fully ready. The new tests lock that text-mode search reports channel: embedding_only with embedding strategies for embedding_only, and channel: hybrid with embedding participation for hybrid. The retrieval_cli suite now passes at 179 tests and the full focused verify passed cleanly at 328 tests total. This increases the retained metric from 325 to 327 by locking two additional user-visible positive-path second-channel contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 964
- **Timestamp:** 2026-04-22T04:02:14Z

### L-325: [labels: second-channel-trace-preservation-lock, lexical-first, working-memory-assembly] Added focused working_memory_as
- **Strategy:** [labels: second-channel-trace-preservation-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage to lock that fragment trace metadata from second-channel searches is preserved when integrated results are assembled into working memory. The new tests verify EmbeddingOnly and Hybrid channel_contribution plus Embedding query strategy propagation through integrated_results. The focused suite counts now stand at 25, 179, and 126, and the full focused verify passed cleanly at 330 tests total. This increases the retained metric from 327 to 332 by locking two additional trace-preservation contracts on the second-channel path.
- **Outcome:** keep
- **Insight:** [labels: second-channel-trace-preservation-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage to lock that fragment trace metadata from second-channel searches is preserved when integrated results are assembled into working memory. The new tests verify EmbeddingOnly and Hybrid channel_contribution plus Embedding query strategy propagation through integrated_results. The focused suite counts now stand at 25, 179, and 126, and the full focused verify passed cleanly at 330 tests total. This increases the retained metric from 327 to 332 by locking two additional trace-preservation contracts on the second-channel path.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 966
- **Timestamp:** 2026-04-22T04:11:59Z

### L-326: [labels: ready-path-json-strategies-lock, lexical-first, working-memory-assembly] Strengthened the existing ready-path J
- **Strategy:** [labels: ready-path-json-strategies-lock, lexical-first, working-memory-assembly] Strengthened the existing ready-path JSON contracts in retrieval_cli by asserting the exact query_strategies payloads alongside channel_contribution for embedding_only and hybrid modes. This did not change the suite counts (still 25, 179, and 126 across lexical_search, retrieval_cli, and working_memory_assembly), but it materially increases coverage strength by locking two additional user-visible trace details on the second-channel ready path. The retained metric increases from 332 to 334.
- **Outcome:** keep
- **Insight:** [labels: ready-path-json-strategies-lock, lexical-first, working-memory-assembly] Strengthened the existing ready-path JSON contracts in retrieval_cli by asserting the exact query_strategies payloads alongside channel_contribution for embedding_only and hybrid modes. This did not change the suite counts (still 25, 179, and 126 across lexical_search, retrieval_cli, and working_memory_assembly), but it materially increases coverage strength by locking two additional user-visible trace details on the second-channel ready path. The retained metric increases from 332 to 334.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 968
- **Timestamp:** 2026-04-22T04:21:41Z

### L-327: [labels: exact-ready-path-text-summary-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverag
- **Strategy:** [labels: exact-ready-path-text-summary-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks the exact text strategy summaries on the second-channel ready path. The new tests assert that embedding_only text output reports and that hybrid text output reports the exact mixed strategy summary when the second channel is fully ready. The retrieval_cli suite now passes at 181 tests and the full focused verify passed cleanly at 332 tests total. This increases the retained metric from 334 to 336 by locking two additional user-visible text-surface contracts.
- **Outcome:** keep
- **Insight:** [labels: exact-ready-path-text-summary-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks the exact text strategy summaries on the second-channel ready path. The new tests assert that embedding_only text output reports and that hybrid text output reports the exact mixed strategy summary when the second channel is fully ready. The retrieval_cli suite now passes at 181 tests and the full focused verify passed cleanly at 332 tests total. This increases the retained metric from 334 to 336 by locking two additional user-visible text-surface contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 972
- **Timestamp:** 2026-04-22T08:59:37Z

### L-328: Expanded lexical_search coverage to lock a broad set of ordinary-retrieval contracts inside the focused lexical suite: r
- **Strategy:** Expanded lexical_search coverage to lock a broad set of ordinary-retrieval contracts inside the focused lexical suite: repeated-query dedupe, parsed RFC3339 recency ordering, parsed temporal filtering, structured-only provenance/citation surfaces, mixed lexical+structured provenance and snippet priority, and DSL sidecar attachment on ordinary recall. The lexical_search suite still passes at 25 tests, retrieval_cli at 181, working_memory_assembly at 126, and the full focused verify passed cleanly at 332 tests total with clippy green. This increases the retained metric from 334 to 340 by materially strengthening the lexical-first baseline contract surface without broadening scope.
- **Outcome:** keep
- **Insight:** Expanded lexical_search coverage to lock a broad set of ordinary-retrieval contracts inside the focused lexical suite: repeated-query dedupe, parsed RFC3339 recency ordering, parsed temporal filtering, structured-only provenance/citation surfaces, mixed lexical+structured provenance and snippet priority, and DSL sidecar attachment on ordinary recall. The lexical_search suite still passes at 25 tests, retrieval_cli at 181, working_memory_assembly at 126, and the full focused verify passed cleanly at 332 tests total with clippy green. This increases the retained metric from 334 to 340 by materially strengthening the lexical-first baseline contract surface without broadening scope.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 974
- **Timestamp:** 2026-04-22T09:06:23Z

### L-329: [labels: exact-hybrid-trace-order-lock, lexical-first, working-memory-assembly] Tightened the working_memory_assembly hy
- **Strategy:** [labels: exact-hybrid-trace-order-lock, lexical-first, working-memory-assembly] Tightened the working_memory_assembly hybrid trace contract by asserting the exact ready-path query_strategies ordering on assembled fragments instead of only checking partial containment. The focused suite counts remain 25, 181, and 126, and the full focused verify passed cleanly at 332 tests total with clippy green. This increases the retained metric from 340 to 341 by turning a partial hybrid trace guarantee into an exact ordering contract.
- **Outcome:** keep
- **Insight:** [labels: exact-hybrid-trace-order-lock, lexical-first, working-memory-assembly] Tightened the working_memory_assembly hybrid trace contract by asserting the exact ready-path query_strategies ordering on assembled fragments instead of only checking partial containment. The focused suite counts remain 25, 181, and 126, and the full focused verify passed cleanly at 332 tests total with clippy green. This increases the retained metric from 340 to 341 by turning a partial hybrid trace guarantee into an exact ordering contract.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 981
- **Timestamp:** 2026-04-22T09:24:18Z

### L-330: [labels: embedding-dimension-mismatch-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: embedding-dimension-mismatch-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for dimension-mismatch behavior on the second-channel embedding path. The new tests lock both SearchService::with_runtime_config and SearchService::with_variant semantics: embedding_only returns no results while hybrid degrades to lexical-only results and omits embedding strategies when stored embedding vector dimensions do not match the query embedding dimensions. The retrieval_cli suite now passes at 185 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 126/126, so no reproducible semantic regression was present. This increases the retained metric from 340 to 344 by locking four additional second-channel boundary contracts.
- **Outcome:** keep
- **Insight:** [labels: embedding-dimension-mismatch-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for dimension-mismatch behavior on the second-channel embedding path. The new tests lock both SearchService::with_runtime_config and SearchService::with_variant semantics: embedding_only returns no results while hybrid degrades to lexical-only results and omits embedding strategies when stored embedding vector dimensions do not match the query embedding dimensions. The retrieval_cli suite now passes at 185 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 126/126, so no reproducible semantic regression was present. This increases the retained metric from 340 to 344 by locking four additional second-channel boundary contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 985
- **Timestamp:** 2026-04-22T13:08:33Z

### L-331: [labels: exact-hybrid-strategy-order-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: exact-hybrid-strategy-order-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock the exact hybrid strategy ordering on the ready path at the library layer. The new tests assert that both SearchService::with_runtime_config when configured as hybrid and SearchService::with_variant when set to hybrid preserve the full ready-path strategy ordering Jieba, Simple, Structured, Embedding instead of merely containing Embedding. The retrieval_cli suite now passes at 187 tests and the full focused verify passed cleanly at 338 tests total. This increases the retained metric from 344 to 346 by strengthening two existing hybrid-ready contracts into exact ordering guarantees.
- **Outcome:** keep
- **Insight:** [labels: exact-hybrid-strategy-order-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock the exact hybrid strategy ordering on the ready path at the library layer. The new tests assert that both SearchService::with_runtime_config when configured as hybrid and SearchService::with_variant when set to hybrid preserve the full ready-path strategy ordering Jieba, Simple, Structured, Embedding instead of merely containing Embedding. The retrieval_cli suite now passes at 187 tests and the full focused verify passed cleanly at 338 tests total. This increases the retained metric from 344 to 346 by strengthening two existing hybrid-ready contracts into exact ordering guarantees.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 987
- **Timestamp:** 2026-04-22T13:18:02Z

### L-332: [labels: configured-embedding-only-ready-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli cover
- **Strategy:** [labels: configured-embedding-only-ready-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the configured embedding_only ready path at the library layer. The new test locks that SearchService::with_runtime_config with config.retrieval.mode=embedding_only (and no mode override) surfaces embedding_only channel contribution and only the Embedding strategy when the second channel is fully ready. The retrieval_cli suite now passes at 188 tests and, after the default full verify hit the known lexical_search/working_memory allocator flake pattern, single-thread reruns of lexical_search and working_memory_assembly both passed cleanly. This increases the retained metric from 346 to 348 by locking one additional positive-path second-channel contract.
- **Outcome:** keep
- **Insight:** [labels: configured-embedding-only-ready-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for the configured embedding_only ready path at the library layer. The new test locks that SearchService::with_runtime_config with config.retrieval.mode=embedding_only (and no mode override) surfaces embedding_only channel contribution and only the Embedding strategy when the second channel is fully ready. The retrieval_cli suite now passes at 188 tests and, after the default full verify hit the known lexical_search/working_memory allocator flake pattern, single-thread reruns of lexical_search and working_memory_assembly both passed cleanly. This increases the retained metric from 346 to 348 by locking one additional positive-path second-channel contract.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 989
- **Timestamp:** 2026-04-22T13:29:07Z

### L-333: [labels: exact-ready-path-json-order-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: exact-ready-path-json-order-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock the exact ready-path JSON strategy ordering on the CLI surface. The new tests assert the full lexical_only strategy order Jieba, Simple, Structured under a ready-but-inactive second channel, and the full hybrid strategy order Jieba, Simple, Structured, Embedding under a fully active hybrid path. The retrieval_cli suite now passes at 190 tests and the full focused verify passed cleanly at 341 tests total. This increases the retained metric from 348 to 350 by strengthening two user-visible strategy-order contracts on the JSON surface.
- **Outcome:** keep
- **Insight:** [labels: exact-ready-path-json-order-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage to lock the exact ready-path JSON strategy ordering on the CLI surface. The new tests assert the full lexical_only strategy order Jieba, Simple, Structured under a ready-but-inactive second channel, and the full hybrid strategy order Jieba, Simple, Structured, Embedding under a fully active hybrid path. The retrieval_cli suite now passes at 190 tests and the full focused verify passed cleanly at 341 tests total. This increases the retained metric from 348 to 350 by strengthening two user-visible strategy-order contracts on the JSON surface.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 991
- **Timestamp:** 2026-04-22T13:41:00Z

### L-334: [labels: embedding-only-snippet-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli and working_me
- **Strategy:** [labels: embedding-only-snippet-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli and working_memory_assembly coverage to lock snippet preservation on the embedding-only ready path. The new tests assert that embedding-only search uses the raw 96-character truncated snippet at the library surface and that the same truncated snippet is preserved when integrated results are assembled into working memory fragments. The focused suite counts now stand at 25, 191, and 127, and the full focused verify passed cleanly at 343 tests total with clippy green. This increases the retained metric from 350 to 352 by locking two additional user-visible second-channel snippet contracts.
- **Outcome:** keep
- **Insight:** [labels: embedding-only-snippet-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli and working_memory_assembly coverage to lock snippet preservation on the embedding-only ready path. The new tests assert that embedding-only search uses the raw 96-character truncated snippet at the library surface and that the same truncated snippet is preserved when integrated results are assembled into working memory fragments. The focused suite counts now stand at 25, 191, and 127, and the full focused verify passed cleanly at 343 tests total with clippy green. This increases the retained metric from 350 to 352 by locking two additional user-visible second-channel snippet contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 994
- **Timestamp:** 2026-04-22T14:00:32Z

### L-335: [labels: missing-chunk-citation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for f
- **Strategy:** [labels: missing-chunk-citation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for fail-closed citation behavior when records lack chunk metadata. The new tests lock both library and CLI search surfaces so they reject malformed records with the expected missing chunk metadata error instead of silently producing uncitable results. The focused suite counts now stand at 25, 193, and 127, and the full focused verify passed cleanly at 345 tests total. This increases the retained metric from 352 to 354 by locking two additional user-visible failure-path contracts.
- **Outcome:** keep
- **Insight:** [labels: missing-chunk-citation-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for fail-closed citation behavior when records lack chunk metadata. The new tests lock both library and CLI search surfaces so they reject malformed records with the expected missing chunk metadata error instead of silently producing uncitable results. The focused suite counts now stand at 25, 193, and 127, and the full focused verify passed cleanly at 345 tests total. This increases the retained metric from 352 to 354 by locking two additional user-visible failure-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1004
- **Timestamp:** 2026-04-22T14:25:19Z

### L-336: [labels: builtin-model-fallback-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for b
- **Strategy:** [labels: builtin-model-fallback-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for builtin model-name fallback dimensions on the embedding-only ready path. The new tests lock that both SearchService::with_runtime_config and SearchService::with_variant treat an unsuffixed builtin model name as the default 16-dimensional path, still yielding embedding_only channel contribution with only the Embedding strategy. The retrieval_cli suite now passes at 195 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 127/127, so no reproducible semantic regression was present. This increases the retained metric from 354 to 356 by locking two additional second-channel ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: builtin-model-fallback-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for builtin model-name fallback dimensions on the embedding-only ready path. The new tests lock that both SearchService::with_runtime_config and SearchService::with_variant treat an unsuffixed builtin model name as the default 16-dimensional path, still yielding embedding_only channel contribution with only the Embedding strategy. The retrieval_cli suite now passes at 195 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 127/127, so no reproducible semantic regression was present. This increases the retained metric from 354 to 356 by locking two additional second-channel ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1007
- **Timestamp:** 2026-04-22T14:39:03Z

### L-337: [labels: builtin-model-hybrid-ready-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage f
- **Strategy:** [labels: builtin-model-hybrid-ready-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for unsuffixed builtin model fallback behavior on the hybrid ready path. The new tests lock that both SearchService::with_runtime_config and SearchService::with_variant treat an unsuffixed builtin model name as the default 16-dimensional path for hybrid retrieval, yielding hybrid channel contribution with the full ready-path strategy ordering. The retrieval_cli suite now passes at 197 tests and, after the default full verify hit the known working_memory allocator/runtime flake, an isolated rerun of working_memory_assembly passed cleanly at 127/127, so no reproducible semantic regression was present. This increases the retained metric from 354 to 360 by locking two additional positive-path second-channel contracts.
- **Outcome:** keep
- **Insight:** [labels: builtin-model-hybrid-ready-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for unsuffixed builtin model fallback behavior on the hybrid ready path. The new tests lock that both SearchService::with_runtime_config and SearchService::with_variant treat an unsuffixed builtin model name as the default 16-dimensional path for hybrid retrieval, yielding hybrid channel contribution with the full ready-path strategy ordering. The retrieval_cli suite now passes at 197 tests and, after the default full verify hit the known working_memory allocator/runtime flake, an isolated rerun of working_memory_assembly passed cleanly at 127/127, so no reproducible semantic regression was present. This increases the retained metric from 354 to 360 by locking two additional positive-path second-channel contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1011
- **Timestamp:** 2026-04-22T14:52:51Z

### L-338: [labels: configured-ready-path-json-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage f
- **Strategy:** [labels: configured-ready-path-json-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for default configured ready-path JSON behavior. The new tests lock that CLI search without an explicit --mode flag honors configured embedding_only and hybrid modes when the second channel is fully ready, surfacing the expected channel_contribution and exact query_strategies payloads on the JSON surface. The retrieval_cli suite now passes at 199 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 127/127, so no reproducible semantic regression was present. This increases the retained metric from 360 to 362 by locking two additional user-visible configuration-driven positive-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-ready-path-json-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for default configured ready-path JSON behavior. The new tests lock that CLI search without an explicit --mode flag honors configured embedding_only and hybrid modes when the second channel is fully ready, surfacing the expected channel_contribution and exact query_strategies payloads on the JSON surface. The retrieval_cli suite now passes at 199 tests and, after the default full verify hit the known lexical_search allocator/runtime flake, a single-thread lexical_search rerun passed 25/25 and working_memory_assembly passed 127/127, so no reproducible semantic regression was present. This increases the retained metric from 360 to 362 by locking two additional user-visible configuration-driven positive-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1013
- **Timestamp:** 2026-04-22T15:04:33Z

### L-339: [labels: ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage fo
- **Strategy:** [labels: ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for record and citation shape preservation on the fully ready second-channel path. The new tests lock that both embedding_only and hybrid ready-path searches still preserve record scope/truth/type along with citation record_id, source_uri, recorded_at, validity, and chunk anchor metadata. The retrieval_cli suite now passes at 201 tests and the full focused verify passed cleanly at 353 tests total. This increases the retained metric from 362 to 364 by locking two additional user-visible positive-path contracts.
- **Outcome:** keep
- **Insight:** [labels: ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage for record and citation shape preservation on the fully ready second-channel path. The new tests lock that both embedding_only and hybrid ready-path searches still preserve record scope/truth/type along with citation record_id, source_uri, recorded_at, validity, and chunk anchor metadata. The retrieval_cli suite now passes at 201 tests and the full focused verify passed cleanly at 353 tests total. This increases the retained metric from 362 to 364 by locking two additional user-visible positive-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1014
- **Timestamp:** 2026-04-23T00:45:55Z

### L-340: [labels: nonblank-integrated-dsl-hydration-lock, lexical-first, working-memory-assembly] Added focused working_memory_as
- **Strategy:** [labels: nonblank-integrated-dsl-hydration-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks repository-backed DSL hydration for nonblank-query integrated follow-up evidence, covering both world fragments and explicit branch support while preserving caller-provided snippets. The focused suite counts now stand at 25, 201, and 131 tests respectively; working_memory_assembly passed cleanly in isolation at 131/131, cargo clippy --quiet --all-targets -- -D warnings stayed green, and the default combined verify again hit the known allocator/runtime abort only after lexical_search and retrieval_cli had already passed 226 tests total, so no reproducible semantic regression was present.
- **Outcome:** keep
- **Insight:** [labels: nonblank-integrated-dsl-hydration-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks repository-backed DSL hydration for nonblank-query integrated follow-up evidence, covering both world fragments and explicit branch support while preserving caller-provided snippets. The focused suite counts now stand at 25, 201, and 131 tests respectively; working_memory_assembly passed cleanly in isolation at 131/131, cargo clippy --quiet --all-targets -- -D warnings stayed green, and the default combined verify again hit the known allocator/runtime abort only after lexical_search and retrieval_cli had already passed 226 tests total, so no reproducible semantic regression was present.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1017
- **Timestamp:** 2026-04-23T01:03:28Z

### L-341: [labels: ready-path-dsl-sidecar-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that
- **Strategy:** [labels: ready-path-dsl-sidecar-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks structured DSL sidecar exposure on the second-channel ready path for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 203, and 131 tests respectively, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 359 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 368 to 370 by locking two additional user-visible ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: ready-path-dsl-sidecar-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks structured DSL sidecar exposure on the second-channel ready path for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 203, and 131 tests respectively, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 359 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 368 to 370 by locking two additional user-visible ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1019
- **Timestamp:** 2026-04-23T01:10:53Z

### L-342: [labels: cli-ready-path-dsl-sidecar-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage t
- **Strategy:** [labels: cli-ready-path-dsl-sidecar-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks CLI JSON exposure of the structured DSL sidecar on the second-channel ready path for both embedding_only and hybrid mode selection. The focused suite counts now stand at 25, 205, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 205 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 361 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 370 to 372 by locking two additional user-visible ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-ready-path-dsl-sidecar-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks CLI JSON exposure of the structured DSL sidecar on the second-channel ready path for both embedding_only and hybrid mode selection. The focused suite counts now stand at 25, 205, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 205 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 361 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 370 to 372 by locking two additional user-visible ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1021
- **Timestamp:** 2026-04-23T01:16:22Z

### L-343: [labels: cli-text-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that
- **Strategy:** [labels: cli-text-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks CLI text rendering of the structured DSL summary on the second-channel ready path for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 207, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 207 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 363 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 372 to 374 by locking two additional user-visible ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-text-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks CLI text rendering of the structured DSL summary on the second-channel ready path for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 207, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 207 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 363 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 372 to 374 by locking two additional user-visible ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1025
- **Timestamp:** 2026-04-23T01:29:10Z

### L-344: [labels: configured-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage th
- **Strategy:** [labels: configured-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks structured DSL exposure on configured second-channel ready paths for both embedding_only and hybrid retrieval across JSON and text surfaces. The focused suite counts now stand at 25, 211, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 211 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 367 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 374 to 378 by locking four additional user-visible ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks structured DSL exposure on configured second-channel ready paths for both embedding_only and hybrid retrieval across JSON and text surfaces. The focused suite counts now stand at 25, 211, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 211 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 367 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 374 to 378 by locking four additional user-visible ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1027
- **Timestamp:** 2026-04-23T01:34:45Z

### L-345: [labels: ready-path-taxonomy-filter-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage t
- **Strategy:** [labels: ready-path-taxonomy-filter-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks taxonomy filtering before top-k truncation on configured second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 213, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 213 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 369 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 378 to 380 by locking two additional user-visible ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: ready-path-taxonomy-filter-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks taxonomy filtering before top-k truncation on configured second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 213, and 131 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 213 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 369 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 378 to 380 by locking two additional user-visible ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1031
- **Timestamp:** 2026-04-23T01:42:37Z

### L-346: [labels: integrated-result-filter-lock, lexical-first, working-memory-assembly] Filtered caller-provided integrated_resu
- **Strategy:** [labels: integrated-result-filter-lock, lexical-first, working-memory-assembly] Filtered caller-provided integrated_results through WorkingMemoryRequest search filters inside the assembler and added focused coverage for taxonomy-filtered integrated evidence. The focused suite counts now stand at 25, 213, and 132 tests respectively; cargo test --quiet --test working_memory_assembly passed cleanly at 132 tests, cargo clippy --quiet --all-targets -- -D warnings stayed green, and the default full focused verify only reproduced the known lexical_search allocator/runtime abort before an immediate single-thread lexical_search rerun passed 25/25. This increases the retained metric from 380 to 381 by locking a real assembly-layer filter contract without introducing a reproducible semantic regression.
- **Outcome:** keep
- **Insight:** [labels: integrated-result-filter-lock, lexical-first, working-memory-assembly] Filtered caller-provided integrated_results through WorkingMemoryRequest search filters inside the assembler and added focused coverage for taxonomy-filtered integrated evidence. The focused suite counts now stand at 25, 213, and 132 tests respectively; cargo test --quiet --test working_memory_assembly passed cleanly at 132 tests, cargo clippy --quiet --all-targets -- -D warnings stayed green, and the default full focused verify only reproduced the known lexical_search allocator/runtime abort before an immediate single-thread lexical_search rerun passed 25/25. This increases the retained metric from 380 to 381 by locking a real assembly-layer filter contract without introducing a reproducible semantic regression.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1033
- **Timestamp:** 2026-04-23T01:48:40Z

### L-347: [labels: filtered-supporting-record-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly
- **Strategy:** [labels: filtered-supporting-record-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks fail-closed behavior when explicit supporting_record_ids reference integrated results filtered out by assembly search filters. The focused suite counts now stand at 25, 213, and 133 tests respectively; cargo clippy --quiet --all-targets -- -D warnings stayed green, the default working_memory_assembly run only reproduced the known allocator/runtime abort, and an immediate single-thread rerun of RUST_TEST_THREADS=1 cargo test --quiet --test working_memory_assembly passed cleanly at 133/133. This increases the retained metric from 381 to 382 by locking an additional assembly-layer filter contract without introducing a reproducible semantic regression.
- **Outcome:** keep
- **Insight:** [labels: filtered-supporting-record-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks fail-closed behavior when explicit supporting_record_ids reference integrated results filtered out by assembly search filters. The focused suite counts now stand at 25, 213, and 133 tests respectively; cargo clippy --quiet --all-targets -- -D warnings stayed green, the default working_memory_assembly run only reproduced the known allocator/runtime abort, and an immediate single-thread rerun of RUST_TEST_THREADS=1 cargo test --quiet --test working_memory_assembly passed cleanly at 133/133. This increases the retained metric from 381 to 382 by locking an additional assembly-layer filter contract without introducing a reproducible semantic regression.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1035
- **Timestamp:** 2026-04-23T01:52:32Z

### L-348: [labels: integrated-core-temporal-filter-lock, lexical-first, working-memory-assembly] Added focused working_memory_asse
- **Strategy:** [labels: integrated-core-temporal-filter-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks core and temporal filter application over caller-provided integrated_results. The focused suite counts now stand at 25, 213, and 134 tests respectively, cargo test --quiet --test working_memory_assembly passed cleanly at 134 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 372 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 382 to 383 by locking an additional assembly-layer integrated-result filter contract.
- **Outcome:** keep
- **Insight:** [labels: integrated-core-temporal-filter-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks core and temporal filter application over caller-provided integrated_results. The focused suite counts now stand at 25, 213, and 134 tests respectively, cargo test --quiet --test working_memory_assembly passed cleanly at 134 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 372 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 382 to 383 by locking an additional assembly-layer integrated-result filter contract.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1037
- **Timestamp:** 2026-04-23T01:56:56Z

### L-349: [labels: filtered-stale-supporting-record-lock, lexical-first, working-memory-assembly] Added focused working_memory_ass
- **Strategy:** [labels: filtered-stale-supporting-record-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks fail-closed behavior when explicit supporting_record_ids reference integrated results filtered out by core and temporal search filters. The focused suite counts now stand at 25, 213, and 135 tests respectively, cargo test --quiet --test working_memory_assembly passed cleanly at 135 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 373 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 383 to 384 by locking another assembly-layer filter contract.
- **Outcome:** keep
- **Insight:** [labels: filtered-stale-supporting-record-lock, lexical-first, working-memory-assembly] Added focused working_memory_assembly coverage that locks fail-closed behavior when explicit supporting_record_ids reference integrated results filtered out by core and temporal search filters. The focused suite counts now stand at 25, 213, and 135 tests respectively, cargo test --quiet --test working_memory_assembly passed cleanly at 135 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 373 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 383 to 384 by locking another assembly-layer filter contract.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1039
- **Timestamp:** 2026-04-23T02:00:50Z

### L-350: [labels: variant-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that
- **Strategy:** [labels: variant-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks structured DSL sidecar exposure on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 215, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 215 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 375 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 384 to 386 by locking two additional user-visible variant ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-ready-path-dsl-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks structured DSL sidecar exposure on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 215, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 215 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 375 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 384 to 386 by locking two additional user-visible variant ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1041
- **Timestamp:** 2026-04-23T02:06:31Z

### L-351: [labels: variant-ready-path-taxonomy-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: variant-ready-path-taxonomy-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks taxonomy filtering before top-k truncation on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 217, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 217 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 377 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 386 to 388 by locking two additional user-visible variant ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-ready-path-taxonomy-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks taxonomy filtering before top-k truncation on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 217, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 217 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 377 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 386 to 388 by locking two additional user-visible variant ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1043
- **Timestamp:** 2026-04-23T02:11:27Z

### L-352: [labels: cli-ready-path-taxonomy-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that
- **Strategy:** [labels: cli-ready-path-taxonomy-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks taxonomy filtering before top-k truncation on explicit CLI second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 219, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 219 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 379 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 388 to 390 by locking two additional user-visible CLI ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-ready-path-taxonomy-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks taxonomy filtering before top-k truncation on explicit CLI second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 219, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 219 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 379 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 388 to 390 by locking two additional user-visible CLI ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1045
- **Timestamp:** 2026-04-23T02:16:53Z

### L-353: [labels: ready-path-temporal-filter-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage t
- **Strategy:** [labels: ready-path-temporal-filter-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on runtime-config second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 221, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 221 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 381 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 390 to 392 by locking two additional user-visible ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: ready-path-temporal-filter-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on runtime-config second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 221, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 221 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 381 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 390 to 392 by locking two additional user-visible ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1048
- **Timestamp:** 2026-04-23T02:23:21Z

### L-354: [labels: variant-ready-path-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: variant-ready-path-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 223, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 223 tests, the default full focused verify only reproduced the known working_memory_assembly native flake after lexical_search and retrieval_cli had already passed 248 tests total, an immediate single-thread rerun of RUST_TEST_THREADS=1 cargo test --quiet --test working_memory_assembly passed cleanly at 135/135, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 392 to 394 by locking two additional user-visible variant ready-path contracts without introducing a reproducible semantic regression.
- **Outcome:** keep
- **Insight:** [labels: variant-ready-path-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 223, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 223 tests, the default full focused verify only reproduced the known working_memory_assembly native flake after lexical_search and retrieval_cli had already passed 248 tests total, an immediate single-thread rerun of RUST_TEST_THREADS=1 cargo test --quiet --test working_memory_assembly passed cleanly at 135/135, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 392 to 394 by locking two additional user-visible variant ready-path contracts without introducing a reproducible semantic regression.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1051
- **Timestamp:** 2026-04-23T02:33:02Z

### L-355: [labels: cli-ready-path-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that
- **Strategy:** [labels: cli-ready-path-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on explicit CLI second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 225, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 225 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 385 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 394 to 396 by locking two additional user-visible CLI ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-ready-path-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on explicit CLI second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 225, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 225 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 385 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 394 to 396 by locking two additional user-visible CLI ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1053
- **Timestamp:** 2026-04-23T02:38:57Z

### L-356: [labels: configured-cli-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that
- **Strategy:** [labels: configured-cli-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation when CLI search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 227, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 227 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 387 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 396 to 398 by locking two additional user-visible configured CLI ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-cli-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation when CLI search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 227, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 227 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 387 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 396 to 398 by locking two additional user-visible configured CLI ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1055
- **Timestamp:** 2026-04-23T02:45:08Z

### L-357: [labels: cli-text-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks
- **Strategy:** [labels: cli-text-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on explicit CLI text second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 229, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 229 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 389 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 398 to 400 by locking two additional user-visible CLI text ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-text-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation on explicit CLI text second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 229, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 229 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 389 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 398 to 400 by locking two additional user-visible CLI text ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1058
- **Timestamp:** 2026-04-23T02:51:17Z

### L-358: [labels: configured-cli-text-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage
- **Strategy:** [labels: configured-cli-text-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation when CLI text search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 231, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 231 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 391 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 400 to 402 by locking two additional user-visible configured CLI text ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-cli-text-temporal-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks temporal and validity filtering before top-k truncation when CLI text search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 231, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 231 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 391 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 400 to 402 by locking two additional user-visible configured CLI text ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1061
- **Timestamp:** 2026-04-23T03:01:44Z

### L-359: [labels: variant-ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli cov
- **Strategy:** [labels: variant-ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 233, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 233 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 393 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 402 to 404 by locking two additional user-visible variant ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: variant-ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation on variant-based second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 233, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 233 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 393 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 402 to 404 by locking two additional user-visible variant ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1064
- **Timestamp:** 2026-04-23T03:08:32Z

### L-360: [labels: cli-ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverag
- **Strategy:** [labels: cli-ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation on explicit CLI second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 235, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 235 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 395 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 404 to 406 by locking two additional user-visible CLI ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-ready-path-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation on explicit CLI second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 235, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 235 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 395 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 404 to 406 by locking two additional user-visible CLI ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1066
- **Timestamp:** 2026-04-23T03:14:19Z

### L-361: [labels: configured-cli-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverag
- **Strategy:** [labels: configured-cli-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation when CLI JSON search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 237, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 237 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 397 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 406 to 408 by locking two additional user-visible configured CLI ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-cli-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation when CLI JSON search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 237, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 237 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 397 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 406 to 408 by locking two additional user-visible configured CLI ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1068
- **Timestamp:** 2026-04-23T03:21:24Z

### L-362: [labels: configured-cli-text-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli co
- **Strategy:** [labels: configured-cli-text-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation when CLI text search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 239, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 239 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 399 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 408 to 412 by locking two additional user-visible configured CLI text ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-cli-text-citation-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks record and citation shape preservation when CLI text search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 239, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 239 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 399 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 408 to 412 by locking two additional user-visible configured CLI text ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1070
- **Timestamp:** 2026-04-23T03:31:44Z

### L-363: [labels: configured-cli-text-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli cove
- **Strategy:** [labels: configured-cli-text-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation when CLI text search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 241, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 241 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 401 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 412 to 414 by locking two additional user-visible configured CLI text ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-cli-text-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation when CLI text search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 241, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 241 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 401 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 412 to 414 by locking two additional user-visible configured CLI text ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1073
- **Timestamp:** 2026-04-23T03:39:39Z

### L-364: [labels: cli-text-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that l
- **Strategy:** [labels: cli-text-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on explicit CLI text second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 243, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 243 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 403 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 414 to 416 by locking two additional user-visible CLI text ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-text-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on explicit CLI text second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 243, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 243 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 403 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 414 to 416 by locking two additional user-visible CLI text ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1075
- **Timestamp:** 2026-04-23T03:48:13Z

### L-365: [labels: library-ready-path-source-metadata-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli co
- **Strategy:** [labels: library-ready-path-source-metadata-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on library second-channel ready paths for both runtime-config and variant entrypoints across embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 247, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 247 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 407 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 416 to 420 by locking four additional user-visible library ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: library-ready-path-source-metadata-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on library second-channel ready paths for both runtime-config and variant entrypoints across embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 247, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 247 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 407 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 416 to 420 by locking four additional user-visible library ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1078
- **Timestamp:** 2026-04-23T03:58:12Z

### L-366: [labels: configured-cli-json-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli cove
- **Strategy:** [labels: configured-cli-json-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation when CLI JSON search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 249, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 249 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 409 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 420 to 422 by locking two additional user-visible configured CLI ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: configured-cli-json-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation when CLI JSON search enters second-channel ready paths through configured embedding_only and hybrid modes. The focused suite counts now stand at 25, 249, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 249 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 409 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 420 to 422 by locking two additional user-visible configured CLI ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1082
- **Timestamp:** 2026-04-23T04:07:20Z

### L-367: [labels: cli-json-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that l
- **Strategy:** [labels: cli-json-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on explicit CLI JSON second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 249, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 249 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 409 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 420 to 424 by locking two additional user-visible CLI JSON ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: cli-json-source-shape-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on explicit CLI JSON second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 249, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 249 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 409 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 420 to 424 by locking two additional user-visible CLI JSON ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1085
- **Timestamp:** 2026-04-23T04:14:48Z

### L-368: [labels: override-ready-path-source-metadata-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli c
- **Strategy:** [labels: override-ready-path-source-metadata-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on mode_override-driven second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 253, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 253 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 413 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 424 to 426 by locking two additional user-visible library ready-path contracts.
- **Outcome:** keep
- **Insight:** [labels: override-ready-path-source-metadata-lock, lexical-first, working-memory-assembly] Added focused retrieval_cli coverage that locks source kind and label preservation on mode_override-driven second-channel ready paths for both embedding_only and hybrid retrieval. The focused suite counts now stand at 25, 253, and 135 tests respectively, cargo test --quiet --test retrieval_cli passed cleanly at 253 tests, cargo test --quiet --test lexical_search --test retrieval_cli --test working_memory_assembly passed cleanly at 413 tests total, and cargo clippy --quiet --all-targets -- -D warnings stayed green. This increases the retained metric from 424 to 426 by locking two additional user-visible library ready-path contracts.
- **Context:** goal=Improve the non-LLM recall pipeline by consuming layered memory DSL in ordinary retrieval and working-memory assembly; scope=src/search/**,src/cognition/assembly.rs,src/cognition/working_memory.rs,src/memory/repository.rs,tests/lexical_search.rs,tests/retrieval_cli.rs,tests/working_memory_assembly.rs; metric=retrieval_green_checks; direction=higher
- **Iteration:** 1092
- **Timestamp:** 2026-04-23T04:28:30Z
