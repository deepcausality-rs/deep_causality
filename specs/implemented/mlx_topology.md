# MLX Acceleration for deep_causality_topology

## Overview

This specification defines how to leverage MLX GPU acceleration in the `deep_causality_topology` crate. The design uses **transparent internal dispatch** - a single public API that automatically uses GPU acceleration when the `mlx` feature is enabled, with no changes required to user code.

## Design Philosophy: Transparent Acceleration

**Key Principle:** Users interact with a single, stable API. The implementation internally dispatches to CPU or GPU based on:
1. Whether the `mlx` feature flag is enabled
2. Whether the operation benefits from GPU (heuristics like matrix size)

```rust
// User code - same regardless of feature flags
let volume = manifold.simplex_volume_squared(&simplex)?;
let eigenvalues = manifold.eigen_covariance()?;

// Build with CPU (default):  cargo build
// Build with GPU:            cargo build --features mlx
```

---

## Executive Summary

| Component | CPU (default) | With `--features mlx` | Expected Speedup |
|-----------|---------------|----------------------|------------------|
| Cayley-Menger Determinant | O(n!) Laplace | O(n³) QR on GPU | 10-100x |
| Covariance Matrix | O(n²) loops | O(n²) GPU matmul | 50-500x |
| Eigendecomposition | O(n³) CPU | O(n³) GPU | 100-1000x |
| Field Tensor Operations | O(n) loops | O(n) GPU parallel | 10-100x |
| Boundary Operator (∂) | O(nnz) sparse | O(nnz) sparse | 1x (no benefit) |

---

## Architecture

### Feature Flag

```toml
# deep_causality_topology/Cargo.toml
[features]
default = []
mlx = ["deep_causality_tensor/mlx"]

[dependencies]
deep_causality_tensor = { workspace = true }
```

### Module Structure

```
deep_causality_topology/src/
├── types/
│   ├── manifold/
│   │   ├── mod.rs
│   │   ├── geometry.rs           # PUBLIC API (dispatches internally)
│   │   ├── geometry_cpu.rs       # CPU implementation
│   │   ├── geometry_mlx.rs       # MLX implementation (cfg-gated)
│   │   ├── covariance.rs         # PUBLIC API (dispatches internally)
│   │   ├── covariance_cpu.rs     # CPU implementation
│   │   ├── covariance_mlx.rs     # MLX implementation (cfg-gated)
│   │   └── differential/
│   │       ├── laplacian.rs
│   │       ├── hodge.rs
│   │       └── exterior.rs
│   └── regge_geometry/
└── lib.rs
```

---

## Existing MlxCausalTensor API (from deep_causality_tensor)

The topology crate leverages the existing `MlxCausalTensor` from `deep_causality_tensor`. This section documents the available GPU operations.

### Import

```rust
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::{MlxCausalTensor, MlxCompatible};
```

### Constructors

| Method | Description |
|--------|-------------|
| `new_f32(data: Vec<f32>, shape: Vec<usize>)` | Direct f32, no conversion |
| `new_from_f64(data: &[f64], shape: Vec<usize>)` | **Downcasts f64 → f32** |
| `from_causal_tensor(tensor: &CausalTensor<f32>)` | Bridge from existing f32 tensor |
| `from_causal_tensor_f64(tensor: &CausalTensor<f64>)` | Bridge from f64 tensor (downcasts) |

### Converters

| Method | Description |
|--------|-------------|
| `to_causal_tensor()` | Converts back to `CausalTensor<f32>` |
| `as_mlx_array()` | Access underlying `mlx_rs::Array` |

