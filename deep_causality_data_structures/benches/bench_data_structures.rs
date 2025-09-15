/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::causal_tensor_type::bench_causal_tensor::causal_tensor_benches,
    benchmarks::grid_type::bench_grid_array::array_grid,
    benchmarks::window_type::bench_window_arr::window_array_backed,
    benchmarks::window_type::bench_window_vec::window_vector_backed,
    benchmarks::window_type::bench_window_comp::window_impl_comp
}
