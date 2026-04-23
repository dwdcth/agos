# Phase 9: Dual-Channel Retrieval Fusion - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 8
**Analogs found:** 8 / 8

## Revision Notes

- Phase 9 should extend the existing `SearchService` pipeline, not fork it.
- The new requirement around `config.toml` means config parsing and test-fixture generation are now part of the core pattern map.
- Fusion should reuse current result/citation contracts wherever possible and add trace fields additively.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/search/mod.rs` | service | request/response | `src/search/mod.rs` | exact |
| `src/search/lexical.rs` | recall | candidate generation | `src/search/lexical.rs` | exact |
| `src/search/score.rs` | scoring | transform | `src/search/score.rs` | exact |
| `src/search/rerank.rs` | shaping/trace | transform | `src/search/rerank.rs` | exact |
| `src/core/config.rs` or retrieval-config adapter module | config | parse/derive | `src/core/config.rs` | exact/composite |
| `tests/retrieval_cli.rs` | test | end-to-end | `tests/retrieval_cli.rs` | exact |
| `tests/lexical_search.rs` | test | focused retrieval behavior | `tests/lexical_search.rs` | exact |
| `tests/dual_channel_retrieval.rs` (likely new) | test | config matrix + fusion | `tests/retrieval_cli.rs` + `tests/lexical_search.rs` | composite |

## Pattern Assignments

### `src/search/mod.rs` (service, request/response)

**Analog:** current `SearchService::search(...)`

**Phase 9 application**

- Keep one `SearchService::search(...)` entrypoint.
- Add mode/config-driven branching inside the service or a delegated internal helper.
- Preserve the final `SearchResponse` contract shape and extend it additively where needed.

### `src/search/lexical.rs` (recall, candidate generation)

**Analog:** current typed lexical candidate generation

**Phase 9 application**

- Mirror this pattern for embedding-side candidates rather than inventing a radically different intermediate shape.
- Preserve record identity, snippets, and query-strategy semantics.

### `src/search/score.rs` / `src/search/rerank.rs` (transform, shaping)

**Phase 9 application**

- Fusion should happen before final result shaping, with trace fields extended additively.
- Current lexical score components should remain visible even if embedding contribution is introduced.
- Prefer explicit per-channel contribution fields over flattening everything into one opaque number.

### Config adapter / parser

**Analog:** `src/core/config.rs`

**Phase 9 application**

- Parse the retrieval-relevant subset of the richer root `config.toml`.
- Generate derived runtime variants for tests instead of maintaining unrelated static fixtures.
- Keep the parsing seam narrow and retrieval-focused if the full app config is broader than runtime needs.

### `tests/dual_channel_retrieval.rs`

**Composite analog:** `tests/retrieval_cli.rs` + `tests/lexical_search.rs`

**Phase 9 application**

- Use one parsed base config and derive lexical-only / embedding-only / hybrid variants.
- Assert channel behavior, dedupe, and trace visibility across those variants.
- Keep this file focused on dual-channel behavior, not generic status or ingest tests.

## Anti-Patterns To Avoid

- Building a second search service for embedding-only behavior.
- Using text equality instead of record identity for candidate dedupe.
- Hiding channel contribution in final ranking with no trace fields.
- Hardcoding three independent config fixtures when one parsed config base would do.
