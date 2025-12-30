# DeepCausality Multivector Benchmarks

This document details the performance benchmarks for the `deep_causality_multivector` crate, comparing the **Algebraic
CPU** backend against the **Matrix MLX** backend (Apple Silicon).

**Device Context:** Apple M1/M2 Max (approximate).

---

## 1. MultiField Benchmarks

*Operations on 3D Grids of Multivectors (Fields).*

These benchmarks measure the performance of operations applied to entire tensor fields (e.g., a $4 \times 4 \times 4$
grid of multivectors).

### Geometric Product Scaling (3D Grid)

| Grid Size    | Cells | CPU Time (Linear) | MLX Time (Constant) | Speedup        |
|:-------------|:------|:------------------|:--------------------|:---------------|
| **4x4x4**    | 64    | ~135 µs           | ~0.60 µs            | **225x**       |
| **8x8x8**    | 512   | ~1.07 ms          | ~0.61 µs            | **1754x**      |
| **16x16x16** | 4096  | ~8.50 ms          | ~0.60 µs            | **14,166x** 🚀 |

**Insight:**

* **CPU:** Scales linearly $O( Cells )$. Processing 4000 cells takes ~8.5ms.
* **MLX:** Exhibits effectively **constant time** $O(1)$ for these batch sizes, dominated by kernel launch overhead.
  Processing 4000 cells is instant (~0.6µs).

### Field Operations Performance

*Field size: 4x4x4 (64 cells)*

| Operation             | CPU (Default) | MLX (`--features mlx`) | Note                    |
|:----------------------|:--------------|:-----------------------|:------------------------|
| **Geometric Product** | ~130 µs       | ~0.60 µs               | **200x** speedup        |
| **Gradient**          | ~160 µs       | ~1.6 ms                | Slow on MLX (overhead)  |
| **From Coefficients** | ~9.5 µs       | ~7.6 µs                | MLX creates data faster |
| **To Coefficients**   | ~11.8 µs      | ~221 µs                | Readback cost on MLX    |

---

## 2. MultiVector Benchmarks

*Operations on Single Multivectors.*

These benchmarks compare the traditional **Algebraic Implementation** (iterating basis blades on CPU) against the *
*Matrix Isomorphism Bridge** (using MLX matrices on GPU).

### High-Dimensional Scaling (Single Op)

Comparing `Geometric Product` for increasingly complex algebras.

| Algebra           | Dimensions | CPU (Algebraic) | MLX (Matrix) | Speedup        |
|:------------------|:-----------|:----------------|:-------------|:---------------|
| **Euclidean 2D**  | 4          | ~94 ns          | ~96 ns       | 1.0x (Parity)  |
| **PGA 3D**        | 16         | ~100 ns         | ~90 ns       | 1.1x           |
| **Dixon Cl(0,6)** | 64         | ~30.5 µs        | ~0.45 µs     | **68x**        |
| **Cl(0,9)**       | 512        | ~7.36 ms        | ~0.60 µs     | **12,266x** 🤯 |

> **Note:** When using `f32` floats (default for MLX feature), the CPU algebraic implementation currently regresses
> significantly (~400x slower for Dixon). The table above compares "Best CPU" (f64, no feature) vs "Best MLX" (f32, mlx
> feature). Even in the fairest comparison, **Matrix MLX wins by 12,000x for 512-dim algebras.**

**Insight:**

* **Algebraic limit:** CPU performance degrades exponentially with algebra dimension ($O(D^2)$ or worse).
* **Matrix power:** MLX matrix multiplication handles 512-dimension algebras (equivalent to small 16x16 or 32x32
  matrices) instantly.
* **Threshold:** For algebras smaller than 6 dimensions (e.g. standard 3D physics), CPU is competitive. For *
  *Hyper-dimensional physics (N > 6)**, the Matrix approach is mandatory.

---
*Benchmarks generate via `cargo bench` (CPU) and `cargo bench --features mlx` (MLX).*
