# Generic Backend: Multivector & MultiField Specification

## 1. Overview

The `deep_causality_multivector` crate integrates the standard Geometric Algebra types with the Generic Backend
architecture.

This specification defines two distinct handling strategies:

1. **Single Multivectors (`CausalMultiVector<T>`):** Kept as lightweight coeff-arrays (CPU-native) for efficiency.
2. **Fields of Multivectors (`CausalMultiField<B, T>`):** Backend-accelerated tensors using the Matrix Isomorphism to
   map Clifford Algebra operations to backend `matmul` operations.

---

## 2. Architecture

### 2.1 Type Definitions

#### A. Single Multivector (Standard + Accelerated)

The `CausalMultiVector` remains a hybrid struct. It stores coefficients on the CPU for low-overhead access but can
dispatch heavy arithmetic to the Generic Backend.

```rust
// deep_causality_multivector/src/types/multivector/mod.rs
pub struct CausalMultiVector<T> {
    pub data: Vec<T>, // CPU Coefficients
    pub metric: Metric,
}
```

### 2.2 Acceleration Strategy (Single Multivector)

For high-dimensional algebras (Spin(10), Cl(10)), O(4^N) CPU operations are too slow. We dispatch to the Generic Backend
using the **Matrix Isomorphism**.

**Thresholds:**

* `DIMENSION_THRESHOLD >= 5` (e.g., N=5, 32x32 matrices) -> Use Backend
* `BATCH_SIZE >= 1000` (for fields) -> Use Backend

**Dispatch Flow:**

```rust
impl<T> CausalMultiVector<T> {
    pub fn geometric_product<B: LinearAlgebraBackend>(&self, rhs: &Self) -> Self {
        if self.metric.dimension() >= 5 {
            // 1. Upload self/rhs to Backend (as Matrix)
            // 2. Perform B::matmul
            // 3. Download result to CPU Coeffs
            return self.geometric_product_accelerated::<B>(rhs);
        }
        self.geometric_product_cpu(rhs) // O(4^N) Fallback
    }
}
```

### 2.3 Performance Analysis

**Actual Benchmark Results (Apple Silicon M3 Max):**

Based on measured `CausalTensor` benchmarks comparing CPU vs MLX GPU:

| Matrix Size | CPU Time  | MLX Time | Measured Speedup |
|-------------|-----------|----------|------------------|
| 128×128     | 1.49 ms   | 0.17 ms  | **8.5x**         |
| 512×512     | 139.6 ms  | 0.22 ms  | **635x**         |
| 1024×1024   | 1,087 ms  | 0.42 ms  | **2,588x**       |

**Mapping to Clifford Algebras:**

| Algebra    | N  | Matrix Size | Projected Speedup |
|------------|----|-------------|-------------------|
| Cl(2,0)    | 2  | 4×4         | ~1x (GPU overhead dominates) |
| Cl(3,0)    | 3  | 8×8         | ~2x               |
| Cl(1,3)    | 4  | 16×16       | ~5x               |
| Cl(4,1)    | 5  | 32×32       | ~8x               |
| Cl(6,0)    | 6  | 64×64       | ~15x              |
| **Cl(10)** | 10 | 1024×1024   | **>2,500x**       |

> **Key Insight:** GPU overhead is ~160 µs baseline. For small matrices (<32×32), CPU is faster.
> For Cl(10) scale physics (1024×1024), MLX delivers >2,500x speedup over CPU.

**Threshold Constants (Updated from Benchmarks):**

```rust
/// Minimum dimension for single-operation acceleration.
/// GPU crossover occurs around 32×32 (N=5).
pub const DIMENSION_THRESHOLD: usize = 5;

/// Minimum batch size for acceleration at dimension >= 3.
/// BatchMatMul: 8×64×64 gives 17x speedup, 8×128×128 gives 100x.
pub const BATCH_THRESHOLD: usize = 8;

/// Minimum field size (cells) for field-level acceleration.
/// ~64³ grid = 262k cells maps to 512×512 matrices → 635x speedup.
pub const FIELD_SIZE_THRESHOLD: usize = 262_144;

fn should_use_backend(dim: usize, batch_size: usize) -> bool {
    dim >= DIMENSION_THRESHOLD ||
        (dim >= 3 && batch_size >= BATCH_THRESHOLD)
}
```

### 2.4 Gamma Matrix Storage Strategy

| Algebra    | Blades | Matrix Size | Total Size | Storage Strategy |
|------------|--------|-------------|------------|------------------|
| Cl(2,0)    | 4      | 2×2         | 64 B       | `const` array    |
| Cl(1,3)    | 16     | 4×4         | 2 KB       | `const` array    |
| Cl(4,1)    | 32     | 4×4         | 4 KB       | `const` array    |
| Cl(6,0)    | 64     | 8×8         | 32 KB      | `static` lazy    |
| **Cl(10)** | 1024   | 32×32       | **2 GB**   | **lazy + cache** |

