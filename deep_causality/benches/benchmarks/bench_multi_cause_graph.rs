/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};

use deep_causality::*;

use crate::benchmarks::utils_linear_graph;

fn small_multi_layer_graph_benchmark(criterion: &mut Criterion) {
    // The data array is no longer used; we use a single Evidence object.
    let (g, _data) = utils_linear_graph::get_small_multi_cause_graph_and_data();
    let evidence = Evidence::Numerical(0.99);

    let root_index = g.get_root_index().expect("Graph has no root");
    criterion.bench_function("small_multi_layer_graph_evaluate_from_root", |bencher| {
        bencher.iter(|| {
            g.evaluate_subgraph_from_cause(root_index, &evidence)
                .unwrap()
        })
    });

    // Start the subgraph evaluation from a non-root node.
    let index = 1;
    criterion.bench_function(
        "small_multi_layer_graph_evaluate_subgraph_from_cause",
        |bencher| bencher.iter(|| g.evaluate_subgraph_from_cause(index, &evidence).unwrap()),
    );

    let start_index = 1;
    let stop_index = 3;
    criterion.bench_function(
        "small_multi_layer_graph_evaluate_shortest_path",
        |bencher| {
            bencher.iter(|| {
                g.evaluate_shortest_path_between_causes(start_index, stop_index, &evidence)
                    .unwrap()
            })
        },
    );

    // To benchmark a single cause, we get it from the graph and call evaluate() directly.
    let single_cause_index = 2;
    let cause_to_eval = g
        .get_causaloid(single_cause_index)
        .expect("Causaloid not found");
    criterion.bench_function("small_multi_layer_graph_evaluate_single_cause", |bencher| {
        bencher.iter(|| cause_to_eval.evaluate(&evidence).unwrap())
    });
}

criterion_group! {
    name = multi_layer_graph;
    config = Criterion::default().sample_size(100);
    targets =
    small_multi_layer_graph_benchmark,
}
