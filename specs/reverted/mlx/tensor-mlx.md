# Specification: MLX Acceleration for CausalTensor

## 1. Overview

This specification outlines the integration of **MLX** (via the `mlx-rs` crate) into the `deep_causality_tensor` library
to provide hardware acceleration on Apple Silicon (M-series) devices.

Currently, `deep_causality_tensor` relies on CPU-bound `Vec<T>` operations. While efficient for small-scale logic, large
physics simulations are memory-bandwidth bound. Apple's Unified Memory Architecture (UMA) and the MLX framework are
ideally suited to address this bottleneck.

This implementation will be **opt-in** via a Cargo feature flag (`mlx`) and will leverage conditional compilation to
ensure zero overhead and no extra dependencies for non-Apple or default builds.

## 2. Architecture

### 2.1 Feature Flag & Conditional Compilation

The integration will be guarded by a `mlx` feature flag and platform checks.

**`Cargo.toml` updates:**

```toml
[features]
default = []
mlx = ["dep:mlx-rs"]

[dependencies]
# Optional dependency on mlx-rs
mlx-rs = { version = "0.21.0", optional = true }
```

**Code Guards:**
All MLX-related code will be guarded by:

```rust
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
```

This ensures that even if the feature is enabled accidentally on Linux/Windows, it won't break the build (though
`mlx-rs` itself might fail to build on non-Apple platforms, so strictly relying on the feature flag is the primary
control).

### 2.2 Data Bridge (Interoperability)

We will introduce a bridge module `src/extensions/mlx.rs` that handles the conversion between `CausalTensor<T>` and
`mlx_rs::Array`.

Since `CausalTensor` uses `Vec<T>` (row-major) and MLX supports row-major arrays, data transfer will initially involve
copying:

1. **To MLX:** `Vec<T>` -> `mlx_rs::Array`.
2. **From MLX:** `mlx_rs::Array` -> `Vec<T>`.

*Note: While zero-copy is the ultimate goal, it requires advanced unsafe pointer handling and `mlx-rs` support for
external buffers which is currently experimental. This v1 specification prioritizes correctness and compute acceleration
for heavy operations (`EinSum`, `MatMul`) where compute cost > copy cost.*

### 2.3 Supported Types

MLX primarily accelerates floating-point operations (`f32`, `f16`, `bf16`). `CausalTensor<T>` is generic.
We will define a trait `MlxCompatible` to limit acceleration to valid types.

```rust
pub trait MlxCompatible: Sized {
    fn to_mlx_dtype() -> mlx_rs::Dtype;
    // Helper to convert Vec<Self> to mlx_rs::Array
    fn into_mlx_array(data: &[Self], shape: &[i32]) -> Result<mlx_rs::Array, CausalTensorError>;
}
```

**f64 Handling Strategy:**

MLX's `Dtype::Float64` exists but runs on **CPU only** (Metal does not support f64). For GPU acceleration,
f64 tensors must be downcast to f32. The trait implementation handles this transparently:

```rust
impl MlxCompatible for f64 {
    fn to_mlx_dtype() -> mlx_rs::Dtype {
        mlx_rs::Dtype::Float32  // GPU-acceleratable precision
    }

    fn into_mlx_array(data: &[Self], shape: &[i32]) -> Result<mlx_rs::Array, CausalTensorError> {
        // Downcast f64 -> f32 for GPU acceleration
        let f32_data: Vec<f32> = data.iter().map(|&x| x as f32).collect();
        mlx_rs::Array::from_slice(&f32_data, shape)
            .map_err(|_| CausalTensorError::MlxConversionFailed)
    }
}
```

> **Note:** Results from MLX operations on downcast tensors are returned as f32. Users requiring f64
> precision must upcast the result if needed. This is an explicit trade-off: GPU speed vs precision.

### 2.4 MLX-Native Constructors

To avoid conversion overhead when MLX usage is known upfront, we provide constructors that build
the `mlx_rs::Array` backing immediately. This eliminates the `Vec<T>` → `mlx_rs::Array` copy at
operation time.

**New Type: `MlxCausalTensor`**

```rust
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub struct MlxCausalTensor {
    array: mlx_rs::Array,
    shape: Vec<usize>,
}

impl MlxCausalTensor {
    /// Creates an MLX-backed tensor directly from f32 data.
    /// No conversion overhead—data goes straight to MLX.
    pub fn new_f32(data: Vec<f32>, shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let array = mlx_rs::Array::from_slice(&data, &mlx_shape)
            .map_err(|_| CausalTensorError::MlxConversionFailed)?;
        Ok(Self { array, shape })
    }

    /// Creates an MLX-backed tensor from f64 data with precision downcast.
    /// **Warning:** This downcasts f64 → f32 for GPU acceleration.
    pub fn new_from_f64(data: &[f64], shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let f32_data: Vec<f32> = data.iter().map(|&x| x as f32).collect();
        Self::new_f32(f32_data, shape)
    }

    /// Creates an MLX-backed tensor from an existing CausalTensor<f32>.
    /// Single copy: Vec<f32> → mlx_rs::Array.
    pub fn from_causal_tensor(tensor: &CausalTensor<f32>) -> Result<Self, CausalTensorError> {
        Self::new_f32(tensor.data().to_vec(), tensor.shape().to_vec())
    }

    /// Creates an MLX-backed tensor from CausalTensor<f64> with downcast.
    pub fn from_causal_tensor_f64(tensor: &CausalTensor<f64>) -> Result<Self, CausalTensorError> {
        Self::new_from_f64(tensor.data(), tensor.shape().to_vec())
    }

    /// Converts back to CausalTensor<f32> after MLX operations.
    pub fn to_causal_tensor(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        self.array.eval().map_err(|_| CausalTensorError::MlxEvalFailed)?;
        let data: Vec<f32> = self.array.as_slice()
            .map_err(|_| CausalTensorError::MlxConversionFailed)?
            .to_vec();
        CausalTensor::new(data, self.shape.clone())
    }

    /// Access the underlying MLX array for advanced operations.
    pub fn as_mlx_array(&self) -> &mlx_rs::Array {
        &self.array
    }
}
```

