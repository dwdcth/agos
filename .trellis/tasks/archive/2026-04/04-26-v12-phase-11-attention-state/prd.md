# v1.2 Phase 11 Attention State

## Goal

Introduce an explicit attention-state layer that sits between user/task state and ordinary retrieval, so recall is no longer driven only by `query + filters`. The new layer should preserve the lexical-first baseline while making retrieval and working-memory assembly sensitive to goal, risk, readiness, and metacognitive bias in an explainable way.

## Requirements

- Add typed attention-state structures for a stable session-level baseline and a per-request delta.
- Allow working-memory and agent-search requests to carry attention-state inputs without breaking existing call sites.
- Make ordinary retrieval consume attention-derived bias in a deterministic and explainable way.
- Preserve lexical-first retrieval as the primary explanation surface; attention may change ordering and trace, but not bypass lexical-first semantics.
- Surface attention influence in structured traces so callers can inspect why retrieval changed.
- Keep the implementation local-first, deterministic, and testable without LLM dependencies.

## Acceptance Criteria

- [ ] There is a typed `AttentionState` contract in the Rust codebase with explicit baseline and delta substructures.
- [ ] Retrieval requests can carry attention-state input end to end from CLI/library-facing request objects into the retrieval path.
- [ ] Attention state can influence ranking or rerank deterministically for at least goal, risk, and metacognitive bias cases.
- [ ] Retrieval traces expose the attention-derived contribution in a way that is inspectable in tests.
- [ ] Existing lexical-only behavior remains valid when no attention state is provided.
- [ ] Integration tests cover both default/no-attention behavior and biased-attention behavior.

## Definition of Done

- Feature implemented in production code.
- New/updated tests pass.
- `cargo clippy --all-targets -- -D warnings` passes.
- No semantic-only or model-dependent path is introduced.

## Technical Approach

- Add a new cognition attention module with explicit baseline/delta types plus a lightweight derived bias object that retrieval can consume.
- Extend `SearchRequest` with optional attention-state input instead of introducing a second retrieval API.
- Apply attention influence in the Rust-side scoring/rerank path, where traceability is already strongest.
- Thread attention state through working-memory assembly and agent-search so higher layers can provide the new control signal without rewriting their top-level contracts.
- Keep v1.2 Phase 11 intentionally narrow: no emotion engine, no long-lived session manager, and no full attention policy runtime yet.

## Decision (ADR-lite)

**Context**: The theory documents require attention to be a first-class control layer, but the current codebase only uses query, filters, active goal, and risk as scattered inputs.

**Decision**: Implement explicit attention state as a typed additive signal that affects existing retrieval ranking/tracing rather than replacing the current retrieval pipeline.

**Consequences**:
- This keeps backward compatibility and lexical-first explainability.
- It creates a clean seam for later self-model and metacognitive modifiers.
- It does not yet implement the full theory-level emotion modulation or session-level attention manager.

## Out of Scope

- Full world-model projection work.
- Full self-model promotion from local adaptations to a first-class store.
- Skill-memory persistence and reuse.
- MCP / HTTP surfaces.
- Embedding lifecycle / rebuild tooling.

## Technical Notes

- Theory references:
  - `doc/0415-注意力状态.md`
  - `doc/0415-工作记忆.md`
  - `doc/0415-元认知层.md`
  - `doc/0415-自我模型.md`
- Current implementation seams likely affected:
  - `src/search/mod.rs`
  - `src/search/score.rs`
  - `src/search/rerank.rs`
  - `src/cognition/assembly.rs`
  - `src/agent/orchestration.rs`
  - `src/interfaces/cli.rs`
- Current invariants to preserve:
  - lexical-first explanation remains primary
  - retrieval still works with no LLM and no embedding backend
  - existing tests for lexical search, retrieval CLI, working-memory assembly, and agent-search should remain green
