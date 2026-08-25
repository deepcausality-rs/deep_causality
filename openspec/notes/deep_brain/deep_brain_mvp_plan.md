# Deep Brain MVP — Bare Bones Memory Bank with MCP

**Objective:** Get a working memory capture + search system running over MCP as fast as possible, using Grafeo as the storage engine and grafeo-memory's architecture as the blueprint for the Rust port. No epistemic model yet — just the plumbing that proves the stack works and gives you something to experiment with.

**Stack:** `grafeo` (Rust, embedded) + `fastembed-rs` (local embeddings, ONNX) + `rmcp` (MCP server, stdio) + `serde`/`serde_json` + `tokio`

**What this replaces from the original plan:** Diesel, rusqlite, sqlite-vec, pgvector, UltraGraph. All of it. One `cargo add grafeo`.

---

## Pre-work: Source Audit (Day 0)

Before writing any Rust, read the grafeo-memory Python source.

1. Clone `https://github.com/GrafeoDB/grafeo-memory`
2. Read these files in order:
   - `src/grafeo_memory/config.py` — MemoryConfig, defaults, scoring weights
   - `src/grafeo_memory/models.py` — MemoryEvent, SearchResult, HistoryEntry types
   - `src/grafeo_memory/embeddings/` — EmbeddingClient protocol, OpenAI/Mistral implementations
   - `src/grafeo_memory/extractor.py` — LLM prompt for fact + entity extraction
   - `src/grafeo_memory/reconciler.py` — LLM prompt for ADD/UPDATE/DELETE/NONE decisions
   - `src/grafeo_memory/executor.py` — GQL mutation generation against GrafeoDB
   - `src/grafeo_memory/manager.py` — MemoryManager orchestration (the main loop)
   - `src/grafeo_memory/search.py` — Vector + graph hybrid search, topology boost
3. Note every GQL query string. These are the exact queries you'll use in Rust — Grafeo's Rust API takes the same GQL strings.
4. Note the graph schema: what node labels, edge types, and properties grafeo-memory creates. This is your starting data model.

**Deliverable:** A markdown file listing every GQL query, every node/edge type, and every LLM prompt template from grafeo-memory. This becomes your port checklist.

---

## Confirmed Graph Schema (from grafeo-memory docs)

The grafeo-memory documentation confirms the exact graph structure. This is what
you're porting:

### Node types

| Label | Properties | Role |
|-------|-----------|------|
| `:Memory` | `text`, `embedding`, `user_id`, `memory_type` (semantic/procedural/episodic), `importance`, metadata | Primary knowledge unit |
| `:Entity` | `name`, `entity_type` (person, org, place, etc.) | Extracted named entities |
| `:History` | `event`, `old_text`, `new_text`, `timestamp`, `actor_id`, `role` | Change audit trail |

### Edge types

| Type | From → To | Meaning |
|------|-----------|---------|
| `:HAS_ENTITY` | Memory → Entity | Memory mentions this entity |
| `:RELATION` | Entity → Entity | Named relationship (e.g. "works at", "knows") |
| `:HAS_HISTORY` | Memory → History | Change log link |
| `:DERIVED_FROM` | Memory → Memory | Summary derived from original memories |

### Memory types (confirmed)

- **semantic** — facts, knowledge, preferences (default)
- **procedural** — instructions, rules, preferences
- **episodic** — interaction events, reasoning context

This maps cleanly to Deep Brain's `mem_type` classification. The epistemic extensions
(hypothesis, finding, decision, action_item, definition) are additive — you extend
the `memory_type` enum rather than replacing it.

---

## grafeo-mcp Tool Surface (for reference)

grafeo-mcp (the raw database MCP wrapper) exposes 16 tools. These confirm what
graph algorithms are available natively:

### Already solved by Grafeo (don't rebuild)

| Tool | What it does | Deep Brain equivalent |
|------|-------------|---------------------|
| `query` | Execute arbitrary GQL | Generic escape hatch |
| `vector_search` | k-NN HNSW similarity | `search_memories` core |
| `hybrid_search` | Vector + graph traversal combined | Graph-enriched search |
| `mmr_search` | MMR-diversified search for RAG | De-duped retrieval |
| `create_vector_index` | HNSW index creation | DB bootstrap |
| `pagerank` | PageRank centrality | Gap centrality (future) |
| `shortest_path` | Dijkstra between nodes | Causal chain traversal |
| `community_detection` | Louvain communities | Cross-domain clustering |
| `centrality` | Betweenness + connected components | Assumption blast radius |
| `get_neighbors` | Node neighborhood exploration | `causal_neighborhood` |
| `get_schema` | Schema discovery | Introspection |
| `get_stats` | DB statistics | `memory_stats` |

