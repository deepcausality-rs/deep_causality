## Why

Constraint-based causal discovery (PC, GES, the Meek orientation rules) and the planned BRCD root-cause estimator operate on a **CPDAG** — a graph that carries directed arcs `u → v` and undirected edges `u — v` at the same time. No structure in the repository models this: `ultragraph` is directed-only, `topology::Graph` is undirected-only, and `topology::Hypergraph` models unordered hyperedges. A CPDAG is not a special case of any of these — it is a *superset* edge model (three states per node pair), so no single-edge-kind structure can express it. The general object that does is a **mixed graph with typed edge endpoints**, a well-studied primitive (Andersson–Madigan–Perlman 1997; Zhang 2008) that the repo is missing. See `openspec/notes/mixed-graph.md` for the full rationale.

## What Changes

- Add a **`MixedGraph`** structure to `deep_causality_topology`: a graph whose edges carry a **mark at each endpoint** (`Tail`, `Arrow`, and a reserved `Circle`), so a single type expresses undirected, directed, and partially-directed edges — and therefore DAGs, PDAGs, and CPDAGs.
- Provide constructors, getters, edge mutation, and structural queries: adjacency, parents/children (arc projection), undirected neighbors, edge enumeration by kind, and arc-projection acyclicity / topological order.
- Enforce the **endpoint invariant**: each node pair is in exactly one state (no edge / directed either way / undirected), with symmetric, consistent endpoint marks.
- Integrate with the crate's existing higher-kinded-type scaffolding by adding a `MixedGraphWitness` (HKT) and a `MixedGraphTopology` trait, mirroring `Graph`/`Hypergraph`.
- Keep the structure **scalar-free by default** (structural over `usize` indices), with optional node payload to match the `Graph<T>`/`Hypergraph<T>` family.
- The `Circle` endpoint mark and any PAG/MAG-specific behavior are **declared but not implemented** in this change — reserved so the type does not need a breaking signature change when latent-variable graphs are added later.

## Capabilities

### New Capabilities
- `mixed-graph`: a typed-endpoint mixed graph in `deep_causality_topology` modeling directed, undirected, and partially-directed edges under one type, with the endpoint invariant, structural queries (parents, neighbors, arcs, undirected edges), arc-projection acyclicity / topological sort, and HKT integration.

### Modified Capabilities
<!-- None. No existing spec in openspec/specs/ changes its requirements. The downstream
     brcd-prep-foundations change will be revised separately to consume this type. -->

## Impact

- **New code:** `deep_causality_topology` gains a `MixedGraph` type (constructors, getters, ops, topology trait, HKT witness) and its test suite. No external numeric crates; `unsafe_code = "forbid"`; static dispatch (no `dyn`); full test coverage of new code.
- **No behavior change** to existing types; `Graph`, `Hypergraph`, and the topology core are untouched.
- **Downstream (separate changes):** `brcd-prep-foundations` decision **D2** will be revised so its causal-graph *operations* (Meek orientation, unshielded-collider validity, MEC sizing) live in `deep_causality_algorithms` and consume this `MixedGraph` instead of defining a graph type there. That revision is out of scope here.
