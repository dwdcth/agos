# Pitfalls Research

**Domain:** Agent memory cognition engine
**Researched:** 2026-04-15
**Confidence:** MEDIUM

## Critical Pitfalls

### Pitfall 1: Collapsing retrieval and cognition into one opaque loop

**What goes wrong:**
The system becomes "LLM chats over memory" instead of a memory cognition engine. Ordinary retrieval becomes untestable, and agent search cannot explain its decisions.

**Why it happens:**
It is faster to demo one prompt loop than to define clear service boundaries.

**How to avoid:**
Keep ordinary retrieval as a deterministic service and make agent search consume it as a separate orchestration layer.

**Warning signs:**
- No standalone search API can be tested without an LLM
- Search ranking logic lives inside prompt templates
- Citations disappear once agent mode is enabled

**Phase to address:**
Phase 1 and Phase 3

---

### Pitfall 2: Mixing T1, T2, and T3 in the same undifferentiated record model

**What goes wrong:**
Private hypotheses, shared evidence, and ontology-level structures contaminate each other, making truth promotion and rollback unsafe.

**Why it happens:**
Teams often start with a generic `memory` table and try to bolt truth semantics on later.

**How to avoid:**
Introduce truth-layer metadata and promotion rules early, even if the first version is minimal.

**Warning signs:**
- The same table stores ontology, evidence, and private guesses with no structural distinction
- No clear write rule exists for "shared" vs "private" memory
- Corrections require ad hoc manual cleanup

**Phase to address:**
Phase 1 and Phase 2

---

### Pitfall 3: Relying on vector search alone

**What goes wrong:**
Exact decisions, identifiers, Chinese terms, and symbolic cues are missed. Users lose trust when obvious lexical hits do not show up.

**Why it happens:**
Vector search feels modern and simpler to market.

**How to avoid:**
Ship hybrid retrieval from the start: lexical (`libsimple` FTS5) + semantic (`sqlite-vec`) + fusion/rerank.

**Warning signs:**
- Queries with exact names or Chinese abbreviations fail
- Users start phrasing queries unnaturally to satisfy embeddings
- Search quality depends too much on paraphrasing

**Phase to address:**
Phase 1

---

### Pitfall 4: Letting agent search write shared truth without verification

**What goes wrong:**
Hallucinated summaries or one-off successes become durable memory, poisoning later retrieval.

**Why it happens:**
It feels efficient to let the agent both search and write back immediately.

**How to avoid:**
Separate retrieval, answer synthesis, and write-back. Require evidence, outcome, and metacognitive checks before promotion.

**Warning signs:**
- Search results are summarized and persisted with no source links
- There is no "pending promotion" state
- T2/T1 updates happen during normal chat without explicit gates

**Phase to address:**
Phase 2 and Phase 4

---

### Pitfall 5: Treating working memory as a cache instead of a control field

**What goes wrong:**
The system cannot compare candidate actions, propagate risks, or carry self-state into decisions. Agent search becomes expensive retrieval with nicer wording.

**Why it happens:**
Developers equate "top-k results" with "the current context".

**How to avoid:**
Build explicit working-memory structures containing world fragments, self state, active goal, risks, candidate actions, and metacognitive flags.

**Warning signs:**
- Working memory is just `Vec<SearchResult>`
- No place exists for candidate actions or veto flags
- Metacognition has no concrete data target to modify

**Phase to address:**
Phase 3

---

### Pitfall 6: Designing rumination as a background batch job only

**What goes wrong:**
Critical corrections arrive too late, while important slow-learning tasks compete with urgent fixes in the same queue.

**Why it happens:**
Background jobs feel operationally simple.

**How to avoid:**
Respect the 0415 split between short-cycle and long-cycle rumination, with different triggers and write targets.

**Warning signs:**
- User correction does not affect the next step
- All learning tasks share the same queue and priority
- Write-back conflicts become common