### What Deep Brain adds on top

| Tool | What Grafeo doesn't have |
|------|--------------------------|
| `capture_memory` | Extract → embed → NER → classify → reconcile → store pipeline |
| `recall_session` | Session-scoped chronological retrieval |
| `invalidate_memory` | Qualified invalidation with assumption tracking |
| `challenge_invalidation` | Assumption dissolution, blast radius propagation |
| `query_gaps` | Gap node traversal with due diligence context |
| `extract_causal_report` | ADLR structured extraction pipeline |
| `link_memories` | Manual causal edge creation (ground truth) |

The grafeo-mcp tools confirm that Grafeo already provides the graph algorithm
infrastructure (PageRank, shortest path, Louvain, centrality, connected components)
that Deep Brain planned to implement via UltraGraph's frozen CsmGraph. These are now
just `CALL grafeo.pagerank(...)` GQL queries — no custom implementation needed.

**Key insight from the `hybrid_search` tool:** Grafeo natively combines vector similarity
with graph traversal in a single operation. This is exactly what Deep Brain's
"graph context enrichment of search results" requires. The original plan had this as
a multi-step process (vector search → get IDs → traverse graph → merge scores).
With Grafeo it's one tool call.

---

## Phase 0 — Skeleton Crate (Day 1)

**Goal:** Cargo project compiles, Grafeo embedded, MCP server starts and responds to `ping`.

```
deep-brain/
├── Cargo.toml
├── src/
│   ├── main.rs          # tokio::main, rmcp server bootstrap
│   ├── db.rs            # GrafeoDB wrapper, init, schema setup
│   ├── tools/           # MCP tool handlers
│   │   ├── mod.rs
│   │   ├── capture.rs   # capture_memory
│   │   └── search.rs    # search_memories
│   ├── embedding.rs     # fastembed-rs wrapper (EmbeddingClient trait)
│   └── types.rs         # MemoryEvent, SearchResult, Config structs
```

### Cargo.toml — Minimal dependencies

```toml
[dependencies]
grafeo           = { version = "...", features = ["gql", "ai"] }
fastembed        = "..."
ort              = "2"
rmcp             = { version = "...", features = ["server", "transport-io"] }
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"
ulid             = "..."
tokio            = { version = "1", features = ["full"] }
```

Note: `grafeo` features `ai` includes HNSW vector search. The `embed` feature adds
ONNX embedding generation inside Grafeo itself — evaluate whether this can replace
`fastembed-rs` entirely. If Grafeo's built-in embedder supports nomic-embed or
equivalent, you can drop `fastembed` and `ort` from deps.

### db.rs — Bootstrap with indexes

```rust
use grafeo::GrafeoDB;

pub struct DeepBrainDB {
    db: GrafeoDB,
}

impl DeepBrainDB {
    pub fn new(path: Option<&str>) -> Self {
        let db = match path {
            Some(p) => GrafeoDB::open(p).expect("failed to open db"),
            None => GrafeoDB::new_in_memory(),
        };

        let mut session = db.session();

        // ── Vector index ──────────────────────────────────────────────
        // HNSW index for semantic similarity search on Memory embeddings.
        // 384 dimensions = nomic-embed-text-v1.5 output.
        session.execute(r#"
            CREATE VECTOR INDEX IF NOT EXISTS memory_embedding_idx
            ON :Memory(embedding)
            DIMENSION 384
            METRIC 'cosine'
        "#).ok();

        // ── Property indexes on :Memory ───────────────────────────────
        // Every WHERE clause in the capture/search pipeline hits one of
        // these. Without them, every filtered query is a full scan.
        // At 1M nodes, that's 50ms per query (benchmarked). With index: 0.01ms.
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Memory(id)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Memory(user_id)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Memory(mem_type)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Memory(memory_type)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Memory(epistemic_state)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Memory(created_at)"
        ).ok();

        // ── Property indexes on :Entity ───────────────────────────────
        // Entity lookups by name are on the critical path for MERGE
        // operations during NER extraction. Unindexed MERGE on name
        // scans all Entity nodes on every capture.
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Entity(name)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Entity(entity_type)"
        ).ok();

        // ── Property indexes on :Session ──────────────────────────────
        // recall_session queries filter by session ID.
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Session(id)"
        ).ok();

        // ── Extension indexes (created early, cost nothing until used) ─
        // These support future epistemic extensions. Creating them now
        // avoids a migration step later. Empty indexes have zero
        // performance cost.
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Gap(id)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Invalidation(id)"
        ).ok();
        session.execute(
            "CREATE PROPERTY INDEX IF NOT EXISTS ON :Assumption(id)"
        ).ok();

        Self { db }
    }

    pub fn session(&self) -> grafeo::Session {
        self.db.session()
    }
}
```

