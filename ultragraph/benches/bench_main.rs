// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::criterion_main;

mod benchmarks;

// In case of SIGSEGV: invalid memory reference,
// just reduce sample size.
criterion_main! {
    benchmarks::bench_linear::liner_graph_bench_collection,
}
