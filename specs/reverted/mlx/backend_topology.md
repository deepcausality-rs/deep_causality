# Generic Backend: Topology Specification

## 1. Overview

The `deep_causality_topology` crate deals with complex networks, hypergraphs, and simplicial complexes. While structural
manipulation (adding nodes/edges) is best suited for the CPU, **global analysis** (spectral properties, diffusion,
centrality) benefits massively from hardware acceleration via Linear Algebra.

This specification defines the **Graph-Linear Algebra Bridge**, enabling topology structures to utilize the
`TensorBackend` for compute-heavy operations.

---

## 2. Architecture

### 2.1 The Tensor-Topology Bridge

Instead of replacing the existing rich graph structures (which hold arbitrary payload data), we provide **Projection
Views** that map the topological structure to backend tensors.

```rust
// deep_causality_topology/src/backend/mod.rs

/// An accelerated view of a topology structure
pub struct TopologyView<B: TensorBackend, T: TensorData> {
    /// Adjacency Matrix (Graph) or Incidence Matrix (Hypergraph)
    /// Stored on Device (CPU/MLX/CUDA)
    matrix: B::Tensor<T>,

    /// Node Degrees (Diagonal components)
    degrees: B::Tensor<T>,

    /// Mapping from Matrix Index to Node ID (on Host CPU)
    index_map: Vec<u64>,
}
```

### 2.2 Constructing Views

```rust
impl<B: TensorBackend, T: TensorData> TopologyView<B, T> {
    /// Projects a CausalGraph into a TensorBackend Adjacency Matrix
    pub fn from_graph(graph: &CausalGraph) -> Self {
        // 1. Enumerate nodes -> index_map
        // 2. Build adjacency matrix on host (CSR or Dense)
        // 3. Upload to Backend via B::create()
    }

    /// Projects a HyperGraph into a TensorBackend Incidence Matrix
    /// Shape: [Num_Nodes, Num_HyperEdges]
    pub fn from_hypergraph(hypergraph: &HyperGraph) -> Self { ... }
}
```

---

## 3. Operations

Once projected to the backend, we can perform global O(N³) or O(E) operations efficiently.

### 3.1 Spectral Analysis (Laplacian)

```rust
impl<B: TensorBackend, T: TensorData> TopologyView<B, T> {
    /// Computes the Normalized Laplacian: L = I - D^(-1/2) A D^(-1/2)
    pub fn normalized_laplacian(&self) -> B::Tensor<T> {
        // 1. Compute D^(-1/2)
        // 2. Matmul: D_inv_sqrt @ A @ D_inv_sqrt
        // 3. Subtract from Identity
        // ALL operations run on Backend (MLX/CUDA)
    }

    /// Computes Spectral Gap (Algebraic Connectivity)
    /// Requires B::eigen or B::svd support
    pub fn spectral_gap(&self) -> T {
        let l = self.normalized_laplacian();
        // Backend specific solver (e.g., MLX linalg.eigvalsh)
        // Returns the second smallest eigenvalue (Fiedler value)
    }
}
```

### 3. Diffusion (Information Flow)

Simulating how information spreads through the causal network over time steps.

```rust
impl<B: TensorBackend, T: TensorData> TopologyView<B, T> {
    /// Simulates diffusion for k steps: X_{t+1} = A * X_t
    pub fn diffuse(&self, initial_state: &B::Tensor<T>, steps: usize) -> B::Tensor<T> {
        let mut state = initial_state.clone();
        for _ in 0..steps {
            state = B::matmul(&self.matrix, &state);
        }
        state
    }
}
```

### 3.4 Manifold Acceleration

Manifolds require heavy tensor calculus (Einstein Summation) to compute curvature and geodesics. These operations map
perfectly to the Generic Backend.

**Key Operations:**

1. **Metric Tensor ($g_{mn}$):** Stored as a generic tensor field.
2. **Inverse Metric ($g^{mn}$):** Computed via `B::inverse(g)`.
3. **Christoffel Symbols ($\Gamma^k_{ij}$):**
   $$ \Gamma^k_{ij} = \frac{1}{2} g^{kl} (\partial_j g_{il} + \partial_i g_{jl} - \partial_l g_{ij}) $$
    * Implemented via `B::slice` (finite differences) and `B::matmul/contract`.
4. **Geodesics:**
   $$ \frac{d^2x^k}{d\lambda^2} + \Gamma^k_{ij} \frac{dx^i}{d\lambda} \frac{dx^j}{d\lambda} = 0 $$
    * Solved via Runge-Kutta integration using generic tensor arithmetic.

```rust
// deep_causality_topology/src/backend/manifold.rs

/// Accelerated Manifold Chart
pub struct ManifoldView<B: LinearAlgebraBackend, T: TensorData> {
    /// Metric tensor field g_{mn}
    /// Shape: [Batch/Grid, Dim, Dim]
    metric_tensor: B::Tensor<T>,

    /// Precomputed inverse metric
    inverse_metric: B::Tensor<T>,
}

impl<B: LinearAlgebraBackend, T: TensorData> ManifoldView<B, T> {
    pub fn compute_christoffel(&self) -> B::Tensor<T> {
        // Backend accelerated tensor contraction
    }
}
```

---

## 4. Type Aliases (Ergonomics)

To avoid verbose `<B, T>` syntax everywhere, we define standard aliases for the generic backend.

```rust
pub mod types {
    use super::*;

    /// Adjacency Matrix mapped to backend (Shape: [N, N])
    pub type AdjacencyMatrix<B, T> = B::Tensor<T>;

    /// Incidence Matrix mapped to backend (Shape: [Nodes, Edges])
    pub type IncidenceMatrix<B, T> = B::Tensor<T>;

    /// Laplacian Matrix (L = D - A)
    pub type LaplacianMatrix<B, T> = B::Tensor<T>;

    /// Manifold Metric Tensor (g_{mn})
    pub type MetricTensor<B, T> = B::Tensor<T>;
}
```

