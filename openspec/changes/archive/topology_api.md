# Topology Crate API Refactoring Specification

## Problem Statement

The `deep_causality_topology` crate lacks a consistent, isolated API layer that separates public interfaces from implementation details. This makes MLX acceleration difficult to implement because:

1. **No clear dispatch point** - Public methods directly contain implementation logic
2. **Inconsistent module structure** - Each type has different file organization
3. **No `_cpu` / `_mlx` pattern** - Unlike `deep_causality_tensor` and `deep_causality_multivector`

## Design Goal

Establish a consistent API pattern across all topology types:
- **Public API** in `api/` submodule
- **Implementations** organized by trait/operation (e.g., `geometry/`, `differential/`)
- **CPU and MLX** implementations side-by-side in same folder
- **Transparent dispatch** from API to backend

---

## Proposed Structure

### Unified Module Layout (Per Type)

```
types/<type_name>/
├── mod.rs              # Struct definition, re-exports
├── api/                # PUBLIC API layer (thin dispatch)
│   ├── mod.rs
│   └── <operation>.rs  # Public methods dispatching to _cpu or _mlx
├── <operation>/        # Implementations grouped by operation/trait
│   ├── mod.rs
│   ├── <operation>_cpu.rs  # CPU implementation
│   └── <operation>_mlx.rs  # MLX implementation (feature-gated)
├── getters/            # Accessor methods
│   ├── mod.rs
│   └── getters.rs
├── display/            # Display/Debug impls
│   ├── mod.rs
│   └── display.rs
└── utils/              # Type-specific utilities
    ├── mod.rs
    └── utils_<type>.rs # i.e.  utils_manifold.rs
```

---

## Per-Type Refactoring Plan

### 1. Manifold

**Current Structure:**
```
manifold/
├── mod.rs              # 6.4KB - struct + constructors
├── api.rs              # EMPTY (6 lines)
├── geometry.rs         # 6KB - volume, determinant
├── differential/       # laplacian, hodge, exterior, codifferential
├── base_topology.rs
├── manifold_topology.rs
├── simplicial_topology.rs
└── utils.rs
```

**Proposed Structure:**
```
manifold/
├── mod.rs                    # Struct def, re-exports only
├── api/
│   ├── mod.rs
│   ├── constructors.rs       # new(), with_metric()
│   ├── geometry.rs           # simplex_volume_squared(), determinant()
│   ├── differential.rs       # laplacian(), hodge_star(), exterior_derivative()
│   └── covariance.rs         # covariance_matrix(), eigen_covariance()
├── geometry/                 # Geometry implementations (CPU + MLX side-by-side)
│   ├── mod.rs
│   ├── geometry_cpu.rs       # simplex_volume_squared_cpu(), determinant_cpu()
│   └── geometry_mlx.rs       # simplex_volume_squared_mlx(), determinant_mlx()
├── differential/             # Differential ops (CPU + MLX side-by-side)
│   ├── mod.rs
│   ├── laplacian_cpu.rs
│   ├── laplacian_mlx.rs
│   ├── hodge_cpu.rs
│   ├── hodge_mlx.rs
│   ├── exterior_cpu.rs
│   └── codifferential_cpu.rs
├── covariance/               # Covariance ops (CPU + MLX side-by-side)
│   ├── mod.rs
│   ├── covariance_cpu.rs     # covariance_matrix_cpu(), eigen_covariance_cpu()
│   └── covariance_mlx.rs     # covariance_matrix_mlx(), eigen_covariance_mlx()
├── base_topology/            # BaseTopology trait impl
│   ├── mod.rs
│   ├── base_topology_cpu.rs
│   └── base_topology_mlx.rs
├── manifold_topology/        # ManifoldTopology trait impl
│   ├── mod.rs
│   └── manifold_topology_cpu.rs
├── simplicial_topology/      # SimplicialTopology trait impl
│   ├── mod.rs
│   └── simplicial_topology_cpu.rs
├── getters/
│   ├── mod.rs
│   └── getters.rs
├── display/
│   ├── mod.rs
│   └── display.rs
└── utils/
    ├── mod.rs
    └── utils_manifold.rs
```

**Example API Implementation:**

