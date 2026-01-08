/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::causal_tensor_type::bench_causal_tensor::causal_tensor_benches,
    benchmarks::mlx_vs_cpu::bench_matmul::mlx_vs_cpu_benches,
    benchmarks::mlx_vs_cpu::bench_roundtrip::roundtrip_benches,
    benchmarks::mlx_vs_cpu::bench_ein_sum::einsum_cpu_benches,
    benchmarks::mlx_vs_cpu::bench_ein_sum::einsum_mlx_benches,
}