---

## 3. The Matrix Isomorphism Bridge (Generic)

To utilize the generic backend (which knows only Tensors and Matmul), we implement the generic `MatrixRep` trait.

### 3.1 Gamma Matrices via Backend

We need a generic way to load Gamma matrices (the basis vectors of the algebra) onto any device `B`.

```rust
pub trait BackendGamma<B: TensorBackend, T: TensorData> {
    /// Returns the pre-loaded Gamma matrices for a given metric
    /// Shape: [Num_Blades, Matrix_Dim, Matrix_Dim]
    /// Storage: Lazy initialization (static/cached on device)
    fn get_gammas(metric: &Metric) -> &B::Tensor<T>;
}
```

**Implementation Strategy:**

* **CPU:** Const arrays for small dims, lazy `Vec` for large.
* **MLX/CUDA:** Upload once to GPU memory, return reference to `B::Tensor`.

### 3.2 MatrixRep Trait

```rust
pub trait MatrixRep<B: LinearAlgebraBackend, T: TensorData> {
    /// Transforms coefficients to Matrix Representation on Device
    /// Op: Tensor Contraction (Coeffs * Gammas)
    fn to_matrix(&self) -> B::Tensor<T>;

    /// Transforms Matrix Representation back to coefficients
    /// Op: Projection / Trace
    fn from_matrix(matrix: B::Tensor<T>, metric: Metric) -> Self;
}
```

#### B. MultiField (Accelerated)

The field type wraps the generic `CausalTensor` in **Matrix Representation**.

```rust
// deep_causality_multivector/src/types/multifield/mod.rs
use deep_causality_tensor::CausalTensor;

pub struct CausalMultiField<B: TensorBackend, T: TensorData> {
    /// Storage: [Batch, X, Y, Z, Matrix_Dim, Matrix_Dim]
    /// Stored in Matrix Isomorphism representation
    pub data: CausalTensor<B, T>,

    pub metric: Metric,
    pub dx: Vec<T>, // Grid spacing (generic T for precision match)
}
```

---

## 3. The Matrix Isomorphism Bridge

To utilize the generic backend (which knows only Tensors and Matmul), we implement the **Matrix Isomorphism**
generically.

### 3.1 Gamma Matrices via Backend

The constant gamma matrices (basis vectors) must be loaded onto the backend device.

```rust
trait BackendGamma<B: TensorBackend, T: TensorData> {
    /// Returns the pre-loaded Gamma matrices for a given metric
    /// Shape: [Num_Blades, Matrix_Dim, Matrix_Dim]
    fn get_gammas(metric: &Metric) -> CausalTensor<B, T>;
}
```

### 3.2 Conversion: Coeffs $\leftrightarrow$ Matrix

* **To Matrix:** `field_coeffs (contract) gammas = field_matrix`
    * Tensordot / Contraction operation provided by backend.
* **From Matrix:** `matrix_field (project) gammas = field_coeffs`
    * Projection via trace/inner-product.

---

## 4. Operations Implementation

### 4.1 Field Geometric Product

`A ⊙ B` maps directly to matrix multiplication on the generic backend.

```rust
impl<B: TensorBackend, T: TensorData> CausalMultiField<B, T> {
    pub fn geometric_product(&self, rhs: &Self) -> Self {
        // Backend::matmul handles [Batch, ..., M, M] broadcasting
        let result_data = B::matmul(&self.data.data, &rhs.data.data);

        Self {
            data: CausalTensor { data: result_data, ... },
            metric: self.metric,
            dx: self.dx.clone(),
        }
    }
}
```

### 4.2 Differential Operators (Curl, Div)

These rely on the backend's `gather` or `convolution` primitives (if available via extensions) or explicit stencil
arithmetic.

```rust
// Generic Stencil Implementation
fn apply_stencil<B: TensorBackend, T: TensorData>(
    field: &CausalTensor<B, T>,
    axis: usize
) -> CausalTensor<B, T> {
    // 1. Shift Left (slice/padding)
    // 2. Shift Right
    // 3. (Right - Left) / (2 * dx)
    // All using B::slice, B::sub, B::div
}
```

---

## 5. Backend-Specific Behaviors

### 5.1 CPU Backend (`f64` / `Complex64`)

* **Use Case:** Verification of MHD conservation laws.
* **Precision:** Standard `f64`.
* **Matrix Rep:** Uses `ndarray` broadcasting.

### 5.2 MLX Backend (`f32`)

* **Use Case:** High-speed MHD simulation (Hero Run).
* **Precision:** `f32` (Metal GPU constraint—no f64 support).
* **Constraint:** The user must explicitly instantiate `CausalMultiField<MlxBackend, f32>`.
* **Warning:** Initial data provided as `f64` will be downcast (8 µs for 128×128, 53 µs for 1024×1024).

