# Note: A Mixed Graph for `deep_causality_topology`

**Status:** discussion note for review — not a specification.
**Author context:** surfaced while planning the BRCD root-cause-discovery foundations (`brcd-prep-foundations`, Tier B). Investigating where to host a CPDAG showed the repo has no structure that models it. This note records why, what already exists, the relevant mathematics, and a sensible path forward. A specification follows after review.

## 1. Why this came up

The BRCD estimator and, more generally, constraint-based causal discovery (PC, GES, Meek orientation) operate on a **CPDAG** — a graph carrying *both* directed arcs `u → v` and undirected edges `u — v` at the same time, with the invariant that any node pair is in exactly one of {no edge, arc one way, arc the other way, undirected edge}.

The prep change planned to build this CPDAG as modules inside `deep_causality_algorithms`. Before writing it, we checked whether an existing graph structure could express it. None can — and the reason is structural, not incidental.

## 2. The precise problem: a CPDAG is a *superset* edge model

Edge "direction" is an axis orthogonal to the other ways a graph can be general (node payload, edge arity). An undirected edge `{u, v}` asserts symmetry and discards orientation; a directed edge `u → v` carries orientation. Neither is a special case of the other — they are **incomparable**.

A CPDAG needs **three states per node pair simultaneously** (none / directed / undirected). That is strictly more general than any single-edge-kind structure. You cannot faithfully express a superset through a subset:

- through a **directed-only** structure: encoding `u — v` as the pair `u → v` and `v → u` turns every undirected edge into a 2-cycle, which breaks acyclicity and topological sort — the very operations the algorithms need;
- through an **undirected-only** structure: orientation has nowhere to live;
- through a **hypergraph**: loses direction *and* the binary-edge constraint.

So the CPDAG is not a niche specialization that ought to fit inside a general graph — it is itself the more general object, and the existing structures are each a restriction of it. The general object that subsumes undirected, directed, DAG, CPDAG (and later MAG/PAG) is a **mixed graph with typed edge endpoints**.

## 3. What the repo already has, and why each does not fit

| Structure | Crate | Edge model | Node data | Verdict |
|---|---|---|---|---|
| `UltraGraph` | `ultragraph` | single **directed** projection; CSR-backed; explicit `freeze`/`unfreeze` split for build-once/query-many at scale | payload `N`, weight `W` | Expresses a **DAG** well. Cannot hold undirected edges; bidirected encoding breaks acyclicity. Optimized for large static graphs, not a small mutation-heavy CPDAG. |
| `topology::Graph<T>` | `deep_causality_topology` | **undirected** (`add_edge` pushes both `u→v` and `v→u`) | `CausalTensor<T>` + comonadic cursor | No direction; carries scalar data a CPDAG does not want. |
| `topology::Hypergraph<T>` | `deep_causality_topology` | **unordered hyperedges** via an `i8` incidence matrix | `CausalTensor<T>` + cursor | Generalizes edge *arity*, not direction; not a binary-edge model. |

Conclusion: `ultragraph` is the right tool for large directed graphs and remains so. But a CPDAG is small, mutation-interleaved (orientation runs to a fixpoint), dual-edge, and structural (no scalar). No existing type models the dual-edge core.

## 4. The mathematics — this is well studied

### Pure graph theory
- **Bang-Jensen & Gutin, *Digraphs: Theory, Algorithms and Applications*, 2nd ed., Springer (2009).** "Mixed graph" (directed + undirected edges), orientations, acyclicity. The rigorous graph-theoretic foundation.

### Causal graphical models — where the typed-endpoint object is fully developed
- **Andersson, Madigan & Perlman (1997), "A characterization of Markov equivalence classes for acyclic digraphs," *Annals of Statistics* 25(2):505–541.** Defines the **essential graph** — which *is* the CPDAG — explicitly as a graph with both directed and undirected edges, and proves its invariants (the undirected part is a chain graph with chordal components). This is the foundational reference for the exact object.
- **Meek (1995), "Causal inference and causal explanation with background knowledge," *UAI*.** The four orientation rules that complete a PDAG to a CPDAG operate directly on the mixed graph.
- **Spirtes, Glymour & Scheines, *Causation, Prediction, and Search*, 2nd ed., MIT Press (2000).** The textbook; "patterns" = CPDAGs.

### The fully general endpoint-mark object
Each edge endpoint is marked **tail `—`, arrowhead `>`, or circle `∘`**, which subsumes undirected, directed, bidirected, and partially-oriented edges — hence DAG, CPDAG, MAG, PAG:
- **Richardson & Spirtes (2002), "Ancestral graph Markov models," *Annals of Statistics* 30(4):962–1030.** MAGs.
- **Zhang (2008), "On the completeness of orientation rules for causal discovery in the presence of latent confounders and selection bias," *Artificial Intelligence* 172(16–17):1873–1896.** PAGs and the three-mark endpoint calculus — the most general form.

