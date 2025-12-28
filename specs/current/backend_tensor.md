# Generic Backend: Tensor Specification

## 1. Overview

The `deep_causality_tensor` crate provides the foundational N-dimensional array abstractions. By introducing a **Generic
Backend Architecture**, we decouple the high-level API (`CausalTensor`) from the low-level execution engine (`CPU`,
`MLX`, `CUDA`).

This enables:

1. **Verification (CPU):** `f64` precision for rigorous physics/math correctness.
2. **Performance (MLX/CUDA):** `f32`/`f16` hardware acceleration for production workloads.
3. **Portability:** Code written once runs on MacBook NPUs and Linux GPU Clusters.

---

## 2. Architecture

### 2.1 The Backend Trait

The `TensorBackend` trait defines the contract that all compute engines must satisfy. It abstracts memory management,
arithmetic, and linear algebra.

```rust
// deep_causality_tensor/src/backend/mod.rs

pub trait TensorBackend: Clone + Send + Sync + 'static {
    /// The concrete tensor type used by this backend
    type Tensor<T>;

    /// Device identifier (e.g., Cpu, Gpu(0))
    fn device(&self) -> &Device;

    // --- Creation ---
    fn create<T: TensorData>(data: &[T], shape: &[usize]) -> Self::Tensor<T>;
    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T>;
    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T>;
    fn from_shape_fn<T: TensorData, F>(shape: &[usize], f: F) -> Self::Tensor<T>
    where
        F: FnMut(&[usize]) -> T;

    // --- Data Access (Sync) ---
    /// Downloads data from device to host Vector
    fn to_vec<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<T>;

    // --- Shape & Manipulation ---
    fn shape<T: TensorData>(tensor: &Self::Tensor<T>) -> &[usize];
    fn reshape<T: TensorData>(tensor: &Self::Tensor<T>, shape: &[usize]) -> Self::Tensor<T>;
    fn permute<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T>;
    fn slice<T: TensorData>(tensor: &Self::Tensor<T>, ranges: &[Range<usize>]) -> Self::Tensor<T>;

    // --- Arithmetic (Element-wise) ---
    fn add<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;
    fn sub<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;
    fn mul<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;
    fn div<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;

    // --- Reduction ---
    fn sum<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T>;
    fn max<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T>;
}

/// Advanced Linear Algebra operations (Split for modularity)
/// Required for MultiField and Topology Spectral Analysis
pub trait LinearAlgebraBackend: TensorBackend {
    // --- Matrix Multiplication ---
    /// General Matrix Multiplication (GEMM) supporting broadcasting
    fn matmul<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;

    // --- Decomposition & Solvers (Future/Optional) ---
    fn qr<T: TensorData>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>);
    fn svd<T: TensorData>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>, Self::Tensor<T>);
    fn inverse<T: TensorData>(input: &Self::Tensor<T>) -> Self::Tensor<T>;
}

/// Marker trait for allowed tensor data types
/// We reuse `deep_causality_num::Field` which covers f32, f64, Complex, etc.
pub trait TensorData: Field + Copy + Send + Sync + 'static {}

// Blanket implementation
impl<T> TensorData for T
where
    T: Field + Copy + Send + Sync + 'static
{}
```

### 2.2 The Generic `CausalTensor`

The `CausalTensor` struct now wraps the backend-specific tensor type.

```rust
// deep_causality_tensor/src/types/causal_tensor.rs

pub struct CausalTensor<B: TensorBackend, T: TensorData> {
    pub(crate) data: B::Tensor<T>,
    pub(crate) shape: Vec<usize>, // Cached for fast access
    _marker: PhantomData<B>,
}
```

---

## 3. Implementations

### 3.1 Dependency Philosophy

> **Minimal External Dependencies for Long-Term Maintainability**
>
> External crates introduce maintenance risk: denied feature requests, slow bug fixes, breaking changes,
> and abandonment. The backend architecture prioritizes internal implementations over external dependencies
> wherever feasible.

