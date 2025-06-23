// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::{criterion_group, Criterion};

use deep_causality::traits::causable::CausableReasoning;

use crate::benchmarks::utils_collection;

// Small = 10
// Medium = 1_000
// Large = 10_000

fn small_causality_collection_benchmark(criterion: &mut Criterion) {
    let (coll, data) = utils_collection::get_small_collection_and_data();
    criterion.bench_function("small_causality_collection", |bencher| {
        bencher.iter(|| coll.reason_all_causes(&data).unwrap())
    });
}

fn medium_causality_collection_benchmark(criterion: &mut Criterion) {
    let (coll, data) = utils_collection::get_medium_collection_and_data();
    criterion.bench_function("medium_causality_collection", |bencher| {
        bencher.iter(|| coll.reason_all_causes(&data).unwrap())
    });
}

fn large_causality_collection_benchmark(criterion: &mut Criterion) {
    let (coll, data) = utils_collection::get_large_collection_and_data();
    criterion.bench_function("large_causality_collection", |bencher| {
        bencher.iter(|| coll.reason_all_causes(&data).unwrap())
    });
}

criterion_group! {
    name = causality_collection;
    config = Criterion::default().sample_size(100);
    targets =
    small_causality_collection_benchmark,
    medium_causality_collection_benchmark,
    large_causality_collection_benchmark,
}
