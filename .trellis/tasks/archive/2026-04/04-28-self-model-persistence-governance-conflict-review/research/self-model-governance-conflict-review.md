# Self-Model Governance Conflict Review Notes

## Why this phase exists

The self-model persistence layer now has:

- a ledger-first read model
- lifecycle core semantics
- snapshot/compaction support

What it still lacks is explicit governance for conflicting self-model writes. Right now, precedence rules are enough for routine updates, but not for materially incompatible state.

## Constraints

- Stay self-model-only.
- Preserve the existing projection seam.
- Keep the phase internal-only.
- Add explicit conflict/review semantics without widening into external review UX.
