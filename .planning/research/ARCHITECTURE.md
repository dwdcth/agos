# Architecture Research

**Domain:** Rust memory cognition engine for agent search
**Researched:** 2026-04-15
**Confidence:** MEDIUM

## Standard Architecture

### System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    Interface Layer                          │
├─────────────────────────────────────────────────────────────┤
│  CLI / Library API / Optional HTTP or MCP surface          │
├─────────────────────────────────────────────────────────────┤
│                 Application Service Layer                   │
├─────────────────────────────────────────────────────────────┤
│  Ingest   Retrieval   Agent Search   Rumination   Admin     │
├─────────────────────────────────────────────────────────────┤
│                    Cognitive Core Layer                     │
├─────────────────────────────────────────────────────────────┤
│  Truth Layers  World  Self  Skill  Attention               │
│  Working Memory  Value  Metacognition                      │
├─────────────────────────────────────────────────────────────┤
│                   Storage / Infra Layer                     │
├─────────────────────────────────────────────────────────────┤
│ SQLite schema  FTS5/libsimple  optional sqlite-vec         │
│ traces  migration  bonus rules  background jobs            │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `core` | Own config, DB access, types, IDs, migrations, shared errors | Similar to `reference/mempal/src/core` |
| `memory` | Own typed memory schema, truth layers, promotion rules | Domain-specific models and repository services |
| `search` | Own lexical retrieval, lightweight scoring, optional semantic extension, filters, citations | Lexical-first query pipeline over SQLite |
| `agent` | Own Rig integration, tool routing, multi-step retrieval orchestration | Rig agents and tool wrappers over internal services |
| `cognition` | Own attention, working memory, value, metacognition, rumination logic | Pure domain modules with minimal IO |
| `interfaces` | Expose CLI / API / MCP without owning domain logic | Thin handlers calling services |

## Recommended Project Structure

```text
src/
├── lib.rs                    # public module graph
├── main.rs                   # CLI entry
├── core/                     # config, db, types, ids, errors
│   ├── config.rs
│   ├── db.rs
│   ├── ids.rs
│   ├── types.rs
│   └── mod.rs
├── memory/                   # memory entities and truth layers
│   ├── record.rs
│   ├── truth.rs
│   ├── repository.rs
│   ├── promote.rs
│   └── mod.rs
├── ingest/                   # source detection, normalization, chunking
│   ├── detect.rs
│   ├── normalize.rs
│   ├── chunk.rs
│   └── mod.rs
├── search/                   # lexical-first retrieval
│   ├── lexical.rs
│   ├── score.rs
│   ├── bonus.rs
│   ├── rerank.rs
│   ├── citation.rs
│   ├── semantic.rs           # optional sqlite-vec extension
│   └── mod.rs
├── cognition/                # 0415 cognitive core
│   ├── attention.rs
│   ├── working_memory.rs
│   ├── world.rs
│   ├── self_model.rs
│   ├── skill.rs
│   ├── value.rs
│   ├── metacog.rs
│   ├── rumination.rs
│   └── mod.rs
├── agent/                    # rig orchestration
│   ├── tools.rs
│   ├── ordinary_search.rs
│   ├── agent_search.rs
│   ├── planner.rs
│   └── mod.rs
└── interfaces/               # CLI / HTTP / MCP
    ├── cli.rs
    ├── api.rs
    ├── mcp.rs
    └── mod.rs
```

### Structure Rationale

- **`core/`:** keep storage primitives and shared infrastructure stable early, as `mempal` does.
- **`memory/`:** truth layers and typed records deserve their own boundary; they are not just `search` metadata.
- **`search/`:** ordinary retrieval must stay deterministic and testable, separate from LLM orchestration.
- **`cognition/`:** the 0415 model is the domain center of gravity and should remain mostly pure logic.
- **`agent/`:** Rig integration should adapt internal services, not replace them.
- **`interfaces/`:** delivery surfaces should be thin so the project can evolve from CLI to API without rewrites.

## Architectural Patterns

### Pattern 1: Storage-Owned Domain Model

**What:** Own the SQLite schema and repositories directly instead of outsourcing the primary model to an agent framework.
**When to use:** From the first implementation phase.
**Trade-offs:** More upfront schema design, but much better control over truth layers, traces, and migrations.

