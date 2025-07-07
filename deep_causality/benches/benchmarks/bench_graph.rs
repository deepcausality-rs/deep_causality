/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};

use deep_causality::*;

use crate::benchmarks::utils_linear_graph;

fn small_linear_graph_benchmark(criterion: &mut Criterion) {
    let (g, data) = utils_linear_graph::get_small_linear_graph_and_data();

    criterion.bench_function("small_linear_graph_reason_all_causes", |bencher| {
        bencher.iter(|| g.reason_all_causes(&data, None).unwrap())
    });

    let index = data.len() / 2;

    criterion.bench_function("small_linear_graph_reason_subgraph_from_cause", |bencher| {
        bencher.iter(|| g.reason_subgraph_from_cause(index, &data, None).unwrap())
    });

    let x = data.len() / 2;
    let start_index = x;
    let stop_index = data.len() - 1;

    criterion.bench_function(
        "small_linear_graph_reason_shortest_path_between_causes",
        |bencher| {
            bencher.iter(|| {
                g.reason_shortest_path_between_causes(start_index, stop_index, &data, None)
                    .unwrap()
            })
        },
    );

    let obs = 0.99;

    criterion.bench_function("small_linear_graph_reason_single_cause", |bencher| {
        bencher.iter(|| g.reason_single_cause(index, &[obs]).unwrap())
    });
}

fn medium_linear_graph_benchmark(criterion: &mut Criterion) {
    let (g, data) = utils_linear_graph::get_medium_linear_graph_and_data();

    criterion.bench_function("medium_linear_graph_reason_all_causes", |bencher| {
        bencher.iter(|| g.reason_all_causes(&data, None).unwrap())
    });

    let index = data.len() / 2;

    criterion.bench_function(
        "medium_linear_graph_reason_subgraph_from_cause",
        |bencher| bencher.iter(|| g.reason_subgraph_from_cause(index, &data, None).unwrap()),
    );

    let x = data.len() / 2;
    let start_index = x;
    let stop_index = data.len() - 1;

    criterion.bench_function(
        "medium_linear_graph_reason_shortest_path_between_causes",
        |bencher| {
            bencher.iter(|| {
                g.reason_shortest_path_between_causes(start_index, stop_index, &data, None)
                    .unwrap()
            })
        },
    );

    let obs = 0.99;

    criterion.bench_function(
        "medium_linear_graph_linear_graph_reason_single_cause",
        |bencher| bencher.iter(|| g.reason_single_cause(index, &[obs]).unwrap()),
    );
}

fn large_linear_graph_benchmark(criterion: &mut Criterion) {
    let (g, data) = utils_linear_graph::get_large_linear_graph_and_data();

    criterion.bench_function("large_linear_graph_reason_all_causes", |bencher| {
        bencher.iter(|| g.reason_all_causes(&data, None).unwrap())
    });

    let index = data.len() / 2;

    criterion.bench_function("large_linear_graph_reason_subgraph_from_cause", |bencher| {
        bencher.iter(|| g.reason_subgraph_from_cause(index, &data, None).unwrap())
    });

    let x = data.len() / 2;
    let start_index = x;
    let stop_index = data.len() - 1;

    criterion.bench_function(
        "large_linear_graph_reason_shortest_path_between_causes",
        |bencher| {
            bencher.iter(|| {
                g.reason_shortest_path_between_causes(start_index, stop_index, &data, None)
                    .unwrap()
            })
        },
    );

    let obs = 0.99;

    criterion.bench_function("large_reason_single_cause", |bencher| {
        bencher.iter(|| g.reason_single_cause(index, &[obs]).unwrap())
    });
}

criterion_group! {
    name = linear_graph;
    config = Criterion::default().sample_size(100);
    targets =
    small_linear_graph_benchmark,
    medium_linear_graph_benchmark,
    large_linear_graph_benchmark,
}
