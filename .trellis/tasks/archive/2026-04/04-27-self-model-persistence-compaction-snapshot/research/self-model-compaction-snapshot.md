# Self-Model Compaction Snapshot Notes

## Why this phase exists

The self-model persistence layer now has a durable ledger-first read model with lifecycle core rules. The next step is to make that substrate more durable and recoverable by adding compaction/snapshot behavior before governance/conflict review lands.

## Constraints

- Preserve the current self-model projection seam.
- Preserve lifecycle-core semantics already established.
- Do not widen into governance or external surfaces in this phase.
