/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::criterion_main;

mod benchmarks;

// Always run these benchmarks
#[cfg(not(feature = "unsafe"))]
criterion_main! {
    benchmarks::grid_type::bench_grid_array::array_grid,
    benchmarks::window_type::bench_window_arr::window_array_backed,
    benchmarks::window_type::bench_window_vec::window_vector_backed,
    benchmarks::window_type::bench_window_comp::window_impl_comp
}

// Run these benchmarks only when unsafe feature is enabled
#[cfg(feature = "unsafe")]
criterion_main! {
    // Standard benchmarks
    benchmarks::grid_type::bench_grid_array::array_grid,
    benchmarks::grid_type::bench_grid_array_unsafe::array_grid_unsafe,
    benchmarks::window_type::bench_window_arr::window_array_backed,
    benchmarks::window_type::bench_window_vec::window_vector_backed,
    benchmarks::window_type::bench_window_comp::window_impl_comp,
    // Unsafe-specific benchmarks
    benchmarks::window_type::bench_window_unsafe_arr::window_unsafe_array_backed,
    benchmarks::window_type::bench_window_unsafe_vec::window_unsafe_vector_backed
}
