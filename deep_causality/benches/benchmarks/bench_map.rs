/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};

use deep_causality::traits::causable::causable_reasoning::CausableReasoning;

use crate::benchmarks::utils_map;

// Small = 10
// Medium = 1_000
// Large = 10_000

fn small_causality_map_benchmark(criterion: &mut Criterion) {
    let (map, data) = utils_map::get_small_map_and_data();
    criterion.bench_function("small_causality_map", |bencher| {
        bencher.iter(|| map.reason_all_causes(&data).unwrap())
    });
}

fn medium_causality_map_benchmark(criterion: &mut Criterion) {
    let (map, data) = utils_map::get_medium_map_and_data();
    criterion.bench_function("medium_causality_map", |bencher| {
        bencher.iter(|| map.reason_all_causes(&data).unwrap())
    });
}

fn large_causality_map_benchmark(criterion: &mut Criterion) {
    let (map, data) = utils_map::get_large_map_and_data();
    criterion.bench_function("large_causality_map", |bencher| {
        bencher.iter(|| map.reason_all_causes(&data).unwrap())
    });
}

criterion_group! {
    name = causality_map;
    config = Criterion::default().sample_size(100);
    targets =
    small_causality_map_benchmark,
    medium_causality_map_benchmark,
    large_causality_map_benchmark,
}
