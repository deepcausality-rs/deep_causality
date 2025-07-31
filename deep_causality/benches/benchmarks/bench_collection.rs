/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};
use deep_causality::*;

use crate::benchmarks::utils_collection;

// Small = 10
// Medium = 1_000
// Large = 10_000

fn small_causality_collection_benchmark(criterion: &mut Criterion) {
    // The `_data` is no longer needed as we pass a single evidence object.
    let (coll, _data) = utils_collection::get_small_collection_and_data();
    // All propagation methods now take a single `&Evidence`.
    let evidence = PropagatingEffect::Numerical(0.99);

    criterion.bench_function("small_causality_collection_propagation", |bencher| {
        bencher.iter(|| {
            coll.evaluate_deterministic(&evidence, &AggregateLogic::All)
                .unwrap()
        })
    });
}

fn medium_causality_collection_benchmark(criterion: &mut Criterion) {
    let (coll, _data) = utils_collection::get_medium_collection_and_data();
    let evidence = PropagatingEffect::Numerical(0.99);

    criterion.bench_function("medium_causality_collection_propagation", |bencher| {
        bencher.iter(|| {
            coll.evaluate_deterministic(&evidence, &AggregateLogic::All)
                .unwrap()
        })
    });
}

fn large_causality_collection_benchmark(criterion: &mut Criterion) {
    let (coll, _data) = utils_collection::get_large_collection_and_data();
    let evidence = PropagatingEffect::Numerical(0.99);

    criterion.bench_function("large_causality_collection_propagation", |bencher| {
        bencher.iter(|| {
            coll.evaluate_deterministic(&evidence, &AggregateLogic::All)
                .unwrap()
        })
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