```rust
// api/geometry.rs - PUBLIC API
impl<T> Manifold<T> {
    /// Public API for simplex volume calculation.
    /// Automatically dispatches to GPU when mlx feature is enabled.
    pub fn simplex_volume_squared(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        let k = simplex.vertices.len() - 1;
        
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        {
            if k >= 4 {
                return self.simplex_volume_squared_mlx(simplex);
            }
        }
        
        self.simplex_volume_squared_cpu(simplex)
    }
}

// geometry/geometry_cpu.rs - CPU IMPLEMENTATION
impl<T> Manifold<T> {
    pub(crate) fn simplex_volume_squared_cpu(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        // Existing CPU implementation
    }
}

// geometry/geometry_mlx.rs - MLX IMPLEMENTATION
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
impl<T> Manifold<T> {
    pub(crate) fn simplex_volume_squared_mlx(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        // GPU-accelerated implementation
    }
}
```

---

### 2. SimplicialComplex

**Current Structure:**
```
simplicial_complex/
├── mod.rs              # Struct def
├── builder.rs          # Builder pattern
├── base_topology.rs
├── simplicial_topology.rs
├── ops_boundary.rs     # Boundary operators
├── getters.rs
└── display.rs
```

**Proposed Structure:**
```
simplicial_complex/
├── mod.rs              # Struct def, re-exports
├── api/
│   ├── mod.rs
│   ├── boundary.rs     # boundary_operator(), coboundary_operator()
│   └── topology.rs     # dimension(), euler_characteristic()
├── boundary/
│   ├── mod.rs
│   ├── boundary_cpu.rs
│   └── boundary_mlx.rs
├── base_topology/
│   ├── mod.rs
│   └── base_topology_cpu.rs
├── simplicial_topology/
│   ├── mod.rs
│   └── simplicial_topology_cpu.rs
├── builder/
│   ├── mod.rs
│   └── builder.rs
├── getters/
│   ├── mod.rs
│   └── getters.rs
└── display/
    ├── mod.rs
    └── display.rs
```

---

### 3. Graph

**Current Structure:**
```
graph/
├── mod.rs              # Struct + ctors + logic mixed
├── base_topology.rs
├── graph_topology.rs
├── getters.rs
├── display.rs
└── clone.rs
```

**Proposed Structure:**
```
graph/
├── mod.rs              # Struct def only
├── api/
│   ├── mod.rs
│   ├── constructors.rs
│   └── graph_ops.rs    # add_edge(), neighbors(), etc.
├── graph_ops/
│   ├── mod.rs
│   └── graph_ops_cpu.rs
├── base_topology/
│   ├── mod.rs
│   └── base_topology_cpu.rs
├── graph_topology/
│   ├── mod.rs
│   └── graph_topology_cpu.rs
├── getters/
│   ├── mod.rs
│   └── getters.rs
├── display/
│   ├── mod.rs
│   └── display.rs
└── clone/
    ├── mod.rs
    └── clone.rs
```

---

### 4. Other Types

Apply same pattern to:
- `Hypergraph`
- `PointCloud`
- `Chain`
- `Skeleton`
- `Simplex`
- `Topology`
- `ReggeGeometry`

---

## Naming Conventions

| Layer | Suffix | Visibility | Example |
|-------|--------|------------|---------|
| Public API | (none) | `pub` | `fn matmul()` |
| CPU impl | `_cpu` | `pub(crate)` | `fn matmul_cpu()` |
| MLX impl | `_mlx` | `pub(crate)` | `fn matmul_mlx()` |

---

## Implementation Priority

| Priority | Type | Reason |
|----------|------|--------|
| 1 | **Manifold** | Most complex, MLX target |
| 2 | **SimplicialComplex** | Used by Manifold |
| 3 | **Graph** | Core type |
| 4 | Hypergraph | Lower priority |
| 5 | Others | Simple types, quick refactor |

---

## Migration Steps

### Step 1: Manifold Refactoring

1. Create `manifold/api/` directory
2. Create `manifold/geometry/` directory
3. Move public methods from `geometry.rs` to `api/geometry.rs`
4. Create `geometry/geometry_cpu.rs` with `_cpu` suffix methods
5. Update `api/geometry.rs` to dispatch to `geometry/`
6. Create `geometry/geometry_mlx.rs` (feature-gated stub)
7. Repeat for `differential/`, `covariance/`, trait folders
8. Update `mod.rs` to re-export API

