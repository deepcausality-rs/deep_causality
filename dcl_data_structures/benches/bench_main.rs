// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::criterion_main;

mod benchmarks;

#[cfg(not(feature = "unsafe"))]
criterion_main! {
    benchmarks::bench_grid_array::array_grid,
}

#[cfg(feature = "unsafe")]
criterion_main! {
    benchmarks::bench_grid_array::array_grid,
    benchmarks::bench_grid_array_unsafe::array_grid_unsafe,
}
