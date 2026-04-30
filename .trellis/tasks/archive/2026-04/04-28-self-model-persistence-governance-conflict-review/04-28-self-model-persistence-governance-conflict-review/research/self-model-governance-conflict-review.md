# Self-Model Governance Conflict Review Notes

## Why this phase exists

The self-model persistence layer already has:

- a ledger-first read model
- lifecycle core semantics
- snapshot + compaction support

What it still lacks is an explicit governance/conflict review layer for materially incompatible self-model writes.

## Constraints

- Stay self-model-only.
- Stay internal-only.
- Preserve the existing `ProjectedSelfModel -> SelfStateSnapshot` seam.
- Build on top of the current ledger + snapshot + read-model substrate rather than redesigning storage.