| Dependency | Status | Rationale |
|------------|--------|-----------|
| `ndarray` | **Not Used** | Existing `CausalTensor<T>` is pure Rust with `Vec<T>` + strides |
| `rayon` | **Opt-in** (`parallel` feature) | Mature, stable API, essential for CPU parallelism |
| `mlx-rs` | **Opt-in** (`mlx` feature) | Required for Apple Silicon GPU — no alternative |
| `cudarc` | **Opt-in** (`cuda` feature) | Required for NVIDIA GPU — no alternative |

### 3.2 CpuBackend (Reference)

**Backing:** The existing pure Rust `CausalTensor<T>` with `Vec<T>` and stride-based indexing.

```rust
// CpuBackend reuses existing CausalTensor — no new external deps
pub struct CpuBackend;

impl TensorBackend for CpuBackend {
    type Tensor<T> = CausalTensor<T>;
    
    fn create<T: TensorData>(data: &[T], shape: &[usize]) -> Self::Tensor<T> {
        CausalTensor::new(data.to_vec(), shape.to_vec()).unwrap()
    }
    // ... other methods delegate to existing CausalTensor ops
}
```

**Parallelism (Optional):**

```toml
# Cargo.toml
[features]
default = []
parallel = ["rayon"]  # Opt-in CPU parallelism

[dependencies]
rayon = { version = "1.10", optional = true }
```

```rust
impl CpuBackend {
    fn matmul<T: TensorData + Send + Sync>(a: &CausalTensor<T>, b: &CausalTensor<T>) -> CausalTensor<T> {
        #[cfg(feature = "parallel")]
        return matmul_parallel(a, b);  // Uses rayon::par_iter
        
        #[cfg(not(feature = "parallel"))]
        return matmul_sequential(a, b);  // Naive O(n³) loop
    }
}
```

| Feature | With `parallel` | Without `parallel` |
|---------|-----------------|-------------------|
| matmul 256×256 | ~4ms (8 cores) | ~20ms |
| element-wise ops | ~5× faster | Baseline |
| External dep | `rayon` only | Zero |

**Precision:** Full support (`f32`, `f64`, `Complex64`, `Complex32`, `i32`, `i64`).

**Role:** Correctness verification, debugging, small-scale physics, cross-platform fallback.

### 3.3 MlxBackend (Apple Silicon)

* **Backing:** `mlx_rs::array`
* **Precision:**
    * `f32`: Native.
    * `f16`/`bfloat16`: Native.
    * `f64`: **Not Supported** (panics in strict mode).
    * `Complex64`: Simulated as `Complex32` (f32/f32) or interleaved buffers.
* **Role:** High-performance production runs on Mac Studio/MacBook Pro.

### 3.4 CudaBackend (Future)

* **Backing:** `cudarc::CudaSlice` + cuBLAS/cuDNN bindings.
* **Precision:** `f32`, `f64` (double precision on Data Center GPUs).
* **Role:** Scaling to clusters (AWS, DGX).

---

## 4. Precision Policy

Since `TensorBackend` is generic over `T`, how do we handle backend limitations (like MLX lacking f64)?

**Strategy: Trait Bounds & Runtime Checks**

1. **Compile-Time (Ideal):**
   If a backend implements `TensorBackend`, it *should* support the requested `T`.
    * `CpuBackend` implements `TensorBackend` for `f64`.
    * `MlxBackend` technically *can* implement it but panics, or we use a subtrait `Float64Backend`.

2. **Pragmatic Approach:**
   Implement `TensorBackend` for `MlxBackend` but `create<f64>` internally casts to `f32` (with a warning logging once)
   or Panics (Strict Mode).
    * *Decision:* **Panic in Strict Mode** (Default). Explicitly unsafe/lossy cast method provided if user explicitly
      forces it.
    * Reason: Physics simulations failing silently due to precision loss is catastrophic.

---

## 5. Migration Strategy