**Index rationale by query pattern:**

| Index | Query pattern it accelerates |
|-------|------------------------------|
| `Memory(id)` | Every point lookup, update, delete, history link |
| `Memory(user_id)` | Multi-user isolation on every search and list |
| `Memory(mem_type)` | Filtered search by memory type |
| `Memory(memory_type)` | grafeo-memory compatible type filter (semantic/procedural/episodic) |
| `Memory(epistemic_state)` | Future: filter active vs superseded vs invalidated |
| `Memory(created_at)` | `browse_recent`, date range filters, session ordering |
| `Entity(name)` | MERGE during NER capture (prevents full scan per entity) |
| `Entity(entity_type)` | Filtered entity queries (persons, orgs, concepts) |
| `Session(id)` | `recall_session` lookup |
| `Gap(id)`, `Invalidation(id)`, `Assumption(id)` | Future extensions — zero cost until populated |

**Performance impact (from Grafeo benchmarks at 1M nodes):**
- Unindexed property lookup: **50ms**
- Indexed property lookup: **0.01ms**
- That's a **5000x improvement** on every filtered query in the pipeline.

### main.rs — MCP bootstrap

```rust
// Scaffold only — wire rmcp server with stdio transport,
// register tool handlers, start event loop.
// Follow rmcp examples for exact API.
```

**Done when:** `cargo run` starts, Claude Desktop (or any MCP client) connects, tool list is visible.

---

## Phase 1 — Capture Pipeline (Days 2–3)

**Goal:** `capture_memory` MCP tool accepts text, generates embedding, stores as a graph node, returns confirmation.

### Step 1: Embedding (embedding.rs)

```rust
pub trait EmbeddingClient: Send + Sync {
    fn embed(&self, texts: &[&str]) -> Vec<Vec<f32>>;
    fn dimensions(&self) -> usize;
}

pub struct LocalEmbedder {
    model: fastembed::TextEmbedding,
}

impl EmbeddingClient for LocalEmbedder {
    fn embed(&self, texts: &[&str]) -> Vec<Vec<f32>> {
        self.model.embed(texts.to_vec(), None).unwrap()
    }
    fn dimensions(&self) -> usize { 384 }
}
```

### Step 2: Capture tool (tools/capture.rs)

The MVP capture skips LLM-based extraction entirely. No fact extraction, no reconciliation, no entity extraction. Just:

1. Accept raw text from MCP call
2. Generate embedding via `LocalEmbedder`
3. Store as `:Memory` node with properties: `id` (ULID), `content`, `embedding`, `created_at`, `user_id`, `source`
4. Return `MemoryEvent { action: ADD, memory_id, text }`

GQL for insert:

```gql
INSERT (:Memory {
    id: $id,
    content: $content,
    embedding: vector($embedding),
    user_id: $user_id,
    source: $source,
    mem_type: $mem_type,
    created_at: $created_at
})
```

**No LLM calls in Phase 1.** You're testing the storage path, not the intelligence.

---

## Phase 2 — Search Pipeline (Days 4–5)

**Goal:** `search_memories` MCP tool accepts a query string, returns ranked results.

### Step 1: Vector search

1. Embed the query text
2. Run HNSW similarity search against `:Memory(embedding)`

grafeo-mcp confirms three search modes are available natively:

- **`vector_search`** — pure k-NN HNSW similarity
- **`hybrid_search`** — vector similarity + graph traversal in one call
- **`mmr_search`** — MMR-diversified for RAG (avoids redundant results)

