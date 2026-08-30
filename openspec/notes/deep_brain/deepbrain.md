# Deep Brain — Architecture

**Knowledge propagation on a single graph store**

Status: current. Supersedes the storage and hypergraph mechanics of *Technical Architecture Rev 3*
and *Next Generation* Part IV. The *Philosophical Foundation* stands, with one addition to §4.2
carried here. *deep_brain_mvp_plan.md* is superseded in full.

Author: Marvin Hansen / Emet-Labs, with Claude. August 2026.

---

## Result

Deep Brain propagates knowledge the way the EPP propagates effects. That thesis is unchanged. Three
things changed since March.

**The backend collapsed to one component.** Diesel, rusqlite, sqlite-vec, pgvector, and three
UltraGraph instances become a single embedded graph database with native vector search. The
relational store and the hypergraphs stop being separate things that must be kept in sync.

**A fourth extraction method appeared.** Declared provenance: a relation the researcher asserted at
authoring time, inside the artifact, as part of the practice that produced it. A docstring citing a
publication and section. A `\cite` key. An openspec requirement id. A THEOREM_MAP entry binding a Lean
theorem to its Rust witness. It carries the epistemic status of manual linking and needs no model to
extract.

**The audit chain turned out to instantiate the whole ontology.** Tracing an idea from its origin in a
publication, through its specification, through its proof, to its implementation and every call site,
exercises every primitive the KPP defines: memory, causal edge, invalidation, assumption node, gap
node, all six contextoid types, and all six relational edge types. Every one of them, populated from
declared data, with no inference step.

That last point reorganizes the build. The audit chain is a complete vertical slice through the
ontology rather than a subset of it. Building it first gives the epistemic layer a populated,
ground-truth graph to develop against, and it ships with no embedding model, no NER, no LLM, and no
network.

The context hypergraph as a separate in-memory structure moves to Stage 3. In Stage 1 contextoids and
content live in one labeled graph and traversal happens in the database. Projecting a slice of that
graph into an in-memory structure is the bridge problem, and it is not the first problem.

---

# Part I — What changed

## 1.1 The measurement

In August 2026 this repository was indexed with XERJ, a search engine that infers structure from a
folder. The run produced 509,765 documents.

| Field | Docs | Share |
|---|---|---|
| `text`, the only embedded field | 5,803 | 1.1% |
| `body`, BM25 only | 17,940 | 3.5% |
| `code`, BM25 only | 475,013 | 93.2% |

Semantic search reached 1.1% of the corpus. Embedding was inferred for long body text, and code
entered at symbol granularity with a mean field length of 44 characters. A query for *"factoring a
matrix into orthonormal and upper-triangular parts"* returned `tokio-1.53.1/README.md` in the top
three, with all scores inside a 0.4% band. The BM25 control found the right files in 97ms against
3202ms.

Two artifact classes had been indexed into two record shapes with no shared join key. Prose became
passages; code became fragments; nothing spanned them. Similarity across that boundary is guesswork,
because a paper says *reciprocal rank fusion*, a spec says *hybrid ranking*, and the code says
`fuse_rrf`.

The lesson generalizes past that one tool. **Any design that stores what the researcher read and what
the researcher built in two representations loses the join between them.** The join is the product.

## 1.2 The corpus this has to serve

Publications live in 13 per-crate `papers/` folders holding 49 PDFs, with LaTeX sources for the
papers authored here. Concept documents live in `openspec/notes/`. Requirements live in
`openspec/changes/`. Proofs live in `lean/`, indexed by `lean/THEOREM_MAP.md`. Implementations live
across 27 crates.

The per-crate paper layout is itself a declared signal. A publication filed under
`deep_causality_physics/papers/` and cited from a kernel in another crate has been placed in two
domains by hand.

## 1.3 The audit chain

```
Origin              a publication section, or a concept document
   │ GroundedIn
Specification       an openspec requirement id
   │ Refines
Proof               a Lean theorem, bound by THEOREM_MAP
   │ witnesses
Implementation      a function, with the citation in its docstring
   │ Usage
Call sites          every consumer across every crate
```