1. **Define Traits:** Create `src/backend/mod.rs` with `TensorBackend` trait.
2. **Implement CpuBackend:** Wrap existing `CausalTensor<T>` — no new deps required.
3. **Add Parallelism:** Implement `parallel` feature with rayon for matmul/element-wise ops.
4. **Refactor CausalTensor:** Change struct definition to generic `CausalTensor<B, T>`.
5. **Fix Call Sites:** Update all standard code to use `CausalTensor<CpuBackend, f64>` aliased via `DefaultBackend`.
6. **Implement MLX:** Add `MlxBackend` in `src/backend/mlx.rs`.

---

## 6. Example Usage

```rust
// Verification Run
type Backend = CpuBackend;
let a: CausalTensor<Backend, f64> = CausalTensor::zeros( & [2, 2]);

// Production Run
type Backend = MlxBackend;
let a: CausalTensor<Backend, f32> = CausalTensor::zeros( & [128, 128]); // Note f32
```

---

## 7. Default Type Aliases (Feature-Flag Based Backend Selection)

To enable users to write backend-agnostic code while switching backends at compile-time:

```rust
// deep_causality_tensor/src/backend/aliases.rs

/// Default Backend Selection via Feature Flags
/// User code uses `DefaultBackend` and the compiler selects CPU/MLX/CUDA.
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultBackend = MlxBackend;

#[cfg(all(feature = "cuda", not(feature = "mlx")))]
pub type DefaultBackend = CudaBackend;

#[cfg(not(any(feature = "mlx", feature = "cuda")))]
pub type DefaultBackend = CpuBackend;

/// Default Precision Matching Backend
#[cfg(feature = "mlx")]
pub type DefaultFloat = f32;

#[cfg(not(feature = "mlx"))]
pub type DefaultFloat = f64;

/// Convenience Alias: The "just works" tensor type
pub type Tensor<T = DefaultFloat> = CausalTensor<DefaultBackend, T>;
```

**Usage:**

```rust
use deep_causality_tensor::prelude::*;

// User code is backend-agnostic
let field: Tensor = Tensor::zeros(& [128, 128, 128, 4, 4]);
let result = field.matmul( & other); // Dispatches to MLX/CUDA/CPU automatically

// Explicit override when needed
let cpu_tensor: CausalTensor<CpuBackend, f64> = CausalTensor::zeros( & [2, 2]);
```

**Cargo.toml:**

```toml
[features]
default = []
mlx = ["mlx-rs"]
cuda = ["cudarc"]
```

**Build Commands:**

```bash
# CPU (default, f64 precision)
cargo build -p deep_causality_tensor

# MLX (Apple Silicon, f32 precision)
cargo build -p deep_causality_tensor --features mlx

# CUDA (Data Center, f64/f32)
cargo build -p deep_causality_tensor --features cuda
```

---

## 8. Testing Strategy

### 8.1 Test Structure (mirrors AGENTS.md)

```
deep_causality_tensor/
└── tests/
    ├── mod.rs
    ├── backend/
    │   ├── mod.rs
    │   ├── cpu_tests.rs           # CpuBackend unit tests
    │   ├── mlx_tests.rs           # MlxBackend tests [feature-gated]
    │   └── backend_parity_tests.rs # CPU vs MLX numerical equivalence
    └── types/
        └── causal_tensor/
            ├── mod.rs
            └── tensor_ops_tests.rs
```

### 8.2 Test Categories

| Category              | Description                                 | Feature Gate |
|-----------------------|---------------------------------------------|--------------|
| **Unit Tests**        | Individual backend method correctness       | Per-backend  |
| **Parity Tests**      | CPU result == MLX result (within tolerance) | `mlx`        |
| **Property Tests**    | Algebraic laws (associativity, identity)    | None         |
| **Integration Tests** | Full pipeline (create→op→download)          | All backends |

### 8.3 Example Tests