For the Rust port, these are GQL queries against the same engine. Start with
pure vector search, then switch to hybrid once entity extraction is working
in Phase 3.

```gql
-- Pure vector search (Phase 2)
MATCH (m:Memory)
WHERE m.user_id = $user_id
CALL vector.search(m.embedding, $query_embedding, $k)
YIELD score
RETURN m.id, m.content, m.mem_type, m.created_at, score
ORDER BY score DESC
LIMIT $k
```

Check grafeo-memory's `search.py` for the exact hybrid query syntax — it
combines vector scores with topology boost (graph connectivity scoring)
in a single pass. The `MemoryConfig` fields `enable_topology_boost` and
`topology_boost_factor` control this behavior.

### Step 2: MCP tool wiring

Return `Vec<SearchResult>` with `memory_id`, `text`, `score`, `created_at`.

**Done when:** You can capture a few memories via Claude Desktop, then search for them semantically and get ranked results back. This is the minimum viable experimentation loop.

---

## Phase 3 — Entity Extraction + Graph Edges (Days 6–8)

**Goal:** Memories are connected by entity co-occurrence edges, enabling graph-enriched search.

### Option A: Local NER (gline-rs / GLiNER)

Keep the original Deep Brain plan. GLiNER extracts entities locally via ONNX. No LLM needed. Faster, fully offline, but less flexible.

### Option B: LLM-based extraction (grafeo-memory style)

Port grafeo-memory's extractor prompt. Send the captured text to a local LLM
(or Claude via MCP client) and get back structured JSON with entities and relationships.
More flexible, handles implicit relationships, but requires an LLM.

### Recommendation for MVP: Option A

GLiNER keeps the air-gap guarantee and is already in the Deep Brain dependency plan.
LLM extraction can be added later as an alternative extractor behind the same trait.

### Graph mutations on capture

After NER, for each extracted entity — use the confirmed grafeo-memory edge types:

```gql
-- Create entity node (MERGE = create if not exists)
MERGE (e:Entity {name: $entity_name, entity_type: $entity_type})

-- Link memory to entity (grafeo-memory uses :HAS_ENTITY)
MATCH (m:Memory {id: $memory_id}), (e:Entity {name: $entity_name})
MERGE (m)-[:HAS_ENTITY]->(e)
```

For entity relationships (grafeo-memory uses `:RELATION` with a `type` property):

```gql
-- e.g. "John works at Acme" → RELATION {type: "works_at"}
MATCH (e1:Entity {name: $name1}), (e2:Entity {name: $name2})
MERGE (e1)-[:RELATION {type: $relation_type}]->(e2)
```

This is richer than the original Deep Brain `CO_OCCURS` edge — grafeo-memory
extracts *named* relationships via LLM, not just co-occurrence. For the MVP
with GLiNER (no LLM), use co-occurrence first:

```gql
MATCH (e1:Entity {name: $name1}), (e2:Entity {name: $name2})
MERGE (e1)-[:RELATION {type: 'co_occurs'}]->(e2)
```

### Graph-enriched search (use hybrid_search)

After vector search returns top-K, traverse one hop from each result:

```gql
MATCH (m:Memory {id: $id})-[:MENTIONS]->(e:Entity)<-[:MENTIONS]-(related:Memory)
WHERE related.id <> m.id
RETURN related.id, related.content, COUNT(e) as shared_entities
ORDER BY shared_entities DESC
LIMIT 5
```

Merge vector scores with graph connectivity for final ranking.

**Done when:** Capturing "I met John at the DeepCausality meeting" and "John presented the causaloid paper" creates entity nodes for John and DeepCausality, and searching for "causaloid" returns both memories with graph boost.

---

## Phase 4 — Session Grouping + Browse + History (Days 9–10)

**Goal:** Memories captured in the same work session are grouped and retrievable together. Change history is tracked.

### Additional MCP tools

| Tool | Description |
|------|-------------|
| `recall_session` | All memories from a `session_id`, chronologically |
| `browse_recent` | Latest N memories, optional filters (mem_type, date) |
| `memory_stats` | Count by type, entity count, total memories |
| `memory_history` | Change history for a specific memory |

### Session threading

`capture_memory` accepts an optional `session_id`. If provided, all captures in that session are linked:

