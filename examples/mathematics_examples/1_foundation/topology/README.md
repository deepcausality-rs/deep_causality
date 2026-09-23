# Foundation: `deep_causality_topology`

Discrete geometry: graphs, simplicial complexes, manifolds, and the boundary operators that
connect their dimensions.

| Example | What it covers | Command |
|---|---|---|
| [basic_graph.rs](basic_graph.rs) | `Graph` with CSR adjacency and tensor-valued node payload: edges, degree, neighbours, and the undirected `add_edge` contract | `cargo run -p mathematics_examples --example basic_graph_examples` |
| [complex_operators.rs](complex_operators.rs) | `PointCloud::triangulate` building a simplicial complex, and the boundary and coboundary operators it carries per dimension | `cargo run -p mathematics_examples --example complex_operators_examples` |
| [chain_algebra.rs](chain_algebra.rs) | `∂∂ = 0` on a tetrahedron: a 2-chain, its boundary, and the boundary of that boundary vanishing | `cargo run -p mathematics_examples --example chain_algebra_examples` |
| [manifold_analysis.rs](manifold_analysis.rs) | `Manifold` construction with validation, dimension, Euler characteristic and orientation | `cargo run -p mathematics_examples --example manifold_analysis_examples` |
| [chain_cochain_pairing.rs](chain_cochain_pairing.rs) | `ChainWitness` and `CochainWitness`: the pairing `⟨ω, c⟩` as a discrete line integral, an exact cochain vanishing on a closed loop, and `fmap` on either side moving through it | `cargo run -p mathematics_examples --example chain_cochain_pairing_examples` |
| [neighbourhoods_three_ways.rs](neighbourhoods_three_ways.rs) | `HypergraphWitness`, `MixedGraphWitness` and `PointCloudWitness`: one smoothing law under `extend`, and three answers to what a neighbour is | `cargo run -p mathematics_examples --example neighbourhoods_three_ways_examples` |
| [fields_on_complexes.rs](fields_on_complexes.rs) | `CellComplexWitness`, `LatticeComplexWitness` and `TopologyWitness`: `Functor` and `Foldable` across a honeycomb, a lattice and a simplicial patch, and the `CoMonad` only one of them carries | `cargo run -p mathematics_examples --example fields_on_complexes_examples` |
| [manifold_curvature.rs](manifold_curvature.rs) | `CurvatureTensorWitness` carrying `RiemannMap`, checked against the closed form for constant curvature, and `GenericManifoldWitness` over a cubical lattice | `cargo run -p mathematics_examples --example manifold_curvature_examples` |

The boundary operators hold `i8`: their entries are the incidence signs `-1`, `0` and `+1`, so
the working scalar stays out of them.

Fourteen witness types live in this crate; `2_composition/extension/` shows them under `extend`.
