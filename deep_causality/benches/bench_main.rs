/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use criterion::criterion_main;

mod benchmarks;

// In case of SIGSEGV: invalid memory reference,
// just reduce sample size in the causality benchmarks.

criterion_main! {
    benchmarks::bench_grid_array::array_grid,
    benchmarks::bench_collection::causality_collection,
    benchmarks::bench_map::causality_map,
    benchmarks::bench_linear_graph::linear_graph,
    benchmarks::bench_multi_cause_graph::multi_layer_graph,
    benchmarks::bench_window_arr::window_array_backed,
    benchmarks::bench_window_vec::window_vector_backed,
}