---

## 5. Backend Implementation Requirements

To support Topology and Manifolds, the backend needs:

| Operation | Usage            | CPU Support      | MLX Support      |
|-----------|------------------|------------------|------------------|
| `matmul`  | Adjacency/Metric | `ndarray::dot`   | `mlx.matmul`     |
| `inverse` | Manifold Metric  | `ndarray-linalg` | `mlx.linalg.inv` |
| `eig`     | Spectral Gap     | `ndarray-linalg` | `mlx.linalg.eig` |
| `pow`     | Diffusion        | `Array::pow`     | `mlx.power`      |

---

## 5. Precision Handling

* **Topology Constraints:** Graph indices are Integers, but Adjacency Matrices for spectral analysis are Floats.
* **CPU:** Verification can use `f64` for high-precision centrality checks (avoiding rank reversal bugs).
* **MLX:** `f32` is sufficient for most GNN/Diffusion tasks and approximate spectral clustering.

---

## 6. Migration Strategy

1. **Backend Trait Update:** Ensure a separate `LinalgBackend` trait.
2. **Topology Crate:** Add `deep_causality_topology::backend`.
3. **Implement View:** Create the `TopologyView` struct that holds the generic tensor.
4. **Integration:** Add `to_backend_view<B>()` methods to `CausalGraph` and `HyperGraph`.

---

## 7. Example: Spectral Clustering

```rust
// User Code
use deep_causality_topology::{CausalGraph, backend::TopologyView};
use deep_causality_tensor::backend::MlxBackend;

// 1. Standard Graph (CPU structure)
let graph = CausalGraph::new();
// ... add 100k nodes ...

// 2. Project to Backend (GPU Upload)
// MLX Backend selected, f32 precision
let view = TopologyView::<MlxBackend, f32>::from_graph( & graph);

// 3. Compute Laplacian (GPU Accelerated)
let laplacian = view.normalized_laplacian();

// 4. Get Eigenvalues (GPU Accelerated)
// Using MLX's Linear Algebra library
let eigenvalues = view.eigenvalues( & laplacian);
```

---

## 8. Default Type Aliases (Feature-Flag Based)

```rust
// deep_causality_topology/src/backend/aliases.rs

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultBackend = MlxBackend;

#[cfg(not(feature = "mlx"))]
pub type DefaultBackend = CpuBackend;

/// Convenience Alias: Backend-agnostic TopologyView
pub type GraphView<T = f32> = TopologyView<DefaultBackend, T>;

/// Convenience Alias: Backend-agnostic ManifoldView
pub type ChartView<T = f64> = ManifoldView<DefaultBackend, T>;
```

**Usage:**

```rust
use deep_causality_topology::backend::GraphView;

let view: GraphView = GraphView::from_graph( & my_graph);
let laplacian = view.normalized_laplacian(); // Auto-dispatches to MLX/CPU
```

---

## 9. Testing Strategy

### 9.1 Test Structure

```
deep_causality_topology/tests/
├── backend/
│   ├── mod.rs
│   ├── topology_view_tests.rs
│   ├── manifold_view_tests.rs
│   └── backend_parity_tests.rs  # [feature-gated]
```

### 9.2 Test Categories

| Category                   | Description                              |
|----------------------------|------------------------------------------|
| **Graph Projection**       | Adjacency matrix matches graph structure |
| **Laplacian Properties**   | Symmetric, row sums = 0                  |
| **Spectral Accuracy**      | Known graphs have known eigenvalues      |
| **Diffusion Conservation** | Total probability conserved              |
| **Manifold Curvature**     | Flat space → Christoffel = 0             |
| **Backend Parity**         | CPU == MLX within tolerance              |

### 9.3 Example Tests

```rust
#[test]
fn complete_graph_laplacian_eigenvalues() {
    // K_n has eigenvalue 0 (multiplicity 1) and n (multiplicity n-1)
    let k5 = CausalGraph::complete(5);
    let view = TopologyView::<CpuBackend, f64>::from_graph(&k5);
    let eigenvalues = view.laplacian_eigenvalues();

    assert!((eigenvalues[0] - 0.0).abs() < 1e-10);
    assert!((eigenvalues[1] - 5.0).abs() < 1e-10);
}

#[test]
fn flat_minkowski_christoffel_vanishes() {
    let eta = ManifoldView::<CpuBackend, f64>::minkowski(4);
    let christoffel = eta.compute_christoffel();

    assert!(christoffel.max_abs() < 1e-14, "Flat space Γ should be zero");
}
```

---

## 10. RustDoc Guidelines

```rust
/// An accelerated projection of a topology structure onto a tensor backend.
///
/// `TopologyView` converts graph/hypergraph structures into dense adjacency
/// or incidence matrices for GPU-accelerated linear algebra operations.
///
/// # Type Parameters
///
/// * `B` - The compute backend ([`CpuBackend`], [`MlxBackend`]).
/// * `T` - The scalar type for matrix elements.
///
/// # Example
///
/// ```rust
/// use deep_causality_topology::{CausalGraph, backend::TopologyView};
/// use deep_causality_tensor::backend::CpuBackend;
///
/// let graph = CausalGraph::new();
/// let view = TopologyView::<CpuBackend, f64>::from_graph(&graph);
/// let laplacian = view.normalized_laplacian();
/// ```
pub struct TopologyView<B: TensorBackend, T: TensorData> {
    ...
}
```