Every link in that chain already exists in the repository as a human-written assertion. None of it
has been extracted.

---

# Part II — The ontology

The *Philosophical Foundation* survives the engine change intact, because it never mentioned an
engine. This section restates what the architecture must carry, and adds one entry.

## 2.1 What exists

**Reasoning primitives**, recursive and compositional, the causaloid layer:

- **Memory**, the unit of epistemic satisfaction. Isomorphically recursive: a memory, a session of
  memories, and a domain of sessions share one form.
- **Causal edge**, the unit of epistemic relation. Carries extraction method, confidence, validity
  window, and a nullable `causaloid_id` reserved for the bridge.

**Ground truth primitives**, non-recursive by construction, the contextoid layer:

- **Invalidation**, the unit of epistemic exclusion. Cannot contain another invalidation, which
  prevents a chain of mutual invalidations.
- **Assumption node**, the shared ground on which multiple exclusions rest. Dissolving one surfaces
  every dependent invalidation.
- **Gap node**, a characterized absence. Not a missing memory; a positive fact that a question was
  investigated and remains open, with a documented causal structure.

**The source document is the ground.** Every primitive links back to it, because the source is the
practice made persistent. When the graph is insufficient, the system returns to the source. Stage 1
sharpens this: every declared edge carries a byte range, so the return path is a dereference.

**The PropagatingKnowledgeEffect** is the unit that moves through the graph. It is multi-modal:
epistemic update, epistemic exclusion, epistemic gap, epistemic context link. Repository harvest emits
three of the four modes, all declared.

## 2.2 The four extraction methods

Addition to *Philosophical Foundation* §4.2, which currently names three.

> **Declared Provenance (Epistemologically: Deferred Manual Linking).** The researcher asserted the
> relation at authoring time, inside the artifact, as part of the practice that produced it.
> Epistemic status: identical to Manual Linking, because a human directly asserted the relation. It
> requires no review step for the same reason manual linking does not; the assertion is itself the
> review.

The distinguishing property is its relationship to §4.5, which holds that a claim keeps its meaning
only insofar as the source document stays linked to it. A declared relation satisfies that by
construction. The assertion and its ground are co-located.

| Method | Review | Confidence | Source |
|---|---|---|---|
| Manual | none | Observed | researcher asserts now |
| **Declared** | **none** | **Observed** | **researcher asserted at authoring time** |
| ADLR | none | High | completed reasoning, LLM formalizes |
| NLP | required | Low | linguistic signal, candidate only |

Two rules follow, and both are load-bearing.

**Declared and Manual edges are never subject to LLM reconciliation.** A model does not get a vote on
whether a docstring cites Golub and Van Loan. Reconciliation operates on NLP-tier material.

**Extraction method is a ranking input.** Every edge carries it as a property, and every ranked result
reads it. This is the anti-hallucination primitive: an agent separates *the docstring says this
implements §5.2* from *these two spans embed 0.71 similar*.

## 2.3 Worked example

`deep_causality_unified_math/deep_causality_tensor/src/types/causal_tensor/ops/tensor_qr/mod.rs:31`:

```rust
/// Thin Householder QR: `A = Q · R` with `Q` (`m × k`) orthonormal columns and `R` (`k × n`)
/// upper-triangular, where `k = min(m, n)`.
///
/// # Reference
/// G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ. Press,
/// 2013), §5.2 (Householder QR factorization).
pub fn qr(&self) -> Result<(Self, Self), CausalTensorError> {
```

A harvester reads that block and writes one `GroundedIn` edge from the code span to a
`ProvenanceContextoid` for Golub and Van Loan §5.2, extraction method Declared, locator
`mod.rs:24-27`. No model participates.

---

# Part III — One graph

## 3.1 The store

`grafeo`, an embedded graph database with native vector search.

