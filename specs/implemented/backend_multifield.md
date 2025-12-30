# Generic Backend: CausalMultiField Specification

## 1. Overview

`CausalMultiField<B, T>` is a **hardware-accelerated field type** representing a spatial grid of multivectors. It
provides the semantic interface of `CausalTensor<CausalMultiVector<T>>` but with a flat, GPU-optimized memory layout
using the **Matrix Isomorphism**.

While `CausalTensor<CausalMultiVector<T>>` works on CPU via HAFT trait composition, it is computationally inefficient
for large grids due to its Vec-of-structs layout. `CausalMultiField<B, T>` addresses this by:

1. Storing data in Matrix Representation directly on the backend device.
2. Mapping all Clifford operations to `LinearAlgebraBackend::matmul`.
3. Enabling precision selection (`f64` on CPU, `f32` on MLX/CUDA) via Generic Backend.

---

## 2. Architecture

### 2.1 Type Definition

```rust
// deep_causality_multivector/src/types/multifield/mod.rs
use deep_causality_tensor::{CausalTensor, TensorBackend, LinearAlgebraBackend, TensorData};
use deep_causality_metric::Metric;

pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> {
    /// Storage: [Batch, X, Y, Z, Matrix_Dim, Matrix_Dim]
    /// Stored entirely in Matrix Isomorphism representation on Device
    pub(crate) data: B::Tensor<T>,

    /// The metric signature of the underlying Clifford algebra
    pub metric: Metric,

    /// Grid spacing for differential operators [dx, dy, dz]
    /// Stored as T for precision consistency
    pub dx: [T; 3],

    /// Grid shape [Nx, Ny, Nz]
    pub shape: [usize; 3],
}
```

### 2.2 Memory Layout (Matrix View)

The field stays in **Matrix Representation** on the device to avoid costly Coeff↔Matrix conversions.

**Tensor Shape:** `[Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]`

* **Grid Dimensions:** `Nx, Ny, Nz` (Spatial)
* **Algebra Dimensions:** `Matrix_Dim × Matrix_Dim` (per-cell multivector as matrix)

**Examples:**

| Algebra | Metric    | Matrix Size | Field Shape (128³ grid) |
|---------|-----------|-------------|-------------------------|
| Cl(3,0) | MHD       | 2×2 (ℂ)     | `[128, 128, 128, 2, 2]` |
| Cl(1,3) | GR        | 4×4 (ℝ)     | `[128, 128, 128, 4, 4]` |
| Cl(4,1) | Conformal | 4×4 (ℂ)     | `[64, 64, 64, 4, 4]`    |

---

## 3. Trait Composition (HAFT)

`CausalMultiField` implements algebraic traits for generic physics code compatibility.

```rust
use deep_causality_num::{Ring, Module};

impl<B: LinearAlgebraBackend, T: TensorData> Ring for CausalMultiField<B, T> {
    // Add, Sub via B::add, B::sub
    // Mul via B::matmul (geometric product)
}

impl<B: LinearAlgebraBackend, T: TensorData> Module<T> for CausalMultiField<B, T> {
    // Scalar multiplication via B::mul with broadcast
}
```

---

## 4. Operations

All operations dispatch to the Generic Backend. No MLX-specific code in the public API.

### 4.1 Algebraic Products

```rust
impl<B: LinearAlgebraBackend, T: TensorData> CausalMultiField<B, T> {
    /// Field Geometric Product: (A ⊙ B)[x,y,z] = A[x,y,z] ⊙ B[x,y,z]
    /// Maps to batched matrix multiplication
    pub fn geometric_product(&self, rhs: &Self) -> Self {
        let result = B::matmul(&self.data, &rhs.data);
        Self { data: result, metric: self.metric, dx: self.dx, shape: self.shape }
    }

    /// Cross product via Hodge dual: A × B = -I(A ∧ B)
    pub fn cross(&self, rhs: &Self) -> Self {
        let ab = self.geometric_product(rhs);
        let ba = rhs.geometric_product(self);
        let wedge = B::sub(&ab.data, &ba.data); // Antisymmetric part
        // Scale by 0.5 and apply Hodge dual (grade projection)
        self.hodge_dual(&wedge)
    }

    /// Inner product: A · B = ⟨AB⟩₀
    pub fn inner(&self, rhs: &Self) -> Self {
        let product = self.geometric_product(rhs);
        product.grade_project(0) // Scalar part
    }
}
```

### 4.2 Differential Operators

Curl, Divergence, and Gradient use finite-difference stencils via generic backend slicing.

