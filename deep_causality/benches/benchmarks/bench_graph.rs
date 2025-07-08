/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};

use deep_causality::*;

use crate::benchmarks::utils_linear_graph;

fn small_linear_graph_benchmark(criterion: &mut Criterion) {
    let (g, _data) = utils_linear_graph::get_small_linear_graph_and_data();
    let evidence = Evidence::Numerical(0.99);
    let root_index = g.get_root_index().unwrap();

    criterion.bench_function(
        "small_linear_graph_evaluate_subgraph_from_root",
        |bencher| {
            bencher.iter(|| {
                g.evaluate_subgraph_from_cause(root_index, &evidence)
                    .unwrap()
            })
        },
    );

    let start_index = g.number_nodes() / 2;
    let stop_index = g.number_nodes() - 1;

    criterion.bench_function("small_linear_graph_evaluate_shortest_path", |bencher| {
        bencher.iter(|| {
            g.evaluate_shortest_path_between_causes(start_index, stop_index, &evidence)
                .unwrap()
        })
    });

    let single_cause_index = g.number_nodes() / 2;
    let cause_to_eval = g.get_causaloid(single_cause_index).unwrap();
    criterion.bench_function("small_linear_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| cause_to_eval.evaluate(&evidence).unwrap())
    });
}

fn medium_linear_graph_benchmark(criterion: &mut Criterion) {
    let (g, _data) = utils_linear_graph::get_medium_linear_graph_and_data();
    let evidence = Evidence::Numerical(0.99);
    let root_index = g.get_root_index().unwrap();

    criterion.bench_function(
        "medium_linear_graph_evaluate_subgraph_from_root",
        |bencher| {
            bencher.iter(|| {
                g.evaluate_subgraph_from_cause(root_index, &evidence)
                    .unwrap()
            })
        },
    );

    let start_index = g.number_nodes() / 2;
    let stop_index = g.number_nodes() - 1;

    criterion.bench_function("medium_linear_graph_evaluate_shortest_path", |bencher| {
        bencher.iter(|| {
            g.evaluate_shortest_path_between_causes(start_index, stop_index, &evidence)
                .unwrap()
        })
    });

    let single_cause_index = g.number_nodes() / 2;
    let cause_to_eval = g.get_causaloid(single_cause_index).unwrap();
    criterion.bench_function("medium_linear_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| cause_to_eval.evaluate(&evidence).unwrap())
    });
}

fn large_linear_graph_benchmark(criterion: &mut Criterion) {
    let (g, _data) = utils_linear_graph::get_large_linear_graph_and_data();
    let evidence = Evidence::Numerical(0.99);
    let root_index = g.get_root_index().unwrap();

    criterion.bench_function(
        "large_linear_graph_evaluate_subgraph_from_root",
        |bencher| {
            bencher.iter(|| {
                g.evaluate_subgraph_from_cause(root_index, &evidence)
                    .unwrap()
            })
        },
    );

    let start_index = g.number_nodes() / 2;
    let stop_index = g.number_nodes() - 1;

    criterion.bench_function("large_linear_graph_evaluate_shortest_path", |bencher| {
        bencher.iter(|| {
            g.evaluate_shortest_path_between_causes(start_index, stop_index, &evidence)
                .unwrap()
        })
    });

    let single_cause_index = g.number_nodes() / 2;
    let cause_to_eval = g.get_causaloid(single_cause_index).unwrap();
    criterion.bench_function("large_linear_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| cause_to_eval.evaluate(&evidence).unwrap())
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