| | |
|---|---|
| Version | 0.5.42 on crates.io, Apache-2.0 |
| Edition, MSRV | 2024, 1.91.1, against this workspace's 1.98.0 pin |
| Dependencies | 6, of which 4 are its own crates and 2 are optional allocators; no required C dependencies |
| Models | Labeled property graph and RDF triples, typed properties on both |
| Vectors | HNSW, tunable recall; cosine, euclidean, dot product, manhattan; SIMD via AVX2, SSE, NEON |
| Text | BM25 index alongside the vector index |
| Hybrid | Vector, text, and graph traversal combined in one query, RRF or weighted fusion |
| Languages | GQL, Cypher, Gremlin, GraphQL, SPARQL, SQL/PGQ, with EXPLAIN ANALYZE |
| Storage | In-memory and persistent; columnar with dictionary, delta, and RLE compression; WAL |
| History | Bitemporal epochs with time travel |

Unsafe code inside a dependency is out of scope for the workspace lint, which governs code written
here.

## 3.2 What the three hypergraphs become

March specified a relational store as the source of truth with three UltraGraph instances rebuilt
from it on startup, held in a coordinated freeze cycle that "temporarily doubles memory usage".

| March | August |
|---|---|
| `memories` table plus content hypergraph | `:Span` nodes |
| `contextoids` table plus context hypergraph | `:Contextoid` nodes, same graph |
| `content_context_relations` plus relational hypergraph | typed edges carrying properties |
| Rebuild all three on startup | nothing to rebuild |
| Coordinated freeze and unfreeze | no freeze |
| sqlite-vec or pgvector | native HNSW on the same nodes |
| Diesel migrations | grafeo schema plus property indexes |
| Five hand-written indexes on the join table | property indexes plus native traversal |

The three structures remain three structures conceptually. They stop being three artifacts to
populate, synchronize, and freeze together. `create_projection` materializes each as a view when a
view is wanted.

Open Question 4 from Rev 3, on three-hypergraph startup performance and serialized checkpoints,
dissolves. Open Question 5, contextoid deduplication, gets its mechanism from a property index on
`(label, contextoid_type)` with MERGE; the policy is still to be written. Open Question 9, relational
hypergraph inference, becomes a transitive path query, so the materialized `inferred` flag turns into
an optimization rather than a requirement.

## 3.3 Node labels

```
:Span            unit of epistemic satisfaction; the March :Memory generalized
                 to cover harvested artifacts
:Contextoid      six types, unchanged from Rev 3 §3.2
:Invalidation    qualified exclusion, non-recursive
:Gap             characterized absence, non-recursive
:Report          ADLR or equivalent structured document
:Session         a knowledge occasion
```

`:Span` carries both classification axes on one node, so one vector index covers the corpus:

```
provenance         declared | captured
extraction_method  Declared | Manual | ADLR | NLP
artifact_type      paper | spec | proof | code | usage | note
mem_type           hypothesis | experiment | finding | decision | note |
                   definition | action_item | reference | idea
epistemic_state    active | superseded | invalidated
text               natural language, always present
embedding          Value::Vector, 384 dimensions
locator            path plus byte range, always present
blob_sha           git object id for harvested spans
valid_from, valid_until
```

The `text` field is present on every span, code included. For code it holds the documentation
surface: doc comment, signature, canonical path. Embedding the documentation surface rather than the
token stream is the decision that closes the 1.1% split from §1.1.

## 3.4 Edge types

Content relations, unchanged from Rev 3 §4.1:

```
Causes  Enables  Prevents  Amplifies  Triggers  ContributesTo
FailedDueTo  ResolvedBy  Refines  Supersedes
InvalidatedBy  ChallengedBy  SupersededBy  Terminates
```

Content-to-context relations, unchanged from Rev 3 §3.3, now edges rather than join-table rows:

```
ValidIn  BelongsTo  GroundedIn  AssumesC  AuthoredIn  CrossDomain
```

Context-internal relations, unchanged from Rev 3 §3.2:

```
ParentDomain  ChildDomain  Precedes  Supersedes  Dissolves  Cites  CitedBy  ColleagueOf
```

One addition, harvested from the rustdoc item graph:

```
Usage            a span calls, implements, or bounds another span
```

Every edge carries the epistemic properties that the March schema kept in columns:

```
extraction_method  confidence  extraction_confidence
valid_from  valid_until  inferred  locator  causaloid_id
```

