# Topology Examples

Examples for the `deep_causality_topology` crate. Coverage spans graphs, simplicial
complexes, cubical (lattice) complexes, manifolds, differential forms, Hodge ⋆, and
lattice gauge fields. Every numerical surface is parameterized over `R: RealField`;
the examples below pick a concrete `FloatType` (usually `f64`) at the top of `main`.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

## Graphs and combinatorial complexes

| File | Description | Command |
|------|-------------|---------|
| [basic_graph.rs](basic_graph.rs) | Construct a `Graph<T>` with nodes and edges; basic queries (neighbors, degree); the simplest topological structure | `cargo run -p mathematics_examples --example basic_graph_examples` |
| [chain_algebra.rs](chain_algebra.rs) | Boundary and coboundary operators on a simplicial complex; ∂² = 0 verified numerically | `cargo run -p mathematics_examples --example chain_algebra_examples` |
| [complex_operators.rs](complex_operators.rs) | Higher-level chain-complex operations: Betti numbers, Euler characteristic, oriented incidence | `cargo run -p mathematics_examples --example complex_operators_examples` |

## Manifolds and differential geometry

| File | Description | Command |
|------|-------------|---------|
| [manifold_analysis.rs](manifold_analysis.rs) | Constructing a `Manifold<SimplicialComplex<R>, F>`; topological invariants (dimension, Euler characteristic, orientation) | `cargo run -p mathematics_examples --example manifold_analysis_examples` |
| [differential_field.rs](differential_field.rs) | Discrete Laplacian (`δd + dδ`) on a triangle mesh; heat-equation diffusion via the Hodge ⋆ machinery | `cargo run -p mathematics_examples --example differential_field_examples` |

## Cubical complexes and gauge fields

| File | Description | Command |
|------|-------------|---------|
| [cubical_heat_diffusion.rs](cubical_heat_diffusion.rs) | Explicit-Euler heat diffusion on a `Manifold<CubicalComplex<2, FloatType>, FloatType>` with a Moore-neighborhood stencil; prints an ASCII heatmap each step | `cargo run -p mathematics_examples --example cubical_heat_diffusion_examples` |
| [lattice_gauge_simulation.rs](lattice_gauge_simulation.rs) | SU(3) lattice gauge theory on a 4⁴ spacetime: hot start, Metropolis thermalization, plaquette / Wilson-loop / Polyakov-loop measurements, APE smearing, Wilson gradient flow | `cargo run -p mathematics_examples --example lattice_gauge_simulation_examples` |

## HKT composition

| File | Description | Command |
|------|-------------|---------|
| [hkt_graph_convolution.rs](hkt_graph_convolution.rs) | One-layer GCN-style message passing via `GraphWitness::extend` (CoMonad) and `GraphWitness::fmap` (Functor); the comonad walks every node, the closure reads neighbors | `cargo run -p mathematics_examples --example hkt_graph_convolution_examples` |