```gql
MATCH (m:Memory {id: $memory_id})
MERGE (s:Session {id: $session_id})
MERGE (m)-[:CAPTURED_IN]->(s)
```

### Change history (from grafeo-memory pattern)

grafeo-memory tracks changes via `:HAS_HISTORY` edges to `:History` nodes.
This is the foundation for Deep Brain's validity windows. In the MVP,
track updates simply:

```gql
-- On memory update, create history node
INSERT (h:History {
    event: 'UPDATE',
    old_text: $old_text,
    new_text: $new_text,
    timestamp: $now,
    actor_id: $user_id
})
WITH h
MATCH (m:Memory {id: $memory_id})
INSERT (m)-[:HAS_HISTORY]->(h)
```

This gives you `memory_history` for free and becomes the substrate for
`valid_from`/`valid_until` in Extension 1.

**Done when:** You can capture a sequence of related thoughts, then recall the full session arc.

---

## Phase 5 — Classification + Memory Types (Days 11–12)

**Goal:** Memories are auto-classified by type, enabling filtered queries.

### Rule-based classifier (from original Deep Brain plan)

Regex over surface patterns:

| Pattern | mem_type |
|---------|----------|
| "I decided", "decision:" | decision |
| "hypothesis:", "I think that" | hypothesis |
| "experiment showed", "result:" | finding |
| "need to", "todo:", "action:" | action_item |
| "defined as", "definition:" | definition |
| none matched | note |

Confidence threshold — below threshold, tag as `unclassified`.

### mem_type filter on search

```gql
MATCH (m:Memory)
CALL vector.search(m.embedding, $query_embedding, $k)
YIELD score
WHERE m.user_id = $user_id AND m.mem_type = $filter_type
RETURN m.id, m.content, m.mem_type, score
```

**Done when:** Capturing "I decided to use Grafeo instead of Diesel" auto-classifies as `decision`, and `search_memories` with `mem_type: decision` filters correctly.

---

## What You Now Have (End of Day 12)

A working bare-bones Deep Brain:

- **Single binary**, `cargo build --release`
- **Single `.db` file**, Grafeo persistent storage, no external services
- **MCP server** over stdio, compatible with Claude Desktop / Cursor / any MCP client
- **Local embeddings**, fastembed-rs, fully offline
- **Local NER**, gline-rs, fully offline
- **6 MCP tools**: `capture_memory`, `search_memories`, `recall_session`, `browse_recent`, `memory_stats`, `memory_history`
- **Hybrid search**: vector similarity + entity graph connectivity (Grafeo native `hybrid_search`)
- **Change tracking**: `:HAS_HISTORY` edges with full audit trail (foundation for validity windows)
- **Auto-classification**: rule-based memory typing
- **Air-gap compliant**: zero network calls

### Crate stack at this point

```toml
grafeo       # graph + vector storage, single dep
fastembed    # local embeddings, ONNX
gline-rs     # local NER, ONNX
ort          # shared ONNX runtime
rmcp         # MCP server
serde        # serialization
serde_json   # JSON
ulid         # IDs
tokio        # async runtime
regex        # classifier patterns
```

Compare to original plan: **Diesel, diesel_migrations, rusqlite, sqlite-vec, pgvector, ultragraph** — all gone. Replaced by `grafeo`.

---

## What Comes Next (Not in MVP)

These are the Deep Brain-specific extensions that build on this foundation. Each is an independent workstream:

### Extension 1: Validity Windows + Epistemic States

Add `valid_from`, `valid_until`, `epistemic_state` properties to `:Memory` nodes. Implement state transitions (active → superseded → invalidated). This is the process ontology layer.

### Extension 2: Invalidation System

`:Invalidation` nodes with scope, grounds, assumptions, confidence, revisability. `INVALIDATED_BY` edges. Reconstruction detection on capture (vector similarity against invalidated subgraph).

### Extension 3: Gap Nodes

`:Gap` nodes as first-class citizens. `TERMINATES` edges from causal chains. Gap centrality queries via Grafeo's built-in graph algorithms.

### Extension 4: Causal Edge Taxonomy

Replace generic `CO_OCCURS` with typed causal edges: `CAUSES`, `ENABLES`, `PREVENTS`, `AMPLIFIES`, `TRIGGERS`, `CONTRIBUTES_TO`, `FAILED_DUE_TO`, `RESOLVED_BY`, `REFINES`, `SUPERSEDES`.