Putting these on the edge rather than on a reified node is a Stage 1 simplification. The relational
hypergraph's *nodes are connections* framing from *Philosophical Foundation* §3.7 is recoverable
through RDF reification or an explicit `:Relation` node if edges about edges are ever needed. None of
the algorithms in Rev 3 §4.3 require it.

## 3.5 Indexes

```rust
db.create_vector_index("Span", "embedding", Some(384), Some("cosine"), None, None, None)?;
db.create_text_index("Span", "text")?;

for prop in ["provenance", "extraction_method", "artifact_type", "mem_type",
             "epistemic_state", "blob_sha", "locator"] {
    db.create_property_index("Span", prop)?;
}
db.create_property_index("Contextoid", "label")?;
db.create_property_index("Contextoid", "contextoid_type")?;
```

Vector, text, and property indexes sit on the same nodes in the same store. That is the structural
reason the split brain cannot come back.

## 3.6 Verified API

Read directly from `examples/rust/vector_search.rs` and the crate documentation:

```rust
let db = GrafeoDB::open("./deep_brain.grafeo")?;

let span = db.create_node(&["Span", "Code"]);
db.set_node_property(span, "text", Value::from(doc_surface));
db.set_node_property(span, "locator", Value::from("…/tensor_qr/mod.rs:24-31"));
db.set_node_property(span, "extraction_method", Value::from("Declared"));
db.set_node_property(span, "embedding", Value::Vector(vec.into()));

let hits = db.vector_search("Span", "embedding", &query_vec, k, None, /* filters */ None)?;
```

`Value::Vector` wraps `Arc<[f32]>`. The `filters` argument applies property predicates at ANN time,
which is where extraction-method and artifact-type constraints belong.

Present as method names on `GrafeoDB` and not yet read in detail: `hybrid_search`, `mmr_search`,
`create_edge_with_props`, `batch_create_nodes_with_props`, `batch_vector_search`,
`execute_with_params`, `create_projection`, `session_with_cdc`, `execute_at_epoch`,
`get_node_history`, `get_edge_history`, `changes_between`, `history_since`. Confirm signatures before
writing against them.

`mmr_search` earns attention. Maximal marginal relevance trades a little relevance for diversity,
which is what chain assembly wants. The XERJ probe in §1.1 spent two of five result slots on one file.

## 3.7 Epochs

Grafeo carries a bitemporal history. Deep Brain stamps **one epoch per indexed commit** for harvested
content and one per session for captured content.

This gives the validity windows of Rev 3 §3.1 a second implementation path. `valid_from` and
`valid_until` stay on nodes and edges as the epistemic validity window, which is a claim about the
world. Epochs record when the graph itself changed, which is a claim about the record. The two are
different questions, and the March schema could only answer the first.

---

# Part IV — The audit chain

## 4.1 Why this is the first thing to build

A knowledge occasion in the KPP carries content, grounds, conditions, assumptions, authorship, and
relations. The audit chain instantiates every one of them from declared data.

| KPP primitive | Instantiated by | Declared source |
|---|---|---|
| Memory | code span, spec span, paper section, proof span | the artifact |
| Causal edge | Refines, Supersedes, ResolvedBy, Usage | openspec, `reverted/`, rustdoc |
| `GroundedIn` → Provenance | cited publication and section | docstring, `\cite` |
| `BelongsTo` → Domain | the crate | path |
| `AssumesC` → Assumption | Lean theorem hypotheses | THEOREM_MAP |
| `ValidIn` → Regime | commit SHA, toolchain, feature flags | git |
| `AuthoredIn` → Person | author | git blame |
| `CrossDomain` | one publication cited from two crates | docstrings |
| Gap node | a chain missing a link | traversal |
| Invalidation | drift between chain links | epoch comparison |

Nothing on that list requires a model. The chain is a complete vertical slice through the ontology,
not a subset of it. That is the argument for building it first: the epistemic layer then develops
against a populated graph whose every edge points at a byte range someone can open and read.

## 4.2 `chain()`