### GPU-Accelerated Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `matmul` | `fn matmul(&self, other: &MlxCausalTensor) -> Result<Self>` | Matrix multiplication |
| `transpose` | `fn transpose(&self) -> Result<Self>` | Transpose last two dimensions |
| `inverse` | `fn inverse(&self) -> Result<Self>` | Matrix inverse (square only) |
| `ein_sum` | `fn ein_sum(&self, subscripts: &str, other: &MlxCausalTensor) -> Result<Self>` | Binary Einstein summation |
| `ein_sum_unary` | `fn ein_sum_unary(&self, subscripts: &str) -> Result<Self>` | Unary einsum (trace, transpose) |
| `add` | `fn add(&self, other: &MlxCausalTensor) -> Result<Self>` | Element-wise addition |
| `sub` | `fn sub(&self, other: &MlxCausalTensor) -> Result<Self>` | Element-wise subtraction |
| `mul` | `fn mul(&self, other: &MlxCausalTensor) -> Result<Self>` | Element-wise multiplication |
| `sum` | `fn sum(&self) -> Result<f32>` | Sum all elements |

### Example: GPU Covariance

```rust
use deep_causality_tensor::{CausalTensor, MlxCausalTensor};

fn covariance_mlx(data: &CausalTensor<f64>) -> Result<CausalTensor<f32>, TopologyError> {
    // 1. Bridge to MLX (f64 → f32 downcast)
    let mlx = MlxCausalTensor::from_causal_tensor_f64(data)?;
    
    // 2. Compute outer product on GPU: X @ X^T
    let transposed = mlx.transpose()?;
    let cov = mlx.matmul(&transposed)?;
    
    // 3. Bridge back to CPU
    cov.to_causal_tensor()
        .map_err(|e| TopologyError::TensorError(e.to_string()))
}
```

### Example: GPU Determinant via Einsum

```rust
fn trace_mlx(matrix: &CausalTensor<f64>) -> Result<f64, TopologyError> {
    let mlx = MlxCausalTensor::from_causal_tensor_f64(matrix)?;
    
    // Trace: sum of diagonal elements using einsum
    let result = mlx.ein_sum_unary("ii->")?;
    let trace = result.sum()?;
    
    Ok(trace as f64)
}
```

## Transparent Dispatch Pattern

### Pattern 1: Compile-Time Dispatch with cfg

For operations where GPU is always beneficial when available:

```rust
// src/types/manifold/geometry.rs - PUBLIC API

impl<T> Manifold<T> {
    /// Computes the squared volume of a k-simplex.
    /// 
    /// Automatically uses GPU acceleration when `mlx` feature is enabled.
    pub fn simplex_volume_squared(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        {
            self.simplex_volume_squared_mlx(simplex)
        }
        
        #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
        {
            self.simplex_volume_squared_cpu(simplex)
        }
    }
}
```

### Pattern 2: Runtime Dispatch with Size Heuristics

For operations where GPU overhead may exceed benefit for small inputs:

```rust
// src/types/manifold/geometry.rs - PUBLIC API

/// Threshold below which CPU is faster due to GPU transfer overhead
const GPU_DETERMINANT_THRESHOLD: usize = 6;

impl<T> Manifold<T> {
    /// Computes the determinant of a square matrix.
    /// 
    /// - For n < 6: Always uses CPU (transfer overhead dominates)
    /// - For n ≥ 6 with `mlx` feature: Uses GPU
    /// - Without `mlx` feature: Always uses CPU
    pub fn determinant(matrix: &CausalTensor<f64>) -> Result<f64, TopologyError> {
        let n = matrix.shape()[0];
        
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        {
            if n >= GPU_DETERMINANT_THRESHOLD {
                return Self::determinant_mlx(matrix);
            }
        }
        
        Self::determinant_cpu(matrix)
    }
}
```

### Pattern 3: Trait-Based Dispatch (Alternative)

For more flexible runtime control:

```rust
// src/types/manifold/covariance.rs

/// Backend for covariance computations
pub enum ComputeBackend {
    Cpu,
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    Gpu,
}

impl Default for ComputeBackend {
    fn default() -> Self {
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        { ComputeBackend::Gpu }
        
        #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
        { ComputeBackend::Cpu }
    }
}

impl<T> Manifold<T> {
    /// Computes covariance matrix with default backend (GPU if available).
    pub fn covariance_matrix(&self) -> Result<CausalTensor<f64>, TopologyError> {
        self.covariance_matrix_with_backend(ComputeBackend::default())
    }
    
    /// Computes covariance matrix with explicit backend selection.
    pub fn covariance_matrix_with_backend(
        &self, 
        backend: ComputeBackend
    ) -> Result<CausalTensor<f64>, TopologyError> {
        match backend {
            ComputeBackend::Cpu => self.covariance_matrix_cpu(),
            #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
            ComputeBackend::Gpu => {
                let mlx_result = self.covariance_matrix_mlx()?;
                // Convert MlxCausalTensor back to CausalTensor<f64>
                Self::mlx_to_f64_tensor(mlx_result)
            }
        }
    }
}
```

