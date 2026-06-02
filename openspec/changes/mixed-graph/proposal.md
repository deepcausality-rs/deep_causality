## Why

Constraint-based causal discovery (PC, GES, the Meek orientation rules) and the planned BRCD root-cause estimator operate on a **CPDAG** — a graph that carries directed arcs `u → v` and undirected edges `u — v` at the same time. No structure in the repository models this: `ultragraph` is directed-only, `topology::Graph` is undirected-only, and `topology::Hypergraph` models unordered hyperedges. A CPDAG is not a special case of any of these — it is a *superset* edge model (three states per node pair), so no single-edge-kind structure can express it. The general object that does is a **mixed graph with typed edge endpoints**, a well-studied primitive (Andersson–Madigan–Perlman 1997; Zhang 2008) that the repo is missing. See `openspec/notes/mixed-graph.md` for the full rationale.

## What Changes

- Add a **`MixedGraph`** structure to `deep_causality_topology`: a graph whose edges carry a **mark at each endpoint** (`Tail`, `Arrow`, `Circle`), so a single type expresses directed, undirected, bidirected, and partially-directed edges — and therefore DAGs, PDAGs, CPDAGs, MAGs, and PAGs. The **full three-mark calculus is implemented** (not Circle-reserved): building the PAG-capable type once avoids a later breaking migration.
- Store edges in a **canonical-pair edge map** (`BTreeMap<(min, max), Edge>`) — one entry per unordered pair — so the **endpoint invariant** (at most one edge per pair, consistent marks) is enforced structurally by the key and uses `O(m)` memory.
- Provide constructors, getters, edge mutation (per-endpoint orientation across all marks), and structural queries: adjacency, parents/children (directed-arc projection), undirected neighbors, per-kind edge enumeration, and arc-projection acyclicity / topological order.
- Integrate to **full parity** with the crate's higher-kinded-type family: a `MixedGraphWitness` (HKT), a `MixedGraphTopology` trait, **and** a comonadic interface (`extract`/`extend`/`duplicate`) over a node-payload cursor — mirroring `Graph`/`Hypergraph`.
- Carry a node payload `T` like `Graph<T>`/`Hypergraph<T>`; usable structurally (over `usize` indices) when the payload holds no scalar.

## Capabilities

### New Capabilities
- `mixed-graph`: a full three-mark typed-endpoint mixed graph in `deep_causality_topology` modeling directed, undirected, bidirected, and partially-directed edges (DAG/CPDAG/MAG/PAG) under one type, with the structurally-enforced endpoint invariant, per-endpoint orientation, structural queries (parents, neighbors, per-kind enumeration), arc-projection acyclicity / topological sort, and full HKT + comonad integration.

### Modified Capabilities
<!-- None. No existing spec in openspec/specs/ changes its requirements. The downstream
     brcd-prep-foundations change will be revised separately to consume this type. -->

## Impact

- **New code:** `deep_causality_topology` gains a `MixedGraph` type (constructors, getters, ops, topology trait, HKT witness) and its test suite. No external numeric crates; `unsafe_code = "forbid"`; static dispatch (no `dyn`); full test coverage of new code.
- **No behavior change** to existing types; `Graph`, `Hypergraph`, and the topology core are untouched.
- **Downstream (separate changes):** `brcd-prep-foundations` decision **D2** will be revised so its causal-graph *operations* (Meek orientation, unshielded-collider validity, MEC sizing) live in `deep_causality_algorithms` and consume this `MixedGraph` instead of defining a graph type there. That revision is out of scope here.