```
Concept: Householder QR                        anchor: THEOREM_MAP/QRFactorization
  origin  golub_van_loan_2013 §5.2                      Declared    docstring citation
  spec    openspec/…/tensor-qr/spec.md #req-3           Declared    requirement id
  proof   lean/…/QR.lean : QRFactorization              Declared    theorem map
  impl    …/ops/tensor_qr/mod.rs:31 fn qr               Declared    docstring
  impl    deep_causality_linear::qr                     Structural  rustdoc call edge
  usage   47 call sites across 6 crates                 Structural  rustdoc
```

A concept is nucleated by an explicit anchor: a THEOREM_MAP theorem, an openspec requirement id, a
bibkey, or a Lean namespace. Spans join it by declaring an edge to that anchor. Embedding similarity
then *suggests* further members, surfaced as NLP-tier candidates for review.

Nucleating on anchors rather than clustering keeps the concept boundary human-authored, which is what
*Philosophical Foundation* §3.5 requires when it holds that the genesis process is never modified by
the system.

## 4.3 `gaps()`

Rev 3 made gap nodes first-class and populated them from ADLR extraction. The chain populates them
from traversal:

- a concept with an origin and a spec and no implementation
- an implementation with no spec
- a spec with no Lean witness
- a PDF in a `papers/` folder that no kernel cites
- two active domain contextoids with no `CrossDomain` edge between them, which Rev 3 §3.3 already
  names a relational gap

The `termination_reason` field from Rev 3 §3.1 applies unchanged. A structural gap terminates for a
different reason than a research gap, and the field already distinguishes them.

## 4.4 `drift()`

```
concept "Householder QR"
  spec    last changed  epoch 087
  proof   last changed  epoch 087
  impl    last changed  epoch 412
  → implementation moved 325 epochs after its spec and witness
    blast radius = transitive closure over Usage edges
```

A drift result is a candidate invalidation with a computable blast radius. It arrives with grounds
(the epoch comparison), a regime contextoid (the commit), and a target, which is everything Rev 3
§3.1 requires of an invalidation record apart from researcher confirmation.

This grounds reconstruction detection on observation. The March risk profile names that feature the
highest-value and the easiest to get subtly wrong; declared chains give it a ground-truth corpus.

## 4.5 The Completion Invariant

From *knowledge-propagation-process.md* §6:

> A change is not mergeable until the graph is consistent. Like "all tests pass" or "no compiler
> warnings", graph consistency becomes a build gate.

`gaps()` and `drift()` are that gate. A degenerate form already runs here: the formalization workflow
greps a hardcoded crate allowlist for missing Rust witnesses, and a new witness crate fails CI until
someone edits the allowlist by hand. Generalized over the graph, the allowlist stops existing.

The same note records the failure that motivates it. A parameter rename broke four consumer apps; the
consumer set was recoverable and unlinked. `Usage` edges from the rustdoc item graph make that query
answerable before the change is declared complete.

---

# Part V — Ingest

Six pipelines. The March ordering put conversation capture first and had no pipeline for the
repository. Harvest moves to first position because it needs no models and produces ground truth.

## Pipeline 1 — Repository harvest

```
Git commit (git ls-tree -r <sha> + git cat-file; never the working tree)
    │
    ├── rustdoc --output-format json
    │       ├── doc comment + signature + canonical path → span text
    │       ├── item graph → Usage edges (calls, impls, trait bounds)
    │       └── citation blocks in docs → GroundedIn (Declared)
    │
    ├── LaTeX (*/papers/**/*.tex, papers authored here)
    │       ├── \label, \ref → intra-paper structure
    │       ├── \cite → GroundedIn to ProvenanceContextoid (Declared)
    │       └── theorem, lemma environments → span boundaries
    │
    ├── PDF section extraction (*/papers/**/*.pdf, third-party)
    │       └── section tree → one span per section, keyed by bibkey
    │
    ├── openspec/changes/**/spec.md
    │       └── requirement ids → Spec spans; Refines, ResolvedBy edges
    │
    ├── lean/** + lean/THEOREM_MAP.md
    │       ├── theorem → witness binding → proof edge (Declared)
    │       └── theorem hypotheses → AssumptionContextoid + AssumesC
    │
    ├── BUILD.bazel + Cargo.toml → crate dependency edges
    ├── git blame → PersonContextoid + AuthoredIn
    │
    └── write: spans, contextoids (MERGE on label + type), edges,
               epoch stamp = commit SHA
```