**Usage Pattern:**

```rust
// Scenario 1: Known MLX usage upfront (no double conversion)
let mlx_tensor = MlxCausalTensor::new_f32(data, shape)?;
let result = mlx_tensor.matmul(&other_mlx_tensor)?;
let output = result.to_causal_tensor()?;

// Scenario 2: Bridge from existing CausalTensor
let causal: CausalTensor<f64> = /* ... physics simulation ... */;
let mlx = MlxCausalTensor::from_causal_tensor_f64(&causal)?; // downcast + copy
let accelerated = mlx.ein_sum("ij,jk->ik", &other)?;
```

## 3. Implementation Strategy

### 3.1 New Module Structure

```
src/
  extensions/
    mod.rs
    mlx.rs        <-- New file: Bridge code and helpers
  types/
    causal_tensor/
      ops/
        tensor_mlx.rs <-- New file: Accelerated implementation logic
```

### 3.2 Transparent Acceleration

Instead of forcing users to call `mlx_matmul`, we will attempt to transparently accelerate standard operations when
valid.

**Example: `matmul` modification**

```rust
impl<T> Tensor<T> for CausalTensor<T> {
    fn matmul(&self, rhs: &Self) -> Result<Self, CausalTensorError>
    where
        T: Ring + Copy + Default + PartialOrd + MlxCompatible, // Trait usage needs care
    {
        // 1. Try MLX acceleration if enabled
        #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
        if self.len() >= MLX_ACCELERATION_THRESHOLD {
            if let Ok(res) = self.mlx_matmul(rhs) {
                return Ok(res);
            }
        }

        // 2. Fallback to CPU implementation
        self.cpu_matmul(rhs)
    }
}
```

*Refinement:* Since we cannot easily change the trait bounds of the existing `matmul` (it might not require
`MlxCompatible`), the `mlx_matmul` call will likely be inside a specialized block or use `Any` casting / specialization
workaround if `T` is flexible. Use of a "Best Effort" approach is recommended: if `T` assumes support (e.g. `f32`), use
MLX, otherwise CPU.

### 3.3 Priority Operations

The following operations are candidates for acceleration, in order of impact:

1. **`ein_sum`**: High computational complexity ($O(N^k)$). Prime candidate.
2. **`matmul`**: $O(N^3)$. Standard heavy op.
3. **`inverse` / `cholesky`**: $O(N^3)$.
4. **`tensor_product`**: Large memory expansion.

### 3.4 API Extensions

We will also expose explicit MLX methods for users who want deterministic behavior:

- `fn as_mlx_array(&self) -> Result<mlx_rs::Array>`
- `fn from_mlx_array(arr: &mlx_rs::Array) -> Result<Self>`
- `fn mlx_ein_sum(...)`

## 4. Work Definition

### Phase 1: Infrastructure

1. Add `mlx-rs` to `Cargo.toml` as an optional dependency.
2. Create `src/extensions/mlx.rs`.
    - Implement `MlxCompatible` trait for `f32` (native), `f64` (with f32 downcast), `f16`.
    - Implement conversion generic helpers.
3. Create `src/types/mlx_causal_tensor.rs`.
    - Implement `MlxCausalTensor` struct with MLX-native backing.
    - Implement `new_f32`, `new_from_f64`, `from_causal_tensor`, `from_causal_tensor_f64` constructors.
    - Implement `to_causal_tensor` for conversion back to `CausalTensor<f32>`.

### Phase 2: Operations

1. **`ein_sum`**:
    - Parse `EinSumAST`.
    - Map to `mlx_rs::ops::einsum`.
    - Handle shape checking.
2. **`matmul`**:
    - Map to `mlx_rs::ops::matmul`.
3. **`inverse` / `svd` / `cholesky`**:
    - Map to `mlx_rs::linalg::inv`, `svd`, `cholesky`.

### Phase 3: Integration

1. Modify `src/types/causal_tensor/mod.rs` to include the accelerated methods.
2. Add tests guarded by `#[cfg(feature = "mlx")]`.
3. Benchmark comparison (CPU vs MLX) to tune `MLX_ACCELERATION_THRESHOLD`.

## 5. Challenges & Mitigations

| Challenge                     | Mitigation                                                                                                                                                            |
|-------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Data Copy Overhead**        | Only trigger MLX path for tensors with $N > 10,000$ elements or $O(N^3)$ ops.                                                                                         |
| **Generics vs Dynamic Types** | Use a customized `MlxCompatible` trait to gate access. Fallback to CPU for unsupported types.                                                                         |
| **f64 Support**               | Metal GPU does not support `f64`. Auto-downcast f64→f32 for GPU acceleration; users upcast results if needed. `MlxCausalTensor::new_from_f64` handles this transparently. |
| **Compilation Time**          | Feature flag ensures users don't pay compile cost unless requested.                                                                                                   |

## 6. Verification Plan

- **Unit Tests:** Run `cargo test --features mlx` on a Mac M1/M2/M3.
- **Reference Checks:** Compare MLX output vs CPU output for numerical correctness (tolerance $\epsilon \approx 1e-5$
  for f32).
