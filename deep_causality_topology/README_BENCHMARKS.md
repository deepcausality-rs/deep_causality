# Deep Causality Topology Benchmarks

This document details the performance benchmarks for the `deep_causality_topology` crate, comparing the standard CPU
implementation against the hardware-accelerated MLX (Apple Silicon GPU) implementation.

## 1. Performance Comparison: Christoffel Symbol Computation

The following benchmarks measure the time to compute Christoffel symbols for different grid sizes. 
All benchmarks were run on a Apple Silicon M3 Max. 

| Grid Size | CPU Time  | MLX (GPU) Time | Speedup      |
|:----------|:----------|:---------------|:-------------|
| **5³**    | 622.26 µs | 279.34 µs      | **2.2x**     |
| **10³**   | 4.96 ms   | 333.32 µs      | **14.8x**    |
| **20³**   | 39.20 ms  | 671.81 µs      | **58.3x** 🚀 |

> [!NOTE]
> **Scaling Limits**: For standard topology computations (4D), MLX on Apple Silicon remains highly performant up to a **128³ grid (~2 million points)**. For global simulations or grid resolutions beyond 128³, professional data center hardware with CUDA (e.g., NVIDIA H100) becomes necessary to handle the memory bandwidth and compute requirements.

### Benchmark Configuration

- **Total Elements**: Dimension 4 manifold.
- **CPU**: Native Rust implementation with LLVM optimizations.
- **MLX**: Hardware acceleration via Metal Performance Shaders.
- **Verification**: Benchmarks explicitly force MLX evaluation using `to_vec` to ensure real GPU execution time is
  measured (avoiding lazy evaluation artifacts).

---

## 2. Achieving GPU Acceleration with MLX

The performance gains in the topology crate were achieved through three primary optimizations:

### A. Fixing Lazy Evaluation Measurements

MLX uses lazy evaluation, where operations are queued but not executed until the data is actually needed. Previous
benchmarks were only measuring the "queue time" (~0.6µs), which was misleading. By forcing evaluation inside the
benchmark loop, we ensure that the timing reflects actual computation and data materialization.

### B. Implementation of Native MLX Operations

Many tensor operations in `deep_causality_tensor` previously fell back to CPU implementations. We implemented native MLX
versions for several critical bottlenecks:

- **`stack`**: Now uses native `mlx_rs::ops::stack`, avoiding CPU-mediated data movement.
- **`permute`**: Implemented via `mlx_rs::ops::transpose_axes` for high-performance axis reordering on the GPU.
- **`broadcast_op`**: Scalar multiplication (used extensively in Christoffel scaling) now triggers native MLX
  broadcasting kernels.

### C. Elimination of GPU-CPU Round-Trips

By ensuring that the computational graph stays entirely on the GPU during the `compute_christoffel` pass, we eliminated
costly data transfers between unified memory and CPU caches. This is why the speedup factor increases so dramatically
with larger grid sizes (from 2.2x to 58x), as the overhead of GPU kernel management becomes negligible compared to the
massive parallel processing power applied to the larger tensors.


## Run Benchmarks

### CPU 

`RUSTFLAGS="-C target-cpu=native" cargo bench -p deep_causality_topology`

### GPU / MLX 

`RUSTFLAGS="-C target-cpu=native" cargo bench -p deep_causality_topology --features mlx`