Four properties separate this from the pipelines that follow.

**Input is immutable.** Reading the git object store at a fixed SHA removes the class of failure where
a file changes mid-run. The XERJ run aborted after 32 minutes for exactly this reason, while a crate
was being moved to `reverted/`.

**Reconciliation is exact.** `git diff --name-status <last_sha> HEAD` yields adds, modifications,
deletions, and renames. Spans key on blob SHA, so a rename costs no re-embedding and moving a crate
into `reverted/` is free.

**Every span carries `text`.** Code included, holding the documentation surface.

**No model is required.** Embedding is needed only when semantic retrieval is switched on. The graph,
`chain()`, `gaps()`, `drift()`, and the CI gate all work without one.

## Pipeline 2 — General markdown notes

Unchanged from Rev 3 Pipeline 1. Embedding, NER over both tier label sets, rule-based classification,
causal-marker extraction at NLP confidence, reconstruction detection before write.

## Pipeline 3 — ADLR reports

Unchanged from Rev 3 Pipeline 2. Local LLM structured extraction produces the causal chain,
invalidations with their assumption and regime contextoids, open problems, and corrective actions.

## Pipeline 4 — arXiv import

Unchanged from Rev 3 Pipeline 3. Three stages, human-gated at every one, the single controlled
network exception, outbound only.

## Pipeline 5 — Manual causal linking

Unchanged from Rev 3 Pipeline 4. Epistemologically primary; no review step, because it is the review.

## Pipeline 6 — Gap resolution

Unchanged from Rev 3 Pipeline 5. The gap node is never deleted; it becomes epistemic history.

## Model requirements by pipeline

| Pipeline | Method | Models |
|---|---|---|
| 1 Repository harvest | Declared | none |
| 2 Markdown notes | NLP | fastembed, GLiNER |
| 3 ADLR reports | ADLR | local LLM |
| 4 arXiv import | human-gated | fastembed, GLiNER, PDF extraction |
| 5 Manual linking | Manual | none |
| 6 Gap resolution | Manual | none |

---

# Part VI — Retrieval and reasoning

## 6.1 Search

One query against one store. `hybrid_search` combines BM25 over `text`, HNSW over `embedding`, and
graph traversal, fused by RRF. Weight the lexical leg above the vector leg for this corpus; exact
identifiers such as `CausalTensor` or a requirement id are what BM25 handles and embedders do not.

Filters at ANN time carry the epistemic constraints: `extraction_method`, `artifact_type`,
`epistemic_state`, domain contextoid.

Results return the chain rather than the span. A hit maps to its concept anchor, the anchor expands
along Declared and Structural edges, and the caller receives origin, spec, proof, implementation, and
usage as one object with a locator and an extraction method on every element.

## 6.2 Reconstruction detection

Unchanged from Rev 3 §7, with one simplification. The confidence-scaled thresholds stand:

| Confidence | Revisability | Warning threshold |
|---|---|---|
| Hard | Settled | 70% |
| Hard | Contested | 80% |
| Soft | Open | 90% |
| Provisional | Open | 95% |

*Philosophical Foundation* §4.4 holds that these express how evidential burden scales with the
confidence of an exclusion, so they are policy rather than tuning knobs.

The simplification: retrieving the assumption and regime contextoids for a matched invalidation was a
join across `content_context_relations`. It becomes a two-hop traversal from the invalidation node.

## 6.3 Blast radius

Rev 3 §3.3 implemented reverse dependency as a query over the join table with a dedicated index.
Given a contextoid, return all content connected by `AssumesC` or `ValidIn`. That becomes a native
inbound traversal.

Assumption dissolution follows the same path. A `Dissolves` edge into an assumption contextoid, then
inbound `AssumesC` traversal, surfaces every dependent invalidation. This is the computational form of
the Duhem-Quine constraint from §4.3, and it is one traversal.

## 6.4 Cross-domain resonance