```rust
// tests/backend/cpu_tests.rs
#[test]
fn cpu_matmul_correctness() {
    let a = CpuBackend::create(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
    let b = CpuBackend::create(&[5.0, 6.0, 7.0, 8.0], &[2, 2]);
    let c = CpuBackend::matmul(&a, &b);
    let result = CpuBackend::to_vec(&c);
    assert_eq!(result, vec![19.0, 22.0, 43.0, 50.0]);
}

// tests/backend/backend_parity_tests.rs
#[cfg(all(feature = "mlx", target_os = "macos"))]
#[test]
fn mlx_matches_cpu_matmul() {
    let data: Vec<f32> = (0..16).map(|i| i as f32).collect();
    let cpu_a = CpuBackend::create(&data, &[4, 4]);
    let mlx_a = MlxBackend::create(&data, &[4, 4]);

    let cpu_result = CpuBackend::to_vec(&CpuBackend::matmul(&cpu_a, &cpu_a));
    let mlx_result = MlxBackend::to_vec(&MlxBackend::matmul(&mlx_a, &mlx_a));

    for (c, m) in cpu_result.iter().zip(mlx_result.iter()) {
        assert!((c - m).abs() < 1e-5, "Mismatch: CPU {} vs MLX {}", c, m);
    }
}
```

---

## 9. Code Structure Guidelines (per AGENTS.md)

### 9.1 File Layout

```
deep_causality_tensor/src/
├── errors/
│   ├── mod.rs
│   └── tensor_error.rs
├── traits/
│   ├── mod.rs
│   └── tensor_backend.rs      # TensorBackend trait
├── types/
│   └── causal_tensor/
│       ├── mod.rs             # Struct definition + constructors
│       ├── algebra.rs         # Ring, Module impls
│       ├── ops.rs             # Element-wise ops
│       └── display.rs         # Debug, Display impls
├── backend/
│   ├── mod.rs                 # Re-exports + DefaultBackend alias
│   ├── cpu.rs                 # CpuBackend
│   ├── mlx.rs                 # MlxBackend [feature-gated]
│   ├── cuda.rs                # CudaBackend [feature-gated]
│   └── aliases.rs             # Type aliases
└── lib.rs                     # Public exports
```

### 9.2 Conventions

| Rule                  | Application                               |
|-----------------------|-------------------------------------------|
| Private fields        | `pub(crate) data`, `pub(crate) shape`     |
| Static dispatch       | No `dyn`, no trait objects                |
| No unsafe             | All backend implementations are safe Rust |
| No macros in lib      | Test macros only in `/tests`              |
| One type = one module | `CausalTensor` in `types/causal_tensor/`  |

---

## 10. RustDoc Guidelines

### 10.1 Trait Documentation

```rust
/// Defines the compute backend contract for tensor operations.
///
/// This trait abstracts over hardware execution (CPU, MLX, CUDA), enabling
/// backend-agnostic physics code while providing precision and performance
/// flexibility.
///
/// # Implementing a Backend
///
/// Each backend must implement:
/// 1. **Creation:** `create`, `zeros`, `ones`
/// 2. **Arithmetic:** `add`, `sub`, `mul`, `div`
/// 3. **Shape ops:** `reshape`, `permute`, `slice`
/// 4. **Data access:** `to_vec`
///
/// # Example
///
/// ```rust
/// use deep_causality_tensor::{CausalTensor, CpuBackend};
///
/// let a: CausalTensor<CpuBackend, f64> = CausalTensor::zeros(&[2, 2]);
/// ```
pub trait TensorBackend: Clone + Send + Sync + 'static { ... }
```

### 10.2 Method Documentation

```rust
impl<B: TensorBackend, T: TensorData> CausalTensor<B, T> {
    /// Creates a tensor filled with zeros.
    ///
    /// # Arguments
    ///
    /// * `shape` - The dimensions of the tensor (e.g., `&[2, 3, 4]`).
    ///
    /// # Returns
    ///
    /// A new `CausalTensor` with all elements initialized to zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// let t: CausalTensor<CpuBackend, f64> = CausalTensor::zeros(&[3, 3]);
    /// assert_eq!(t.shape(), &[3, 3]);
    /// ```
    pub fn zeros(shape: &[usize]) -> Self { ... }
}
```