### Step 2: Test & Verify

```bash
cargo test -p deep_causality_topology
cargo build -p deep_causality_topology --features mlx
```

### Step 3: Repeat for Other Types

---

## API Dispatch Example

```rust
// Final public API pattern
impl<T> Manifold<T> {
    /// Computes eigenvalues of the field covariance matrix.
    pub fn eigen_covariance(&self) -> Result<Vec<f64>, TopologyError>
    where
        T: Into<f64> + Copy,
    {
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        {
            if self.data.len() >= GPU_THRESHOLD {
                return self.eigen_covariance_mlx();
            }
        }
        
        self.eigen_covariance_cpu()
    }
}
```

---

## Folder Organization Principle

Each operation or trait gets its own folder containing:
- `<name>_cpu.rs` - CPU implementation (always present)
- `<name>_mlx.rs` - MLX implementation (feature-gated, added when needed)

**Example:**
```
geometry/
├── mod.rs              # Re-exports both _cpu and _mlx (when enabled)
├── geometry_cpu.rs     # fn determinant_cpu(), fn volume_cpu()
└── geometry_mlx.rs     # fn determinant_mlx(), fn volume_mlx()
```

This keeps related implementations **visually adjacent** for easier maintenance.

---

## mod.rs Content Guidelines

Each `mod.rs` file has a specific purpose. Follow these patterns:

### Type Root mod.rs (e.g., `manifold/mod.rs`)

```rust
/*
 * SPDX-License-Identifier: MIT
 */

//! Manifold type for smooth geometric structures.

// Submodule declarations
mod api;
mod geometry;
mod differential;
mod covariance;
mod base_topology;
mod manifold_topology;
mod simplicial_topology;
mod getters;
mod display;
mod utils;

// Struct definition (ONLY the struct, no methods)
#[derive(Debug, Clone, PartialEq)]
pub struct Manifold<T> {
    pub(crate) complex: SimplicialComplex,
    pub(crate) data: CausalTensor<T>,
    pub(crate) metric: Option<ReggeGeometry>,
    pub(crate) cursor: usize,
}

// Re-export public API (from api/ module)
pub use api::*;
```

### Operation mod.rs (e.g., `geometry/mod.rs`)

```rust
/*
 * SPDX-License-Identifier: MIT
 */

// CPU implementation (always included)
mod geometry_cpu;
pub(crate) use geometry_cpu::*;

// MLX implementation (feature-gated)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod geometry_mlx;
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub(crate) use geometry_mlx::*;
```

### API mod.rs (e.g., `api/mod.rs`)

```rust
/*
 * SPDX-License-Identifier: MIT
 */

// Group public API by operation
mod constructors;
mod geometry;
mod differential;
mod covariance;

// Re-export all public methods
pub use constructors::*;
pub use geometry::*;
pub use differential::*;
pub use covariance::*;
```

---

## Documentation Guidelines

### Public API Methods (Required)

All public methods in `api/` MUST have doc comments:

