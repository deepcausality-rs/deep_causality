# DeepCausality Topology

**Topological Data Analysis (TDA) and Causal Geometry for Rust**

`deep_causality_topology` is a core crate of the `deep_causality` project, providing rigorous topological data structures and algorithms for causal modeling, geometric deep learning, and complex systems analysis.

It bridges the gap between discrete data (graphs, point clouds) and continuous geometric structures (manifolds, simplicial complexes), enabling advanced reasoning about the "shape" and connectivity of causal systems.

## Features

*   **Comprehensive Topological Types**:
    *   **Graph**: Efficient sparse-matrix based graphs for causal networks.
    *   **Hypergraph**: Modeling higher-order relationships (hyperedges) between multiple nodes.
    *   **SimplicialComplex**: Generalizing graphs to higher dimensions (triangles, tetrahedra) to capture voids and holes.
    *   **Manifold**: Validated geometric structures for differential geometry operations.
    *   **PointCloud**: Raw multi-dimensional data with Vietoris-Rips triangulation capabilities.
*   **Topological Algorithms**:
    *   **Vietoris-Rips Triangulation**: Convert point clouds into simplicial complexes at a given scale.
    *   **Euler Characteristic**: Compute topological invariants ($\chi$) to classify shapes (e.g., healthy vs. pathological tissue).
    *   **Boundary/Coboundary Operators**: Sparse matrix operators for algebraic topology computations.
*   **Algebraic Topology & Differential Geometry**:
    *   **Chain Algebra**: Perform algebraic operations on chains (formal sums of simplices) and verify fundamental topological theorems like `∂∂=0`.
    *   **Differential Operators**: Compute the exterior derivative (`d`), Hodge star (`⋆`), codifferential (`δ`), and Hodge-Laplacian (`Δ`) on discrete differential forms.
    *   **Hodge Theory**: Detect topological features like holes and voids by finding harmonic forms (solutions to `Δω = 0`).
*   **Higher-Kinded Types (HKT)**:
    *   Implements `Functor`, `BoundedComonad` (Extract/Extend), and `BoundedAdjunction` (Unit/Counit) via `deep_causality_haft`.
    *   Enables functional geometric patterns like "neighborhood extraction" (Comonad) and "geometric realization" (Adjunction).

## Core Concepts

| Type | Description | Mathematical Structure |
| :--- | :--- | :--- |
| **PointCloud** | Set of points in $\mathbb{R}^n$ with metadata. | Discrete Metric Space |
| **Graph** | Nodes and binary edges. | 1-Complex |
| **Hypergraph** | Nodes and hyperedges (subsets of nodes). | Hypergraph |
| **SimplicialComplex** | Collection of simplices (points, lines, triangles...) closed under sub-simplices. | Simplicial Complex $K$ |
| **Manifold** | A topological space that is locally Euclidean. | Manifold $M$ |

## Usage

Add this crate to your `Cargo.toml`:

```toml
deep_causality_topology = { version = "0.1" }
```

### 1. Basic Graph Construction

Efficiently model causal dependencies using sparse adjacency matrices.

```rust
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphTopology};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Adjacency: 0->1, 1->2
    let adj = CsrMatrix::from_triplets(3, 3, &[(0, 1, 1), (1, 2, 1)])?;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3])?;
    
    let graph = Graph::new(adj, data, 0)?;
    
    println!("Neighbors of 1: {:?}", graph.get_neighbors(1)?);
    Ok(())
}
```

### 2. Manifold Analysis (Euler Characteristic)

Validate geometric shapes and compute topological invariants.

```rust
use deep_causality_topology::{Manifold, SimplicialComplex, Simplex, Skeleton};
// ... (Setup complex) ...

// Compute Euler Characteristic: Chi = V - E + F ...
let chi = manifold.euler_characteristic();
// Chi = 1 (Contractible/Solid)
// Chi = 0 (Hole/Circle)
// Chi = 2 (Sphere)
```

### 3. Point Cloud Triangulation (i.e. MRI Image Analysis)

Convert raw sensor data into structured geometry to detect anomalies (e.g., tumors/voids).

```rust
use deep_causality_topology::PointCloud;

// 1. Ingest Raw MRI Data
let pc = PointCloud::new(points_tensor, metadata, 0)?;

// 2. Triangulate (Vietoris-Rips)
let complex = pc.triangulate(0.6)?;

// 3. Diagnose based on Topology
let chi = complex.euler_characteristic();
if chi <= 0 {
    println!("Pathological: Detected Void/Necrosis");
}
```

### 4. Differential Geometry (Heat Equation)

Simulate physical processes like diffusion on complex geometries using the Hodge-Laplacian.

```rust
use deep_causality_topology::{Manifold, PointCloud};
// ... (Setup manifold from a PointCloud) ...

// Set initial heat distribution (a 0-form)
// ...

// Time-step the heat equation: ∂u/∂t = -Δu
for _ in 0..num_steps {
    let laplacian = manifold.laplacian(0);
    // ... update data using: new_data = current_data - dt * laplacian
}
```

## Higher-Kinded Types (HKT)

This crate leverages `deep_causality_haft` to provide functional geometric abstractions.

*   **Functor**: Map functions over the data stored in the topology (e.g., transform node weights).
*   **BoundedComonad (Extract/Extend)**:
    *   *Extract*: Get the value at the current "cursor" (focus).
    *   *Extend*: Apply a local computation (convolution) over the neighborhood of every point to produce a new topology. This is the foundation of **Graph Neural Networks (GNNs)** and **Cellular Automata**.
*   **BoundedAdjunction (Unit/Counit)**:
    *   *Unit*: Embed discrete data into a topological structure.
    *   *Counit*: Project/Integrate topological data back into a flat representation.

## Examples

| File Name | Description | Engineering Value |
| :--- | :--- | :--- |
| `basic_graph.rs` | Graph construction & traversal | Foundation for large-scale causal network modeling. |
| `manifold_analysis.rs` | 1-Manifold validation & Euler Char. | Ensuring geometric validity for differential operators. |
| `point_cloud_triangulation.rs` | MRI Tissue Segmentation | Bridging raw sensor data with topological reasoning for diagnosis. |
| `chain_algebra.rs` | Chain complex algebra & `∂∂=0` | Foundational verification for homological algebra. |
| `differential_field.rs` | Solving the Heat Equation on a manifold | Simulating physical diffusion processes on complex shapes. |
| `hodge_theory.rs` | Finding harmonic forms to detect holes | Advanced topological feature detection using the Hodge-Laplacian. |

To run examples:
```bash
cargo run -p deep_causality_topology --example point_cloud_triangulation
```

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## Author

*   [Marvin Hansen](https://github.com/marvin-hansen).
*   Github GPG key ID: 369D5A0B210D39BC
*   GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
