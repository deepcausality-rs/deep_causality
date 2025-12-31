# DeepCausality Multivector Benchmarks

This document details the performance benchmarks for the `deep_causality_multivector` crate, comparing the standard CPU
implementation against the hardware-accelerated MLX (Apple Silicon GPU) implementation.

## 1. CausalMultiField: Geometric Product (Grid Scaling)

The following benchmarks measure the time to compute the geometric product of two `CausalMultiField` objects across
different grid sizes. These tests demonstrate how GPU acceleration scales with data volume.

| Grid Size | CPU Time (Standard) | MLX (GPU) Time | Speedup      |
|:----------|:--------------------|:---------------|:-------------|
| **4³**    | 101.78 µs           | 174.77 µs      | 0.58x        |
| **8³**    | 797.06 µs           | 201.94 µs      | **3.9x**     |
| **16³**   | 6.379 ms            | 265.81 µs      | **24.0x** 🚀 |

> [!NOTE]
> **Scaling Limits**: For `CausalMultiField` operations, MLX on Apple Silicon is optimal for grids up to **128³ (~2
million points)**. For extreme-scale physics simulations exceeding these resolutions, migration to a CUDA-based backend
> on data center hardware (e.g., NVIDIA H100) is recommended. Note, CUDA support might get added in the future. If
> you hit these scaling limits, please fill an issue to priotize CUDA support. 

### Analysis

As the grid size increases, the massive parallel processing power of the GPU overcomes the initial kernel launch
overhead. For a 16³ grid, the MLX implementation is **24 times faster** than the optimized CPU version.

---

## 2. Geometric Algebra Performance

These benchmarks compare the algebraic geometric product (CPU) with the matrix-based geometric product (MLX GPU).

| Algebra            | CPU (Algebraic) | MLX (Matrix GPU) | Speedup      |
|:-------------------|:----------------|:-----------------|:-------------|
| **Dixon (8D)**     | 29.34 µs        | 187.08 µs        | 0.15x        |
| **Cl(0,9) (512D)** | 7.46 ms         | 177.23 µs        | **42.1x** 🚀 |

### Analysis

For low-dimensional algebras like Dixon (~8 dimensions), the optimized algebraic loops on the CPU are faster than the
overhead of GPU kernel submission. However, for high-dimensional experimental algebras like **Cl(0,9)** (512
dimensions), the MLX matrix implementation provides a massive **42x speedup**.

---

## 3. Basic Multivector Operations

| Operation           | CPU Time (64-bit) | MLX Enabled (32-bit CPU) | Note                  |
|:--------------------|:------------------|:-------------------------|:----------------------|
| **Euclidean 2D GP** | 96.22 ns          | 104.01 ns                | Slight f32 regression |
| **PGA 3D GP**       | 92.58 ns          | 99.79 ns                 | Slight f32 regression |
| **Addition 3D**     | 42.33 ns          | 43.60 ns                 | Slight f32 regression |

### The f32 vs f64 Performance Note

When the `mlx` feature is enabled, the crate shifts some internal operations to `f32` to maintain compatibility with
Metal Performance Shaders (which are optimized for 32-bit floats). As seen above, this results in a slight (~5-10%)
performance regression on CPU-only benchmarks compared to the standard 64-bit implementation. For small multivector
operations, the 64-bit CPU path remains the most efficient.

---

## 4. How Acceleration was Achieved

The performance gains in the multivector crate are primarily driven by:

1. **Batched Matrix Multiplication**: The `CausalMultiField` geometric product is mathematically mapped to a batched
   matrix multiplication. This allows us to leverage MLX's highly optimized `matmul` kernels, which utilize the AMX (
   Apple Matrix Coprocessor) and GPU execution units.
2. **Unified Memory Architecture**: Since Apple Silicon uses unified memory, there is zero data-copying overhead when "
   moving" data from the CPU-visible space to the GPU-visible space.
3. **Lazy Evaluation Handling**: We ensure real-world performance by forcing MLX to materialize results during
   benchmarking (using `to_vec`), ensuring that we measure actual computation time rather than just kernel queuing time.

---

## Run Benchmarks

### CPU Only (64-bit)

`RUSTFLAGS="-C target-cpu=native" cargo bench -p deep_causality_multivector`

### GPU / MLX (32-bit)

`RUSTFLAGS="-C target-cpu=native" cargo bench -p deep_causality_multivector --features mlx`