### Reference implementations to mirror
- **Tetrad** (Java, CMU) — origin of the `Endpoint` enum `{TAIL, ARROW, CIRCLE, NULL}` and a generic `Edge` / `EndpointGraph`.
- **causal-learn** (Python, CMU) — `causallearn.graph`: `Endpoint`, `Edge`, `GeneralGraph`. A clean, modern port; the easiest blueprint to read.
- **pcalg** / **bnlearn** (R) — essential-graph (CPDAG) representations and Meek/PC operations.

## 5. Why `deep_causality_topology` is the right home

- The crate **already hosts general combinatorial structures** beside its algebraic-topology core: `Graph` and `Hypergraph`. A mixed graph sits naturally in that family rather than being a domain intrusion.
- It **already has the HKT/comonadic scaffolding** a new structure plugs into: `GraphWitness` / `HypergraphWitness` (HKT witnesses), the `GraphTopology` / `HypergraphTopology` trait family, and comonadic cursors. A `MixedGraph` would add a `MixedGraphWitness` and a `MixedGraphTopology` trait — extending the higher-kinded-type unification already present, not bolting on a foreign pattern.
- Keeping the **general structure** in `topology` and the **causal-specific semantics** (Meek rules, unshielded-collider validity, MEC sizing) in `deep_causality_algorithms` respects the layering: structures in the structures crate, algorithms in the algorithms crate. The algorithms crate consumes the topology type.

This also makes the structure reusable well beyond BRCD — any future causal-discovery work (PC, GES, FCI/PAGs) builds on the same primitive instead of re-deriving graph plumbing.

## 6. Design sketch (for discussion)

A typed-endpoint mixed graph, structural by default:

```text
enum Mark { Empty, Tail, Arrow }        // (add Circle later for PAGs)
struct MixedGraph<T = ()> {
    n: usize,
    // endpoint mark at v on edge (u, v); the pair (mark[u][v], mark[v][u]) names the edge:
    //   arc u → v   : (Arrow at v, Tail at u)
    //   undirected  : (Tail,  Tail)
    //   no edge     : (Empty, Empty)
    marks: /* n×n marks, or sparse adjacency */,
    data: /* optional node payload, () when structural */,
}
```

Derived queries are local: adjacency is a mark pair; `parents(v)` are arcs into `v`; `neighbors(v)` are undirected; orienting `u — v → u → v` flips one mark; the arc-projection topological sort / cycle check is Kahn over the arrowhead pairs.

Key design questions to settle in the spec:
1. **Endpoint vocabulary now:** two marks `{Tail, Arrow}` (sufficient for DAG/CPDAG today) vs. three `{Tail, Arrow, Circle}` (PAG-ready, larger surface). Recommendation: design the enum and API for three, implement/validate the two needed now, leave `Circle` paths explicitly unimplemented-but-reserved.
2. **Structural vs. data-bearing:** default `T = ()` (scalar-free) to match a CPDAG, with optional node payload to match the `Graph<T>`/`Hypergraph<T>` family and the comonadic cursor. Decide whether the cursor/comonad is in scope for v1.
3. **Storage:** dense `n×n` mark matrix (textbook, invariant is a one-line symmetry check, fine for small graphs) vs. sparse adjacency (idiomatic, O(log) ops, invariant spread across structures). For causal graphs (tens–hundreds of nodes) the dense matrix is simplest and safest; revisit if a large-graph consumer appears.
4. **HKT integration scope:** which of `MixedGraphWitness` / `MixedGraphTopology` / comonad land in v1 vs. follow-up.
5. **Acyclicity engine:** self-contained Kahn (≈20 lines, no dependency) vs. delegating to `ultragraph` on demand (reuses tested code but reintroduces a freeze/thaw round-trip). Recommendation: self-contained, given the graph is small and the algorithm is trivial.

## 7. Effort and sequencing

Building a graph structure properly — invariants, constructors, getters, mutation, traversal, the HKT/comonad integration, and exhaustive tests — is a **meticulous, meaningful addition** to `deep_causality_topology`, on the order of the existing `Graph`/`Hypergraph` work. It is **not** something to fold into `brcd-prep-foundations`.

Proposed sequencing:
1. **New change set: `mixed-graph` in `deep_causality_topology`.** Deliver `MixedGraph` (typed endpoints, structural, HKT-integrated) with full tests, as a first-class topology structure.
2. **Revise `brcd-prep-foundations`.** Decision **D2** currently puts the causal-graph *type* in `deep_causality_algorithms`. Change it to: the mixed-graph *type* lives in `topology`; only the causal *operations* (Meek orientation, unshielded-collider validity, MEC sizing) live in `deep_causality_algorithms`, consuming the topology type. Update the proposal, design, the `causal-graph` spec, and tasks 2.2–2.7 accordingly.
3. Tier A of `brcd-prep-foundations` (cg_solve lift, tensor stats, parents accessor) is already complete and unaffected; it can proceed/commit independently of this note.

## 8. Open questions for the owner

- Confirm the home: `deep_causality_topology` (recommended) vs. a standalone graph crate.
- Resolve the five design questions in §6 (endpoint vocabulary, structural vs. data-bearing, storage, HKT scope, acyclicity engine).
- Confirm the sequencing in §7 — in particular that `brcd-prep-foundations`'s Tier B is paused pending the `mixed-graph` change, while its completed Tier A stands.