*Philosophical Foundation* §4.6 makes structural resonance across domain boundaries the primary
epistemic signal, and names HAFT across quantum mechanics, general relativity, and plasma dynamics as
the case.

`cross_domain_gaps` currently ranks on embedding similarity, which is NLP-tier and candidate-only.
Harvest gives it a primary-tier channel: one publication cited from two crates is a declared
`CrossDomain` relation. Those citations sit in the kernel docstrings today, under the convention that
a kernel names its source publication in full with the PDF in that crate's `papers/` folder.

---

# Part VII — MCP surface

Seventeen tools from Rev 3 §9 stand. Four are added for the chain, and their argument and result
shapes are the Stage 1 deliverable.

| Tool | Description |
|---|---|
| `chain` | Given a concept anchor or a free-text query, return origin, spec, proof, implementation, and usage, each with locator and extraction method |
| `gaps` | Concepts whose chain is missing a link; includes the relational gaps of Rev 3 §3.3 |
| `drift` | Chains whose links last changed at different epochs, with blast radius over `Usage` |
| `harvest` | Run Pipeline 1 at a commit; incremental against the last indexed SHA |

`query_gaps` and `query_relational_gaps` from Rev 3 remain the research-facing entry points. `gaps` is
the structural counterpart, and both write into the same `:Gap` nodes.

---

# Part VIII — Stages

## Stage 1 — One graph, harvest, chain, gate

Grafeo embedded. Schema, labels, edge types, indexes. Pipeline 1 across all harvesters. `chain`,
`gaps`, `drift`, `harvest` over MCP. CI gate on the Completion Invariant.

No embedding model, no NER, no LLM, no network. Verifiable by opening the file a locator points at.

Exit: `chain()` returns the Householder QR chain from real harvested data, and `drift()` flags a known
stale link.

## Stage 2 — Capture and the epistemic layer

Pipelines 2 through 6. fastembed and GLiNER. Invalidations, assumption contextoids, reconstruction
detection, challenge and dissolution. arXiv subsystem. The remaining Rev 3 MCP tools.

Captured content enters the same graph as harvested content, separated by the `provenance` property.
Reconciliation applies to NLP-tier material only.

Exit: reconstruction detection fires correctly on known invalidated hypotheses in CI.

## Stage 3 — In-memory projection

The context hypergraph as a structure rather than a label. Project a slice of the store into an
in-memory graph for algorithms that want it: gap centrality, dissolution chains, domain hierarchy
traversal, cross-domain bridge enumeration.

This is deferred deliberately. Mapping database contents into an in-memory structure is tractable and
it is not the first problem. Grafeo's own algorithms cover PageRank, shortest path, Louvain, and
centrality in the meantime, and `create_projection` provides materialized views without a second
representation to keep in sync.

Exit: a measured case where in-memory traversal beats in-database traversal by enough to justify the
synchronization cost.

## Stage 4 — The DeepCausality bridge

Rev 3 §10 stands. The bridge condition remains a ratio above 0.3 with more than 20 high-confidence
edges per domain contextoid.

Harvest changes when this becomes measurable. Declared edges are high-confidence by construction and
arrive in volume, so the condition can be evaluated against real data rather than projected. The
projection mapping is unchanged: temporal contextoids to Tempoids, domain contextoids to Symboids,
regime and assumption contextoids to Datoids.

## Deferred

Zone architecture from *Next Generation*. Zone IDs cost nothing while empty, so carry the property
from Stage 1 and leave the migration protocol unbuilt. The atomic zone lock and cross-zone migration
rules address multi-domain enterprise scale; the single-repository case has a degenerate answer where
crate is zone and commit is version.

*Next Generation* Part IV needs rewriting against projections and epochs before any of it is built.

---

# Part IX — Crate stack

```toml
# ── Store ──────────────────────────────────────────────────────────
grafeo      = { version = "0.5.42", features = ["gql", "ai", "hybrid-search"] }

# ── Harvest (Stage 1) ──────────────────────────────────────────────
rustdoc-types  # rustdoc JSON deserialization — verify current format version
gix            # pure-Rust git object access — candidate, verify API surface
serde, serde_json, ulid

# ── MCP ────────────────────────────────────────────────────────────
rmcp = { features = ["server", "transport-io"] }
tokio

# ── Stage 2 ────────────────────────────────────────────────────────
fastembed, gline-rs, ort   # evaluate grafeo's `embed` feature first
reqwest, quick-xml         # arXiv, outbound only
regex                      # classifier
```