```rust
impl<B: LinearAlgebraBackend, T: TensorData> CausalMultiField<B, T> {
    /// Curl: ∇ × F (returns bivector field in GA)
    pub fn curl(&self) -> Self {
        let dx = self.partial_derivative(Axis::X);
        let dy = self.partial_derivative(Axis::Y);
        let dz = self.partial_derivative(Axis::Z);
        self.construct_curl(&dx, &dy, &dz)
    }

    /// Central difference: ∂F/∂x ≈ (F[x+1] - F[x-1]) / (2dx)
    fn partial_derivative(&self, axis: Axis) -> B::Tensor<T> {
        let shape = &self.shape;
        // Use B::slice to get shifted views, then B::sub and B::div
        let left = B::slice(&self.data, shift_left(axis));
        let right = B::slice(&self.data, shift_right(axis));
        let diff = B::sub(&right, &left);
        B::div(&diff, &B::create(&[self.dx[axis.index()] * T::from(2.0)], &[1]))
    }
}
```

### 4.3 Grade Projection

```rust
impl<B: LinearAlgebraBackend, T: TensorData> CausalMultiField<B, T> {
    /// Extract grade-k component: ⟨A⟩ₖ
    /// Uses matrix trace/projection based on the algebra
    pub fn grade_project(&self, k: usize) -> Self {
        let gammas = BackendGamma::<B, T>::get_gammas(&self.metric);
        // Project via contraction with grade-k gamma basis
        let projected = self.project_to_grade(&gammas, k);
        Self { data: projected, metric: self.metric, dx: self.dx, shape: self.shape }
    }

    pub fn scalar_part(&self) -> Self { self.grade_project(0) }
    pub fn vector_part(&self) -> Self { self.grade_project(1) }
    pub fn bivector_part(&self) -> Self { self.grade_project(2) }
}
```

---

## 5. Conversions (Tensor Interop)

```rust
impl<B: LinearAlgebraBackend, T: TensorData> CausalMultiField<B, T> {
    /// Upload from CPU CausalMultiVector collection
    pub fn from_coefficients(
        mvs: &[CausalMultiVector<T>],
        shape: [usize; 3],
        dx: [T; 3],
    ) -> Self {
        // 1. Extract coefficient vectors
        // 2. Contract with Gamma matrices to get Matrix form
        // 3. Upload to Backend via B::create()
    }

    /// Download to CPU CausalMultiVector collection
    pub fn to_coefficients(&self) -> Vec<CausalMultiVector<T>> {
        // 1. Download from Backend via B::to_vec()
        // 2. Project matrix data back to coefficient form
        // 3. Construct CausalMultiVector instances
    }
}

/// From CausalTensor<CausalMultiVector> (convenience)
impl<B: LinearAlgebraBackend, T: TensorData>
From<&CausalTensor<CpuBackend, CausalMultiVector<T>>> for CausalMultiField<B, T>
{
    fn from(tensor: &CausalTensor<CpuBackend, CausalMultiVector<T>>) -> Self {
        let mvs: Vec<_> = tensor.iter().cloned().collect();
        let shape = [tensor.shape()[0], tensor.shape()[1], tensor.shape()[2]];
        Self::from_coefficients(&mvs, shape, [T::one(); 3]) // Default dx
    }
}
```

---

## 6. Backend-Specific Behaviors

### 6.1 CpuBackend (`f64` / `Complex64`)

* **Precision:** Full `f64` for verification.
* **Use Case:** Energy conservation checks, correctness validation.
* **Backing:** `ndarray::ArrayD<T>`.

### 6.2 MlxBackend (`f32`)

* **Precision:** `f32` (GPU native).
* **Use Case:** High-speed MHD simulation runs.
* **Warning:** Data from `CausalMultiVector<f64>` will be downcast.

### 6.3 CudaBackend (Future)

* **Precision:** `f32` or `f64`.
* **Use Case:** Cluster-scale deployment.
* **Optimization:** `cuTensor` for Gamma contractions.

---

## 7. File Structure

### 7.1 Backend Definitions (in `deep_causality_tensor`)

The core backend traits and implementations live in the tensor crate:

```
deep_causality_tensor/src/backend/
├── mod.rs              # TensorBackend, LinearAlgebraBackend traits
├── cpu.rs              # CpuBackend implementation (ndarray)
├── mlx.rs              # MlxBackend implementation (mlx-rs) [feature-gated]
└── cuda.rs             # CudaBackend implementation (future) [feature-gated]
```

### 7.2 MultiField Type (in `deep_causality_multivector`)

The `CausalMultiField<B, T>` type and its operations:

```
deep_causality_multivector/src/types/multifield/
├── mod.rs              # CausalMultiField struct definition
├── algebra.rs          # Ring, Module trait implementations
├── products.rs         # geometric_product, cross, inner, wedge
├── differential.rs     # curl, divergence, gradient
├── grades.rs           # grade_project, scalar_part, vector_part
├── conversions.rs      # from_coefficients, to_coefficients
└── gamma/              # Backend-specific Gamma matrix loaders
    ├── mod.rs          # BackendGamma trait
    ├── cpu.rs          # CpuBackend gamma tables (const/lazy)
    ├── mlx.rs          # MlxBackend gamma upload [feature-gated]
    └── cuda.rs         # CudaBackend gamma upload [feature-gated]
```