### Pattern 2: Deterministic Search, Agentic Composition

**What:** Ordinary retrieval returns scored, cited, filtered results; agent search consumes those services for multi-step search and reasoning.
**When to use:** Always, if you need explainability and stable tests.
**Trade-offs:** Slightly more code, but avoids opaque "LLM decides everything" failure modes.

### Pattern 3: Side-Effect Isolation for Cognitive Logic

**What:** Attention, working-memory assembly, metacognition, and rumination operate on typed inputs/outputs with minimal direct DB or network calls.
**When to use:** When the domain model is still evolving and needs fast tests.
**Trade-offs:** Requires more adapters, but keeps cognitive semantics legible.

## Data Flow

### Request Flow

```text
User / Agent Query
    ↓
Interface handler
    ↓
Search service
    ├── lexical retrieval (FTS5/libsimple)
    ├── Rust lightweight scoring / bonus rules
    ├── optional semantic retrieval (sqlite-vec)
    └── citations / trace assembly
    ↓
Ordinary search result
    ↓
optional: agent search orchestration (Rig)
    ↓
working memory assembly + validated answer / action support
```

### State Management

```text
Typed memory records
    ↓
truth-layer repositories
    ↓
search services
    ↓
cognitive assembly
    ↓
action outcomes / corrections
    ↓
rumination queues
    ↓
write-back to self / skill / world / truth promotion candidates
```

### Key Data Flows

1. **Ingest flow:** source material → normalization → chunking / typing → lexical indexing and metadata persistence → SQLite persisted memory records.
2. **Ordinary retrieval flow:** query → lexical recall → BM25/TF-IDF-style weighting and bonus scoring → filters → citations.
3. **Agent search flow:** user goal → ordinary retrieval calls → follow-up retrieval / verification → working memory package.
4. **Learning flow:** action result / correction → SPQ or LPQ routing → bounded write-back.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| single-user / local machine | Single binary + single SQLite DB is sufficient |
| larger personal corpus | Add better indexing, archiving, and batch re-embedding jobs |
| team-shared deployment | Add service wrappers and sync strategy only after the schema and truth semantics are stable |

### Scaling Priorities

1. **First bottleneck:** retrieval quality, not raw database throughput.
2. **Second bottleneck:** schema evolution and write-back correctness, not API fan-out.

## Anti-Patterns

### Anti-Pattern 1: Search-Layer God Module

**What people do:** Put schema, retrieval, agent prompting, and write-back in one service.
**Why it's wrong:** It destroys testability and makes it impossible to reason about truth promotion.
**Do this instead:** Keep storage, ordinary retrieval, agent orchestration, and rumination as separate modules.

### Anti-Pattern 2: Treat Working Memory as a Search Result List

**What people do:** Rename `top_k` results to "working memory".
**Why it's wrong:** The 0415 model defines working memory as a control field with world fragments, self state, goals, risks, candidate actions, and metacognitive flags.
**Do this instead:** Build an explicit working-memory assembly step.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| LLM / embedding providers | Through Rig provider abstractions | Keep provider-specific logic out of the memory core. |
| SQLite extensions | Loaded during DB initialization | `libsimple` is baseline; `sqlite-vec` should stay behind an optional path if enabled later. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `search ↔ memory` | Repository + typed query models | Retrieval should not know migration internals. |
| `agent ↔ search` | Service API | Agent search should compose ordinary search, not duplicate it. |
| `cognition ↔ memory` | Typed snapshots and write-back commands | Keep domain semantics explicit. |
| `interfaces ↔ services` | Thin handlers | Preserve portability across CLI/API/MCP. |

## Sources

- `reference/mempal/src/lib.rs`
- `reference/mempal/src/main.rs`
- `reference/mempal/src/core/db.rs`
- `reference/mempal/src/ingest/mod.rs`
- `reference/mempal/src/search/mod.rs`
- `reference/mempal/src/search/route.rs`
- `reference/mempal/src/mcp/tools.rs`
- `doc/0415-00记忆认知架构.md`
- `doc/0415-工作记忆.md`
- `doc/0415-反刍机制.md`

---
*Architecture research for: Rust memory cognition engine*
*Researched: 2026-04-15*
