/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use criterion::{Criterion, criterion_group};
use std::hint::black_box;

use deep_causality::*;

use crate::benchmarks::utils_linear_graph;

const SMALL: usize = 10;
const MEDIUM: usize = 1_000;
const LARGE: usize = 10_000;

fn small_linear_graph_benchmark(criterion: &mut Criterion) {
    let g = utils_linear_graph::build_linear_graph(SMALL);
    let evidence = PropagatingEffect::Numerical(0.99);
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
    criterion.bench_function("small_linear_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| {
            // Perform the graph lookup
            let cause_to_eval = g.get_causaloid(single_cause_index).unwrap();
            // Then perform the evaluation
            // Also black_box the result to ensure this code is never considered "dead" or unused.
            black_box(cause_to_eval.evaluate(&evidence).unwrap());
        })
    });
}

fn medium_linear_graph_benchmark(criterion: &mut Criterion) {
    let g = utils_linear_graph::build_linear_graph(MEDIUM);
    let evidence = PropagatingEffect::Numerical(0.99);
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
    criterion.bench_function("medium_linear_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| {
            // Perform the graph lookup
            let cause_to_eval = g.get_causaloid(single_cause_index).unwrap();
            // Then perform the evaluation
            // Also black_box the result to ensure this code is never considered "dead" or unused.
            black_box(cause_to_eval.evaluate(&evidence).unwrap());
        })
    });
}

fn large_linear_graph_benchmark(criterion: &mut Criterion) {
    let g = utils_linear_graph::build_linear_graph(LARGE);
    let evidence = PropagatingEffect::Numerical(0.99);
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
    criterion.bench_function("large_linear_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| {
            // Perform the graph lookup
            let cause_to_eval = g.get_causaloid(single_cause_index).unwrap();
            // Then perform the evaluation
            // Also black_box the result to ensure this code is never considered "dead" or unused.
            black_box(cause_to_eval.evaluate(&evidence).unwrap());
        })
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
