# Self-Model Persistence Read Model Notes

## Why this phase exists

The codebase already persists short-cycle adaptive state in `local_adaptation_entries`, but cognition still treats that ledger mostly as raw overlay input. The next step is to convert it into a durable self-model read path with explicit lifecycle rules.

## Current substrate

- Durable writes already exist through rumination-triggered `LocalAdaptationEntry` persistence.
- Target kinds already distinguish:
  - `self_state`
  - `risk_boundary`
  - `private_t3`
- The self-model seam is explicit, but its persistence source is not yet modeled as a read model.

## Foundation constraints

- No new schema in this phase.
- No external inspection surface in this phase.
- No world-model or skill-memory persistence in this phase.
- The main value is deterministic aggregation and lifecycle semantics over the existing ledger.
