# Foundation: `deep_causality_topology`

Discrete geometry: graphs, simplicial complexes, manifolds, and the boundary operators that
connect their dimensions.

| Example | What it covers | Command |
|---|---|---|
| [basic_graph.rs](basic_graph.rs) | `Graph` with CSR adjacency and tensor-valued node payload: edges, degree, neighbours, and the undirected `add_edge` contract | `cargo run -p mathematics_examples --example basic_graph_examples` |
| [complex_operators.rs](complex_operators.rs) | `PointCloud::triangulate` building a simplicial complex, and the boundary and coboundary operators it carries per dimension | `cargo run -p mathematics_examples --example complex_operators_examples` |
| [chain_algebra.rs](chain_algebra.rs) | `∂∂ = 0` on a tetrahedron: a 2-chain, its boundary, and the boundary of that boundary vanishing | `cargo run -p mathematics_examples --example chain_algebra_examples` |
| [manifold_analysis.rs](manifold_analysis.rs) | `Manifold` construction with validation, dimension, Euler characteristic and orientation | `cargo run -p mathematics_examples --example manifold_analysis_examples` |

The boundary operators hold `i8`. Their entries are the incidence signs `-1`, `0` and `+1`,
which is combinatorics, so the working scalar stays out of them.

Fourteen witness types live in this crate; `2_composition/extension/` shows them under `extend`.
