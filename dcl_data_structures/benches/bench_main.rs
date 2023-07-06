// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::bench_grid_array::array_grid,
    benchmarks::bench_window_arr::window_array_backed,
    benchmarks::bench_window_vec::window_vector_backed,
}