**Measured Performance (M3 Max):**

| Operation      | CPU Time  | MLX Time | Speedup      |
|----------------|-----------|----------|--------------|
| MatMul 1024²   | 1,087 ms  | 0.42 ms  | **2,588x**   |
| BatchMatMul 8× | 18.14 ms  | 0.18 ms  | **100x**     |
| Reduction 1024²| 4.68 ms   | 0.18 ms  | **26x**      |

> **GPU Overhead**: ~160 µs baseline. Use CPU for matrices < 32×32.

### 5.3 CUDA Backend (Future)

* **Use Case:** Large-scale grid deployment.
* **Precision:** `f32` or `f64`.
* **Advantage:** `cuTensor` optimized contractions for the Gamma projection steps.

---

## 6. Migration Plan

1. **Refactor `CausalMultiVector`**: Keep mostly as-is but enable `Into<CausalMultiField<B, T>>`.
2. **Define `CausalMultiField`**: Create the new struct in `types/multifield`.
3. **Implement Gamma Loader**: Write the generic gamma matrix loader that uploads constants to `B`.
4. **Implement Ops**: Port the logic from `multifield_mlx.md` to use `B::matmul` instead of raw `mlx` calls.

---

## 7. Example: MHD Solver Construction

```rust
// User Code

// 1. Select Backend
type MyBackend = deep_causality_tensor::backend::MlxBackend;
type Dtype = f32;

// 2. Initialize Field
let v_field = CausalMultiField::<MyBackend, Dtype>::from_coefficients(
initial_data, // Vec<CausalMultiVector>
grid_shape,
metric
);

// 3. Run Physics (Generic Code!)
let curl_v = v_field.curl(); // Uses MyBackend::slice/matmul
let v_cross_B = v_field.cross( & B_field); // Uses MyBackend::matmul
```

---

## 8. Default Type Aliases (Feature-Flag Based)

```rust
// deep_causality_multivector/src/aliases.rs

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultBackend = MlxBackend;

#[cfg(not(feature = "mlx"))]
pub type DefaultBackend = CpuBackend;

/// Convenience Alias for CausalMultiVector (always CPU, no backend)
pub type MultiVector<T = f64> = CausalMultiVector<T>;

/// Convenience Alias for CausalMultiField with auto-selected backend
pub type MultiField<T = f32> = CausalMultiField<DefaultBackend, T>;
```

---

## 9. Testing Strategy

### 9.1 Test Structure

```
deep_causality_multivector/tests/
├── types/
│   ├── multivector/
│   │   ├── geometric_product_tests.rs
│   │   └── matrix_rep_tests.rs
│   └── multifield/
│       ├── geometric_product_tests.rs
│       └── differential_tests.rs
└── backend_parity/
    └── cpu_mlx_tests.rs  # [feature-gated]
```

### 9.2 Test Categories

| Category               | Tests                                 |
|------------------------|---------------------------------------|
| **Algebraic Laws**     | (AB)C = A(BC), A·1 = A                |
| **Matrix Isomorphism** | to_matrix → from_matrix roundtrip     |
| **Gamma Matrix**       | Clifford relation: ΓᵢΓⱼ + ΓⱼΓᵢ = 2ηᵢⱼ |
| **Backend Parity**     | CPU == MLX within 1e-5                |

### 9.3 Example Tests

```rust
#[test]
fn matrix_rep_roundtrip() {
    let mv = CausalMultiVector::<f64>::random(Metric::Minkowski(4));
    let matrix = mv.to_matrix::<CpuBackend>();
    let recovered = CausalMultiVector::from_matrix::<CpuBackend>(matrix, mv.metric);

    for (a, b) in mv.coefficients().zip(recovered.coefficients()) {
        assert!((a - b).abs() < 1e-14);
    }
}

#[test]
fn clifford_relation() {
    let gammas = BackendGamma::<CpuBackend, f64>::get_gammas(&Metric::Minkowski(4));
    // Check: Γ₀² = +1, Γ₁² = -1, Γ₂² = -1, Γ₃² = -1
    // And anticommutation for i ≠ j
}
```

---

## 10. RustDoc Guidelines

```rust
/// Converts multivector coefficients to matrix representation.
///
/// This trait enables the Matrix Isomorphism acceleration strategy,
/// mapping Clifford algebra operations to matrix multiplication.
///
/// # Type Parameters
///
/// * `B` - The backend where the matrix will be stored.
/// * `T` - The scalar type of the coefficients.
///
/// # Example
///
/// ```rust
/// let mv = CausalMultiVector::<f64>::new(...);
/// let matrix = mv.to_matrix::<CpuBackend>();
/// ```
pub trait MatrixRep<B: LinearAlgebraBackend, T: TensorData> { ... }
```