Removed from the March stack: Diesel, diesel_migrations, rusqlite, sqlite-vec, pgvector, ultragraph.

Air-gap posture is unchanged. Stage 1 makes no network calls at all, including model downloads.

---

# Part X — Success criteria

**Stage 1**

- `harvest` completes over this repository from a clean state without aborting on a working-tree race
- `chain()` returns the Householder QR chain with correct locators for all five links
- Every returned locator dereferences to the asserted text
- `gaps()` reproduces the known missing Lean witnesses that the current CI grep reports
- `drift()` flags a link whose implementation epoch exceeds its spec epoch
- Incremental harvest after a rename re-embeds nothing
- Zero network calls confirmed in CI

**Stage 2**

- Capture-to-searchable latency at or below 3 seconds
- ADLR extraction produces valid JSON on 90% or more of test reports
- Reconstruction detection fires correctly on known invalidated hypotheses in CI
- Semantic search returns results in 500ms or less over a 10,000-span corpus
- Blast radius returns complete affected content for known contextoid changes

**Stage 4**

- Bridge condition observable and stable across 30 days
- At least one domain contextoid crosses the threshold
- A manually authored `BaseCausaloid` evaluates correctly against new observations

---

# Open questions

**1. Span granularity for code.** One span per item, per impl block, or per module. Item level matches
rustdoc output and gives the finest locator. Measure the node count on this repository before
committing.

**2. Rustdoc JSON stability.** The format is nightly-only and versioned. This workspace pins
`rust-version = "1.97.1"`, so a fixed toolchain exists; confirm the JSON format version is stable
across that pin and pin `rustdoc-types` to match.

**3. Third-party PDF fidelity.** Papers authored here index from LaTeX with exact granularity. The 49
PDFs across the 13 `papers/` folders need a section extractor. Evaluate GROBID against them before
assuming section-level citation targets are reachable.

**4. Citation format in docstrings.** The convention is a full reference under a `# Reference`
heading. Harvesting needs a stable key to MERGE provenance contextoids on. Decide whether to add a
bibkey to the convention or to parse the reference text.

**5. Contextoid deduplication policy.** The mechanism is a property index with MERGE. The policy for
near-matches, such as two spellings of one author, is open. Rev 3 Open Question 5 remains.

**6. Grafeo release cadence.** The crates.io release is dated three months before the repository head.
Pin exactly and decide whether to vendor.

**7. Embedding backend.** Grafeo ships an `embed` feature. Evaluate it against fastembed before adding
`ort` to the tree, and note that Stage 1 needs neither.

---

# Supersession

| Document | Section | Status |
|---|---|---|
| Philosophical Foundation | all | stands; §4.2 gains Declared Provenance per Part II above |
| knowledge-propagation-process | all | stands; §6 Completion Invariant realized in Part IV |
| Technical Architecture | §2 Storage and ORM | superseded by Part III |
| Technical Architecture | §3 Database Schema | re-expressed as labels, edges, and properties in Part III |
| Technical Architecture | §4 Three Hypergraphs | mechanics superseded; taxonomy retained |
| Technical Architecture | §5 Pipelines | retained, renumbered; harvest inserted first |
| Technical Architecture | §7–§10 | retained |
| Technical Architecture | §11 Crate Stack | superseded by Part IX |
| Technical Architecture | §14 Roadmap | superseded by Part VIII |
| Technical Architecture | §16 Q4 | dissolved; Q5 and Q9 mechanism resolved, policy open |
| Next Generation | Parts I–III, V–VIII | deferred, not superseded |
| Next Generation | Part IV | rewrite against projections and epochs |
| deep_brain_mvp_plan | all | superseded by Part VIII |

The March documents stay in place as the record of how the design arrived here. *Philosophical
Foundation* is the one that never needed revising, which is the expected result when the metaphysics
is written independently of the machine.