---

## Public API (Unchanged Interface)

### Geometry Operations

```rust
impl<T> Manifold<T> {
    /// Computes the squared volume of a k-simplex using Cayley-Menger determinant.
    /// GPU-accelerated when `mlx` feature is enabled and k ≥ 4.
    pub fn simplex_volume_squared(&self, simplex: &Simplex) -> Result<f64, TopologyError>;
    
    /// Computes the determinant of a square matrix.
    /// GPU-accelerated when `mlx` feature is enabled and n ≥ 6.
    pub fn determinant(matrix: &CausalTensor<f64>) -> Result<f64, TopologyError>;
    
    /// Computes the inverse of a metric tensor.
    /// GPU-accelerated when `mlx` feature is enabled.
    pub fn inverse_metric(&self, gram: &CausalTensor<f64>) -> Result<CausalTensor<f64>, TopologyError>;
}
```

### Covariance Analysis

```rust
impl<T> Manifold<T>
where
    T: Into<f64> + Copy,
{
    /// Constructs the covariance matrix from manifold field data.
    /// GPU-accelerated when `mlx` feature is enabled.
    pub fn covariance_matrix(&self) -> Result<CausalTensor<f64>, TopologyError>;
    
    /// Computes eigenvalues of the covariance matrix.
    /// GPU-accelerated when `mlx` feature is enabled.
    pub fn eigen_covariance(&self) -> Result<Vec<f64>, TopologyError>;
    
    /// Computes both eigenvalues and eigenvectors of covariance.
    /// GPU-accelerated when `mlx` feature is enabled.
    pub fn eigen_covariance_full(&self) -> Result<(Vec<f64>, CausalTensor<f64>), TopologyError>;
}
```

### Field Operations

```rust
impl<T> Manifold<T>
where
    T: Into<f64> + From<f64> + Copy,
{
    /// Scales the manifold field by a scalar.
    /// GPU-accelerated for large fields when `mlx` feature is enabled.
    pub fn scale_field(&self, scalar: f64) -> Result<Manifold<T>, TopologyError>;
    
    /// Adds two manifold fields element-wise.
    /// GPU-accelerated for large fields when `mlx` feature is enabled.
    pub fn add_fields(&self, other: &Manifold<T>) -> Result<Manifold<T>, TopologyError>;
}
```

---

## Internal Implementation

### CPU Implementation (Always Available)

```rust
// src/types/manifold/geometry_cpu.rs

impl<T> Manifold<T> {
    /// CPU implementation of determinant using Laplace expansion.
    pub(crate) fn determinant_cpu(matrix: &CausalTensor<f64>) -> Result<f64, TopologyError> {
        // Existing O(n!) implementation
        // ... (current geometry.rs code)
    }
    
    pub(crate) fn simplex_volume_squared_cpu(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        // Existing implementation
    }
}
```

### MLX Implementation (Feature-Gated)

