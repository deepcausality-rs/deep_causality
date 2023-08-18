// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::criterion_main;

mod benchmarks;

// In case of SIGSEGV: invalid memory reference,
// just reduce sample size in the linear or multi cause graph benchmarks.
criterion_main! {
    benchmarks::bench_collection::causality_collection,
    benchmarks::bench_map::causality_map,
    benchmarks::bench_graph::linear_graph,
    benchmarks::bench_multi_cause_graph::multi_layer_graph,
}