```rust
// api/geometry.rs
impl<T> Manifold<T> {
    /// Computes the squared volume of a k-simplex using Cayley-Menger determinant.
    ///
    /// GPU-accelerated when `mlx` feature is enabled and k ≥ 4.
    ///
    /// # Arguments
    /// * `simplex` - The simplex to compute volume for
    ///
    /// # Returns
    /// * `Ok(f64)` - The squared volume
    /// * `Err(TopologyError)` - If metric is missing or edges not found
    ///
    /// # Example
    /// ```rust,ignore
    /// let volume_sq = manifold.simplex_volume_squared(&simplex)?;
    /// ```
    pub fn simplex_volume_squared(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        // ...
    }
}
```

### CPU/MLX Implementation Methods (Minimal)

Internal methods need only brief description:

```rust
// geometry/geometry_cpu.rs
impl<T> Manifold<T> {
    /// CPU implementation of Cayley-Menger volume calculation.
    pub(crate) fn simplex_volume_squared_cpu(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        // ...
    }
}
```

### Doc Comment Checklist

| Location | Requirement |
|----------|-------------|
| `api/*.rs` public methods | Full `///` with Arguments, Returns, Example |
| `*_cpu.rs` methods | One-line `///` summary |
| `*_mlx.rs` methods | One-line `///` summary |
| Struct fields | Brief `///` or `//` inline comment |

---

## Test Organization

### Test File Structure

```
deep_causality_topology/
├── src/
│   └── types/manifold/...
└── tests/
    ├── types/
    │   ├── mod.rs
    │   └── manifold/
    │       ├── mod.rs
    │       ├── geometry_tests.rs
    │       ├── differential_tests.rs
    │       ├── covariance_tests.rs
    │       └── mlx_tests.rs          # Feature-gated MLX tests
    └── integration/
        └── manifold_integration_tests.rs
```

### Test Naming Convention

| Test Type | File Name | Test Function |
|-----------|-----------|---------------|
| Unit (CPU) | `geometry_tests.rs` | `test_simplex_volume_squared_cpu()` |
| Unit (MLX) | `mlx_tests.rs` | `test_simplex_volume_squared_mlx()` |
| Integration | `*_integration_tests.rs` | `test_manifold_full_workflow()` |

### Test File Template

```rust
// tests/types/manifold/geometry_tests.rs

use deep_causality_topology::{Manifold, Simplex, TopologyError};

#[test]
fn test_simplex_volume_squared_cpu_triangle() {
    // Arrange
    let manifold = create_test_manifold();
    let simplex = Simplex::new(vec![0, 1, 2]);
    
    // Act
    let result = manifold.simplex_volume_squared(&simplex);
    
    // Assert
    assert!(result.is_ok());
    let volume_sq = result.unwrap();
    assert!(volume_sq > 0.0);
}

#[test]
fn test_simplex_volume_squared_cpu_point() {
    let manifold = create_test_manifold();
    let simplex = Simplex::new(vec![0]);
    
    let result = manifold.simplex_volume_squared(&simplex);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1.0); // Point has volume 1 by convention
}
```

### MLX Test File Template

```rust
// tests/types/manifold/mlx_tests.rs

#![cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]

use deep_causality_topology::{Manifold, Simplex};

#[test]
fn test_covariance_matrix_mlx() {
    let manifold = create_large_test_manifold(10_000);
    
    let result = manifold.covariance_matrix();
    
    assert!(result.is_ok());
}

#[test]
fn test_eigen_covariance_mlx_matches_cpu() {
    let manifold = create_test_manifold(1000);
    
    // Force CPU path
    let cpu_result = manifold.eigen_covariance_cpu();
    
    // GPU path (transparent dispatch for large data)
    let gpu_result = manifold.eigen_covariance();
    
    // Results should be close (f32 precision)
    assert_eigenvalues_close(&cpu_result.unwrap(), &gpu_result.unwrap(), 1e-5);
}
```

### Running Tests

```bash
# CPU tests only
cargo test -p deep_causality_topology

# MLX tests (single-threaded for Metal)
cargo test -p deep_causality_topology --features mlx -- --test-threads=1

# Specific test module
cargo test -p deep_causality_topology types::manifold::geometry_tests
```

---

## File Size Guidelines

- **mod.rs**: ≤100 lines (struct def + re-exports only)
- **api/*.rs**: ≤200 lines per file (thin dispatch layer)
- **<op>/<op>_cpu.rs**: ≤500 lines per file (implementation details)
- **<op>/<op>_mlx.rs**: ≤300 lines per file (GPU wrappers)

---

## Verification Checklist

- [ ] All public methods in `api/` directory
- [ ] All CPU impl methods have `_cpu` suffix
- [ ] All MLX impl methods have `_mlx` suffix
- [ ] All impl methods are `pub(crate)`
- [ ] CPU and MLX implementations in same folder
- [ ] MLX stubs compile with `--features mlx`
- [ ] Existing tests pass
- [ ] No new public API changes
- [ ] All public API methods have full doc comments
- [ ] Test files mirror source structure

---

## Benefits

1. **MLX Ready** - Clear dispatch points for GPU acceleration
2. **Side-by-Side Comparison** - CPU and MLX implementations visually adjacent
3. **Consistency** - Same pattern as tensor/multivector crates
4. **Maintainability** - Smaller, focused files grouped by operation
5. **Future Backends** - Easy to add `_cuda.rs`, `_vulkan.rs`, etc.
6. **Discoverability** - Predictable file locations via conventions