```rust
// src/types/manifold/geometry_mlx.rs

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::MlxCausalTensor;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
impl<T> Manifold<T> {
    /// MLX implementation using QR decomposition.
    pub(crate) fn determinant_mlx(matrix: &CausalTensor<f64>) -> Result<f64, TopologyError> {
        let mlx_matrix = MlxCausalTensor::from_causal_tensor_f64(matrix)
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        
        // Use QR decomposition: det(A) = ±prod(diag(R))
        let (_, r) = mlx_rs::linalg::qr(&mlx_matrix.as_mlx_array())
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        
        let diag = mlx_rs::ops::diagonal(&r, 0)
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        let det = mlx_rs::ops::prod(&diag, None)
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        det.eval().map_err(|_| TopologyError::TensorError("MLX eval failed".into()))?;
        
        Ok(det.as_slice::<f32>()[0] as f64)
    }
    
    pub(crate) fn simplex_volume_squared_mlx(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        let cm_matrix = self.build_cayley_menger_matrix(simplex)?;
        let det = Self::determinant_mlx(&cm_matrix)?;
        
        let k = simplex.vertices.len() - 1;
        let k_fac = (1..=k).map(|i| i as f64).product::<f64>();
        let denominator = 2.0_f64.powi(k as i32) * k_fac.powi(2);
        let sign = if k % 2 == 0 { -1.0 } else { 1.0 };
        
        Ok(((sign / denominator) * det).max(0.0))
    }
}
```

### Covariance with Transparent Dispatch

```rust
// src/types/manifold/covariance.rs - PUBLIC API

const GPU_COVARIANCE_THRESHOLD: usize = 1000;

impl<T> Manifold<T>
where
    T: Into<f64> + Copy + Clone,
{
    /// Constructs the covariance matrix from manifold field data.
    pub fn covariance_matrix(&self) -> Result<CausalTensor<f64>, TopologyError> {
        let n = self.data.len();
        
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        {
            if n >= GPU_COVARIANCE_THRESHOLD {
                let mlx_result = self.covariance_matrix_mlx()?;
                return Self::mlx_to_f64_tensor(mlx_result);
            }
        }
        
        self.covariance_matrix_cpu()
    }
    
    /// Computes eigenvalues of the covariance matrix.
    pub fn eigen_covariance(&self) -> Result<Vec<f64>, TopologyError> {
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        {
            if self.data.len() >= GPU_COVARIANCE_THRESHOLD {
                return self.eigen_covariance_mlx();
            }
        }
        
        self.eigen_covariance_cpu()
    }
}

// src/types/manifold/covariance_cpu.rs
impl<T> Manifold<T>
where
    T: Into<f64> + Copy,
{
    pub(crate) fn covariance_matrix_cpu(&self) -> Result<CausalTensor<f64>, TopologyError> {
        let data: Vec<f64> = self.data.as_slice().iter().map(|&x| x.into()).collect();
        let n = data.len();
        
        // Compute mean
        let mean: f64 = data.iter().sum::<f64>() / n as f64;
        
        // Center and compute outer product
        let centered: Vec<f64> = data.iter().map(|&x| x - mean).collect();
        
        let mut cov = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                cov[i * n + j] = centered[i] * centered[j] / (n - 1) as f64;
            }
        }
        
        CausalTensor::new(cov, vec![n, n])
            .map_err(|e| TopologyError::TensorError(e.to_string()))
    }
    
    pub(crate) fn eigen_covariance_cpu(&self) -> Result<Vec<f64>, TopologyError> {
        let cov = self.covariance_matrix_cpu()?;
        // Use existing CPU eigendecomposition
        Self::eigenvalues_cpu(&cov)
    }
}

// src/types/manifold/covariance_mlx.rs
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
impl<T> Manifold<T>
where
    T: Into<f64> + Copy,
{
    pub(crate) fn covariance_matrix_mlx(&self) -> Result<MlxCausalTensor, TopologyError> {
        let data_f64: Vec<f64> = self.data.as_slice().iter().map(|&x| x.into()).collect();
        let n = data_f64.len();
        
        let mlx_data = MlxCausalTensor::new_from_f64(&data_f64, vec![n, 1])
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        
        // GPU matmul for outer product
        let transposed = mlx_data.transpose()
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        
        mlx_data.matmul(&transposed)
            .map_err(|e| TopologyError::TensorError(e.to_string()))
    }
    
    pub(crate) fn eigen_covariance_mlx(&self) -> Result<Vec<f64>, TopologyError> {
        let cov = self.covariance_matrix_mlx()?;
        
        let (eigenvalues, _) = mlx_rs::linalg::eigh(&cov.as_mlx_array())
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        eigenvalues.eval()
            .map_err(|_| TopologyError::TensorError("MLX eval failed".into()))?;
        
        Ok(eigenvalues.as_slice::<f32>().iter().map(|&x| x as f64).collect())
    }
    
    /// Helper to convert MlxCausalTensor back to CausalTensor<f64>
    pub(crate) fn mlx_to_f64_tensor(mlx: MlxCausalTensor) -> Result<CausalTensor<f64>, TopologyError> {
        let f32_tensor = mlx.to_causal_tensor()
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        let f64_data: Vec<f64> = f32_tensor.data().iter().map(|&x| x as f64).collect();
        CausalTensor::new(f64_data, f32_tensor.shape().to_vec())
            .map_err(|e| TopologyError::TensorError(e.to_string()))
    }
}
```