---

## 8. Example: Generic MHD Solver

```rust
use deep_causality_multivector::CausalMultiField;
use deep_causality_tensor::backend::{CpuBackend, MlxBackend};

// --- Verification Run (CPU, f64) ---
type VerifyBackend = CpuBackend;
let v_field = CausalMultiField::<VerifyBackend, f64>::from_coefficients(...);
let curl_v = v_field.curl();
assert!(energy_conserved(&v_field, &curl_v, 1e-14)); // High precision check

// --- Production Run (MLX, f32) ---
type ProdBackend = MlxBackend;
let v_field = CausalMultiField::<ProdBackend, f32>::from_coefficients(...);
for _ in 0..1000 {
let curl_v = v_field.curl();            // GPU accelerated
let lorentz = j.cross( & b_field);        // GPU accelerated
v_field = v_field.add( & lorentz.scale(dt));
}
```

---

## 9. Applications

| Domain          | Algebra | Grid Size | Backend | Speedup       |
|-----------------|---------|-----------|---------|---------------|
| MHD Stellarator | Cl(3,0) | 256³      | MLX     | ~24×          |
| Lattice QCD     | Cl(10)  | 32⁴       | CUDA    | ~30×          |
| GR Tetrad       | Cl(1,3) | 128³      | CPU     | 1× (baseline) |
| Clifford CNN    | Cl(3,0) | 64×64     | MLX     | ~15×          |

---

## 10. Default Type Aliases (Feature-Flag Based)

```rust
// deep_causality_multivector/src/types/multifield/aliases.rs

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultBackend = MlxBackend;

#[cfg(not(feature = "mlx"))]
pub type DefaultBackend = CpuBackend;

#[cfg(feature = "mlx")]
pub type DefaultFloat = f32;

#[cfg(not(feature = "mlx"))]
pub type DefaultFloat = f64;

/// Convenience Alias: Backend-agnostic MultiField
pub type MultiField<T = DefaultFloat> = CausalMultiField<DefaultBackend, T>;
```

**Usage:**

```rust
use deep_causality_multivector::MultiField;

// Backend selected at compile-time
let v_field: MultiField = MultiField::from_coefficients(...);
let curl = v_field.curl(); // Uses MLX or CPU automatically
```

---

## 11. Testing Strategy

### 11.1 Test Structure

```
deep_causality_multivector/tests/
├── types/
│   └── multifield/
│       ├── mod.rs
│       ├── geometric_product_tests.rs
│       ├── differential_tests.rs    # curl, div, grad
│       ├── grade_projection_tests.rs
│       └── conversion_tests.rs
└── backend_parity/
    ├── mod.rs
    └── cpu_mlx_multifield_tests.rs  # [feature-gated]
```

### 11.2 Test Categories

| Category                  | Description                                  |
|---------------------------|----------------------------------------------|
| **Algebraic Laws**        | Associativity, identity, distributivity      |
| **Differential Accuracy** | Compare finite diff vs. analytic derivatives |
| **Conservation Laws**     | Energy/momentum conservation in MHD step     |
| **Backend Parity**        | CPU results match MLX within tolerance       |

### 11.3 Example Tests

```rust
#[test]
fn geometric_product_associativity() {
    let a = MultiField::<CpuBackend, f64>::random(...);
    let b = MultiField::random(...);
    let c = MultiField::random(...);

    let left = a.geometric_product(&b).geometric_product(&c);
    let right = a.geometric_product(&b.geometric_product(&c));

    assert_approx_eq!(left, right, 1e-12);
}

#[test]
fn curl_of_gradient_is_zero() {
    let scalar_field = MultiField::<CpuBackend, f64>::random_scalar_field(...);
    let grad = scalar_field.gradient();
    let curl = grad.curl();

    assert!(curl.max_norm() < 1e-10, "∇×∇φ should be zero");
}
```

---

## 12. RustDoc Guidelines

```rust
/// A hardware-accelerated field of multivectors.
///
/// `CausalMultiField` stores multivector data in Matrix Isomorphism representation
/// directly on the backend device, enabling efficient GPU/accelerator computation
/// of Clifford algebra operations.
///
/// # Type Parameters
///
/// * `B` - The compute backend ([`CpuBackend`], [`MlxBackend`]).
/// * `T` - The scalar type (typically `f32` or `f64`).
///
/// # Memory Layout
///
/// Shape: `[Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]`
///
/// # Example
///
/// ```rust
/// use deep_causality_multivector::{CausalMultiField, CpuBackend};
///
/// let field = CausalMultiField::<CpuBackend, f64>::zeros([128, 128, 128]);
/// let curl = field.curl();
/// ```
pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> {
    ...
}
```
