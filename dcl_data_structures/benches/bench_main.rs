// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::criterion_main;

mod benchmarks;


criterion_main! {
    benchmarks::bench_grid_array::array_grid,
    benchmarks::bench_window_arr::window_array_backed,
    benchmarks::bench_window_vec::window_vector_backed,
    benchmarks::bench_window_unsafe_arr::window_unsafe_array_backed,
    benchmarks::bench_window_unsafe_vec::window_unsafe_vector_backed,
    // Compares the performance of the different window implementations
    benchmarks::bench_window_comp::window_impl_comp,
}
