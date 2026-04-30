# Bootstrap Guidelines

## Goal

Populate `.trellis/spec/backend/` with the project's actual Rust backend conventions so future `trellis-implement` and `trellis-check` agents stop operating on placeholders and instead load repo-specific guidance with real examples.

## What I already know

- This repo already has meaningful coding signals in `AGENTS.md`, `Cargo.toml`, `src/`, and `tests/`, but `.trellis/spec/backend/*.md` are still stock templates.
- The backend is a single Rust crate with a single binary entrypoint (`src/main.rs`) and modular domain slices under `src/agent`, `src/cognition`, `src/core`, `src/ingest`, `src/interfaces`, `src/memory`, and `src/search`.
- Database access is SQLite-first through `rusqlite` and `rusqlite_migration`, with additive numbered SQL migrations embedded via `include_str!` in [`src/core/migrations.rs`](../../../src/core/migrations.rs).
- Error handling already uses typed domain errors with `thiserror` for lower layers and `anyhow::Result` at CLI/app boundaries; see [`src/core/db.rs`](../../../src/core/db.rs) and [`src/interfaces/cli.rs`](../../../src/interfaces/cli.rs).
- Testing style is integration-heavy under `tests/`, with real SQLite bootstraps and CLI subprocess assertions; see [`tests/foundation_schema.rs`](../../../tests/foundation_schema.rs), [`tests/status_cli.rs`](../../../tests/status_cli.rs), and [`tests/retrieval_cli.rs`](../../../tests/retrieval_cli.rs).
- This task is still not workflow-ready for sub-agents because `implement.jsonl` and `check.jsonl` have not been created yet.

## Assumptions

- The current codebase is the source of truth for conventions, even if some patterns are still evolving.
- For bootstrap, documenting current reality is more important than prescribing aspirational architecture changes.
- English remains the required language for `.trellis/spec/backend/*.md`.

## Open Questions

- Should this bootstrap task be completed now by writing the backend spec files from existing code patterns, or intentionally left paused while product work continues elsewhere?

## Requirements

- Fill all five backend spec files in `.trellis/spec/backend/`.
- Replace placeholder content with repo-specific conventions.
- Include real code examples and file references from this repository.
- Capture actual patterns for directory structure, database usage, error handling, logging expectations, and quality/testing.
- Create `implement.jsonl` and `check.jsonl` entries once the relevant spec files exist so future sub-agents can load them.

## Acceptance Criteria

- [ ] `.trellis/spec/backend/directory-structure.md` documents real module layout and names concrete example files.
- [ ] `.trellis/spec/backend/database-guidelines.md` documents SQLite, `rusqlite`, migration, and naming patterns from the repo.
- [ ] `.trellis/spec/backend/error-handling.md` documents typed error usage, propagation, and boundary behavior with examples.
- [ ] `.trellis/spec/backend/logging-guidelines.md` reflects the repo's current logging reality, including any current gaps.
- [ ] `.trellis/spec/backend/quality-guidelines.md` documents testing and review expectations using current tests and toolchain.
- [ ] Each filled guideline includes concrete file references rather than placeholders.
- [ ] `implement.jsonl` and `check.jsonl` are curated with the spec files needed by future implement/check agents.

## Definition of Done

- Backend spec files are filled with real examples.
- Placeholder "To fill" content is removed from backend spec files and index status is updated.
- Task context manifests for implement/check are present and usable.
- No product code behavior changes are introduced as part of the bootstrap documentation pass.

## Out of Scope

- Defining the next product milestone.
- Resuming paused autoresearch loops.
- Refactoring runtime code just to make the spec look cleaner.
- Filling frontend, deployment, or multi-package guidelines that do not exist in this repo.

## Technical Notes

- Existing convention source: [`AGENTS.md`](../../../AGENTS.md)
- Crate and dependency source: [`Cargo.toml`](../../../Cargo.toml)
- Directory examples:
  - [`src/core/app.rs`](../../../src/core/app.rs)
  - [`src/core/db.rs`](../../../src/core/db.rs)
  - [`src/interfaces/cli.rs`](../../../src/interfaces/cli.rs)
  - [`src/search/lexical.rs`](../../../src/search/lexical.rs)
  - [`src/memory/repository.rs`](../../../src/memory/repository.rs)
- Test examples:
  - [`tests/foundation_schema.rs`](../../../tests/foundation_schema.rs)
  - [`tests/status_cli.rs`](../../../tests/status_cli.rs)
  - [`tests/retrieval_cli.rs`](../../../tests/retrieval_cli.rs)
- Current gap summary:
  - `.trellis/spec/backend/index.md` still marks every guide as `To fill`
  - All five backend guideline files still contain placeholder text
  - `.trellis/tasks/00-bootstrap-guidelines/implement.jsonl` is missing
  - `.trellis/tasks/00-bootstrap-guidelines/check.jsonl` is missing
