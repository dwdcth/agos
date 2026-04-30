# Self-Model Persistence Read Model Lifecycle Core

## Goal

Implement an internal-only self-model persistence phase on top of the existing `local_adaptation_entries` ledger. This phase should turn the ledger into a durable self-model read path with explicit lifecycle-core rules, without adding a new storage schema or exposing a new operator surface.

## Requirements

- Reuse `local_adaptation_entries` as the durable persistence substrate.
- Build an explicit self-model read model over that ledger rather than reading entries ad hoc inside working-memory assembly.
- Define lifecycle-core behavior for persisted self-model state, including at least:
  - overwrite / precedence rules for repeated writes to the same logical key
  - active vs inactive state rules
  - deterministic merge / read order across `SelfState`, `RiskBoundary`, and `PrivateT3`
  - basic conflict resolution semantics
- Keep the phase internal-only:
  - no new CLI/inspect surface
  - no MCP/HTTP/API addition
- Keep scope self-model-only:
  - no world-model persistence
  - no skill-memory persistence
- Preserve the current explicit seams:
  - attention retrieval contracts
  - self-model projection contracts
  - world-model projection contracts
  - skill-memory projection contracts

## Acceptance Criteria

- [ ] There is an explicit self-model persistence/read-model seam layered over `local_adaptation_entries`.
- [ ] Working-memory self-state assembly reads through that seam instead of directly consuming raw ledger rows.
- [ ] Lifecycle rules are deterministic and covered by tests for overwrite, precedence, and active/inactive behavior.
- [ ] Existing short-cycle rumination write paths still work against the same storage substrate.
- [ ] No new storage schema is introduced.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Definition of Done

- Production code updated.
- Relevant tests added or updated.
- Current outward contracts remain stable.
- No new inspection/API surface is added in this phase.

## Technical Approach

- Add an internal self-model persistence/read-model service around `local_adaptation_entries`.
- Separate raw ledger rows from the aggregated self-model state consumed by cognition.
- Keep `ProjectedSelfModel` as the downstream self-model seam, but make it consume a durable aggregated state rather than just a raw row list.
- Avoid schema changes; lifecycle is encoded in deterministic aggregation rules over the existing ledger.

## Out of Scope

- New SQLite tables or migrations
- Snapshot/compaction storage redesign
- World-model or skill-memory persistence
- New CLI / inspect / HTTP / MCP surfaces

## Technical Notes

- Existing durable substrate:
  - `src/memory/repository.rs` → `insert_local_adaptation_entry`, `list_local_adaptation_entries`
- Existing self-model seam:
  - `src/cognition/self_model.rs`
  - `src/cognition/assembly.rs`
- Existing write path:
  - `src/cognition/rumination.rs`