---

## Size Thresholds

| Operation | CPU Only | GPU (when mlx enabled) |
|-----------|----------|------------------------|
| Determinant | n < 6 | n ≥ 6 |
| Simplex Volume | k < 4 | k ≥ 4 |
| Covariance | n < 1000 | n ≥ 1000 |
| Field Scale/Add | n < 10000 | n ≥ 10000 |

These thresholds are based on GPU transfer overhead (~0.5ms) vs compute time.

---

## Limitations

### Operations That Cannot Be Accelerated

| Operation | Reason | Behavior |
|-----------|--------|----------|
| Boundary operator (∂) | Sparse × vector | Always CPU |
| Coboundary operator (∂ᵀ) | Sparse × vector | Always CPU |
| Hodge star (⋆) | Sparse diagonal | Always CPU |
| Laplacian (Δ) | Chains sparse ops | Always CPU |

### Precision Considerations

| Operation | f64 → f32 Impact | Recommendation |
|-----------|------------------|----------------|
| Cayley-Menger det | Low (relative) | Use GPU |
| Covariance eigenvalues | Low (ratios) | Use GPU |
| Clock drift (10⁻¹⁵) | **HIGH** | Force CPU via threshold |
| Volume sqrt | Medium | Post-process in f64 |

---

## Usage Examples

### Default Usage (Transparent)

```rust
use deep_causality_topology::{Manifold, Simplex};

fn compute_geometry(manifold: &Manifold<f64>, simplex: &Simplex) -> f64 {
    // Automatically uses GPU when:
    // 1. `mlx` feature is enabled
    // 2. Simplex dimension k ≥ 4
    manifold.simplex_volume_squared(simplex).unwrap()
}

fn analyze_covariance(manifold: &Manifold<f64>) -> Vec<f64> {
    // Automatically uses GPU when:
    // 1. `mlx` feature is enabled
    // 2. Field size n ≥ 1000
    manifold.eigen_covariance().unwrap()
}
```

### Force CPU for Precision-Critical Work

```rust
use deep_causality_topology::{Manifold, ComputeBackend};

fn precision_critical(manifold: &Manifold<f64>) -> Vec<f64> {
    // Explicit CPU backend for f64 precision
    manifold.eigen_covariance_with_backend(ComputeBackend::Cpu).unwrap()
}
```

---

## Verification Plan

### Unit Tests

```bash
# Test CPU path
cargo test -p deep_causality_topology

# Test GPU path (single thread required for Metal)
cargo test -p deep_causality_topology --features mlx -- --test-threads=1
```

### Benchmark CPU vs GPU

```bash
cargo bench -p deep_causality_topology --features mlx -- covariance
```

### Integration with GQCD

```bash
# Default: uses GPU automatically
RUSTFLAGS='-C target-cpu=native' cargo run --release \
  -p gqcd_chrono_manifold --features mlx
```

---

## Implementation Priority

| Phase | Component | API Change | Effort |
|-------|-----------|------------|--------|
| 1 | Transparent `determinant()` | None | Low |
| 2 | Transparent `covariance_matrix()` | None | Medium |
| 3 | Transparent `eigen_covariance()` | None | Medium |
| 4 | Transparent field ops | None | Low |
| 5 | Optional `ComputeBackend` | Additive | Low |

**Recommendation:** Start with Phase 2-3 (Covariance) for highest physics simulation impact.