### Extension 5: Context Layer

`:Domain`, `:TemporalRegime`, `:Assumption`, `:Provenance`, `:Person` contextoid nodes. The context hypergraph as a label/edge-type namespace within Grafeo.

### Extension 6: LLM-based Reconciliation

Port grafeo-memory's reconciler: before storing a new memory, search for existing related memories, ask the LLM whether this is new (ADD), an update to existing (UPDATE), a contradiction (potential invalidation), or redundant (NONE).

### Extension 7: ADLR Extraction

Local LLM structured extraction from causal reports. Full causal chain write, invalidations, open problems. The highest-complexity feature.

### Extension 8: DeepCausality Bridge

Context projection from Grafeo graph traversal into DeepCausality Context structures. Condition-triggered, not calendar-triggered.

---

## Crate Availability (Confirmed)

All Grafeo Rust crates are published on crates.io. Use standard cargo dependencies:

```toml
grafeo = { version = "...", features = ["gql", "ai"] }
```

No git dependencies needed. Pin versions in `Cargo.lock` as with any production dependency.

---

## Risk Profile

### Low risk: Grafeo foundation

Enterprise-extracted codebase developed by a professional data architect at a
European insurance company. ACID transactions, MVCC, persistent storage, and
graph algorithms have been tested against regulated financial workloads where
correctness is legally mandated. The Rust core has zero required C dependencies.
Apache-2.0 licensed.

### Moderate risk: Deep Brain extensions

The custom epistemic layer (validity windows, invalidation system, gap nodes,
causal edge taxonomy, DeepCausality bridge) is novel application logic built
on top of a proven engine. This is where engineering discipline matters.

**Required practices for extension code:**

1. **Every extension gets its own module** — `src/epistemic/`, `src/invalidation/`,
   `src/gaps/`, `src/causal/`. No mixing extension logic into the base memory pipeline.

2. **Trait boundaries between layers** — the base MemoryManager should not know about
   invalidations. Extensions implement traits that the pipeline calls at defined
   hook points (pre-capture, post-capture, pre-search, post-search).

3. **Property tests for graph invariants** — validity windows must satisfy
   `valid_from < valid_until OR valid_until IS NULL`. Invalidation chains must be
   acyclic. Gap nodes must have at least one inbound `TERMINATES` edge. Use
   `proptest` or `quickcheck` to fuzz these invariants against random graph mutations.

4. **Integration tests against a real Grafeo instance** — not mocks. Grafeo is
   embeddable and in-memory mode starts in milliseconds. Every test gets a fresh
   `GrafeoDB::new_in_memory()`. Test the actual GQL queries you'll run in production.

5. **Typed edge taxonomy from day one** — don't start with string-typed edge names
   and plan to "clean them up later." Define a Rust enum for causal edge types
   (`Causes`, `Enables`, `Prevents`, etc.) and serialize to/from GQL edge types
   at the boundary. The type system should prevent `CAUSES` being misspelled as
   `CAUSE` anywhere in the codebase.

6. **Reconstruction detection gets its own test corpus** — build a small set of
   known invalidated hypotheses with known assumption sets. CI must verify that
   capturing a semantically similar memory triggers the reconstruction warning
   at the correct confidence threshold. This is the highest-value feature and
   the easiest to get subtly wrong.

7. **No silent failures on graph mutations** — every GQL mutation returns a result.
   Check it. If an edge creation fails because a node doesn't exist, that's a bug
   in the capture pipeline, not a transient error to swallow.

---

## Decision Log

| Decision | Rationale |
|----------|-----------|
| Grafeo over Diesel + sqlite-vec + UltraGraph | Collapses three layers into one. Same Rust, same air-gap, radically simpler stack. |
| fastembed-rs over Grafeo's built-in `embed` | Evaluate during Phase 0. If Grafeo's embedder supports nomic-embed, switch. |
| gline-rs over LLM extraction in MVP | Air-gap compliance, no LLM dependency for basic capture, lower latency. |
| No reconciliation in MVP | Get the storage + search loop working first. Reconciliation is Extension 6. |
| No validity windows in MVP | Epistemic model is the differentiator, not the plumbing. Build it on a working base. |
| GQL as query language | ISO standard, most expressive for Grafeo, pattern matching syntax matches Deep Brain's traversal needs. |