**Phase to address:**
Phase 4

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Single generic memory table with JSON blob payloads | Fast prototype | Truth-layer and retrieval semantics become implicit and brittle | Only for throwaway spikes, not for v1 |
| Agent-only search path | Faster demo | No deterministic test surface for retrieval | Never for this project |
| No source / timestamp metadata in search results | Simpler schema | Impossible to debug stale or wrong memory | Never |
| Direct DB writes from interface handlers | Less scaffolding | Invariants spread across the codebase | Never |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `sqlite-vec` | Forgetting explicit extension initialization | Load and register the extension during DB bootstrap before vector queries |
| `libsimple` | Assuming default SQLite tokenizer behavior is enough | Explicitly configure the tokenizer and dictionary path for Chinese/PinYin recall |
| `rig` | Letting framework abstractions dictate the core memory model | Keep Rig at the orchestration boundary and adapt internal services outward |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Re-embedding everything on every write | Slow ingest, growing latency | Batch embedding and selective reindexing | Medium corpus size and above |
| Full-table fusion in memory | Search latency jumps with corpus size | Scope/filter early and cap candidate sets before rerank | Once thousands of memory records accumulate |
| Excessive prompt-driven agent search retries | Expensive agent mode with unstable answers | Bound step count and keep deterministic search cheap | As soon as agent search is exposed interactively |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Persisting sensitive context without scope controls | Unintended recall of secrets or private notes | Add record types, scope filters, and explicit ingest policies |
| Auto-executing write-back from model output | Memory poisoning | Require structured write commands with source links and verification |
| Treating citations as optional formatting | Trust failure and unsafe decisions | Make citations part of the data contract, not a UI nicety |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Search results without explanation | Users cannot tell why a result appeared | Show lexical/vector/rerank cues and source metadata |
| Agent search that hides intermediate retrieval | Users cannot debug incorrect answers | Expose searched scopes, cited evidence, and rejected paths |
| Stale memory shown as current truth | Users stop trusting the system | Surface timestamps, validity state, and truth layer clearly |

## "Looks Done But Isn't" Checklist

- [ ] **Hybrid search:** Often missing lexical ranking or Chinese tokenization — verify exact-term recall and semantic recall both work.
- [ ] **Agent search:** Often missing citation preservation — verify the final answer still carries sources.
- [ ] **Truth layers:** Often missing rollback or pending-promotion states — verify wrong promotions can be contained.
- [ ] **Working memory:** Often missing candidate actions and risks — verify it is more than a result list.
- [ ] **Rumination:** Often missing queue separation — verify short-cycle and long-cycle updates do not share one blind worker.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Retrieval/cognition collapse | HIGH | Extract deterministic search service, move prompt logic outward, rebuild tests |
| Truth-layer pollution | HIGH | Add typed states, mark contaminated records, re-run promotion review |
| Vector-only search gap | MEDIUM | Add lexical index, backfill FTS tokens, retune fusion |
| Unsafe write-back | HIGH | Freeze promotion, add pending-review path, audit recent writes |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Retrieval/cognition collapse | Phase 1 and 3 | Search tests pass without LLM involvement |
| Truth-layer pollution | Phase 1 and 2 | Promotion invariants and record typing are enforced |
| Vector-only gap | Phase 1 | Exact-match and Chinese recall regression tests exist |
| Unsafe write-back | Phase 2 and 4 | Shared-truth writes require evidence and explicit gate checks |
| Working-memory-as-cache | Phase 3 | Working-memory object contains goals, risks, and candidate actions |
| Batch-only rumination | Phase 4 | SPQ and LPQ are separated and independently testable |

## Sources

- `doc/0415-真值层.md`
- `doc/0415-工作记忆.md`
- `doc/0415-元认知层.md`
- `doc/0415-反刍机制.md`
- `reference/mempal/README_zh.md`
- https://github.com/asg017/sqlite-vec
- https://docs.rs/crate/libsimple/latest

---
*Pitfalls research for: agent memory cognition engine*
*Researched: 2026-04-15*
