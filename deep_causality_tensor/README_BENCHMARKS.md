## MLX GPU Benchmarks (Apple Silicon)

Matrix multiplication (`matmul`) benchmarks comparing CPU vs GPU on Apple Silicon M3 Max:

**CPU vs GPU Comparison:**

| Size      | CPU (ms) | GPU (ms)  | Speedup    |
|-----------|----------|-----------|------------|
| 128×128   | 1.47     | 0.17      | **8.6x**   |
| 512×512   | 132.2    | 0.21      | **629x**   |
| 1024×1024 | 1,126    | 0.41      | **2,746x** |
| 2048×2048 | N/A      | 1.86      | **N/A**    |

**Note:** CPU times for large matrices (≥2048) are impractical for frequent benchmarking due to O(N³) growth.

### Data Transfer Overhead

Since `CausalTensor` uses `f64` and MLX uses `f32` (on Metal), using the GPU involves:

1. Downcasting `f64` → `f32`
2. Transfer to specific MLX array layout
3. Computing on GPU
4. Upcasting result `f32` → `f64`

Benchmarks show this roundtrip overhead is **negligible** compared to the compute gains at scale:

| Size      | Roundtrip Overhead | GPU Compute | Total GPU Time | CPU Time | Real Speedup |
|-----------|--------------------|-------------|----------------|----------|--------------|
| 128×128   | 0.007 ms           | 0.17 ms     | 0.18 ms        | 1.47 ms  | **8.2x**     |
| 512×512   | 0.081 ms           | 0.21 ms     | 0.29 ms        | 132 ms   | **455x**     |
| 1024×1024 | 0.370 ms           | 0.41 ms     | 0.78 ms        | 1,126 ms | **1,443x**   |

> **Conclusion:** Even including the full cost of memory allocation, casting, and transfer, MLX offers >1400x speedups for modern workloads.

### Skipping Conversion (Best Performance)

For maximum performance, construct tensors specifically from `f32` data to bypass the conversion overhead entirely. This is **6-7x faster** than the roundtrip method.

```rust
// 1. Initialize data as f32
let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
let shape = vec![2, 2];

// 2. Direct upload (Zero casting overhead)
let tensor = MlxCausalTensor::from_slice(&data, &shape);
```

**Benchmark Results:**

| Size      | Standard Roundtrip (f64→f32→GPU→f64) | Direct f32 (f32→GPU→f32) | Speedup  |
|-----------|--------------------------------------|--------------------------|----------|
| 128×128   | 6.7 µs                               | **0.90 µs**              | **7.4x** |
| 512×512   | 81.0 µs                              | **13.4 µs**              | **6.0x** |
| 1024×1024 | 369.8 µs                             | **51.2 µs**              | **7.2x** |

> **Note:** GPU acceleration scales dramatically with matrix size due to O(N³) complexity.

### Einstein Summation (EinSum) Benchmarks

Native GPU execution of tensor operations via the EinSum API. All operations run entirely on GPU—no CPU roundtrips.

**Matrix Operations (2D Tensors):**

| Operation   | Size      | CPU Time   | GPU Time   | Speedup      |
|-------------|-----------|------------|------------|--------------|
| MatMul      | 128×128   | 1.50 ms    | 0.17 ms    | **8.8x**     |
| MatMul      | 512×512   | 134.6 ms   | 0.22 ms    | **612x**     |
| MatMul      | 1024×1024 | 1,087 ms   | 0.41 ms    | **2,651x**   |
| Transpose   | 128×128   | 9.54 µs    | 22.31 µs   | 0.43x        |
| Transpose   | 512×512   | 125.4 µs   | 22.51 µs   | **5.6x**     |
| Transpose   | 1024×1024 | 565.3 µs   | 21.75 µs   | **26.0x**    |
| Hadamard    | 128×128   | 76.1 µs    | 145.9 µs   | 0.52x        |
| Hadamard    | 512×512   | 1.18 ms    | 158.3 µs   | **7.4x**     |
| Hadamard    | 1024×1024 | 4.81 ms    | 178.9 µs   | **26.9x**    |
| Trace       | 128×128   | 7.32 µs    | 163.9 µs   | 0.04x        |
| Trace       | 512×512   | 89.0 µs    | 170.4 µs   | 0.52x        |
| Trace       | 1024×1024 | 406.5 µs   | 168.5 µs   | **2.4x**     |
| Diagonal    | 128×128   | 9.27 µs    | 159.4 µs   | 0.06x        |
| Diagonal    | 512×512   | 97.0 µs    | 169.4 µs   | 0.57x        |
| Diagonal    | 1024×1024 | 419.6 µs   | 156.7 µs   | **2.7x**     |
| Reduction   | 128×128   | 76.6 µs    | 156.6 µs   | 0.49x        |
| Reduction   | 512×512   | 1.18 ms    | 163.4 µs   | **7.2x**     |
| Reduction   | 1024×1024 | 4.60 ms    | 167.2 µs   | **27.5x**    |

**Vector Operations (1D Tensors):**

| Operation | Size | CPU Time | GPU Time | Speedup      |
|-----------|------|----------|----------|--------------|
| DotProd   | 128  | 3.85 µs  | 147.3 µs | 0.03x        |
| DotProd   | 512  | 13.7 µs  | 160.4 µs | 0.09x        |
| DotProd   | 1024 | 27.1 µs  | 169.1 µs | 0.16x        |

**Batch Matrix Multiply (3D Tensors):**

| Batch × Size | CPU Time  | GPU Time  | Speedup     |
|--------------|-----------|-----------|-------------|
| 8 × 32×32    | 481.2 µs  | 156.3 µs  | **3.1x**    |
| 8 × 64×64    | 3.06 ms   | 157.4 µs  | **19.4x**   |
| 8 × 128×128  | 18.69 ms  | 166.3 µs  | **112x**    |

> **Key Insights:**
> - **O(N³) operations** (MatMul, BatchMatMul): MLX dominates, up to 2,750x faster for 1024×1024.
> - **O(N²) operations** (Hadamard, Reduction): MLX wins at larger sizes (≥512), ~27x faster at 1024.
> - **O(N) operations** (Trace, Diagonal, DotProd): CPU wins for small tensors due to GPU overhead (~160 µs baseline).
> - **Crossover point**: For most operations, GPU becomes faster around 512×512.
> - **Constant Overhead**: Notice how GPU times for Trace, Diagonal, and Reduction remain nearly constant (~160µs) regardless of size, proving we are bottlenecked by dispatch overhead, not computation.


## Run Benchmarks

### CPU Only (64-bit)

`RUSTFLAGS="-C target-cpu=native" cargo bench -p deep_causality_tensor`

### GPU / MLX (32-bit)

`RUSTFLAGS="-C target-cpu=native" cargo bench -p deep_causality_tensor --features mlx`
