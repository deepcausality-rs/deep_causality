/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use criterion::{Criterion, criterion_group};

use deep_causality::prelude::NodeIndex;
use deep_causality::protocols::causable_graph::CausableGraphReasoning;
use deep_causality::utils::bench_utils_graph;

fn small_multi_layer_graph_benchmark(criterion: &mut Criterion)
{
    let (g, data) = bench_utils_graph::get_small_multi_cause_graph_and_data();

    criterion.bench_function("small_multi_layer_graph_reason_all_causes", |bencher| {
        bencher.iter(||
            g.reason_all_causes(&data, None).unwrap()
        )
    });

    let x = data.len() / 2;
    let index = NodeIndex::new(x);

    criterion.bench_function("small_multi_layer_graph_reason_subgraph_from_cause", |bencher| {
        bencher.iter(||
            g.reason_subgraph_from_cause(index, &data, None).unwrap()
        )
    });

    let start_index = NodeIndex::new(0);
    let stop_index = NodeIndex::new(1);

    criterion.bench_function("small_multi_layer_graph_reason_shortest_path_between_causes", |bencher| {
        bencher.iter(||
            g.reason_shortest_path_between_causes(start_index, stop_index, &data, None).unwrap()
        )
    });

    let obs = 0.99;

    criterion.bench_function("small_multi_layer_graph_reason_single_cause", |bencher| {
        bencher.iter(||
            g.reason_single_cause(index, &[obs]).unwrap()
        )
    });
}

criterion_group! {
    name = multi_layer_graph;
    config = Criterion::default().sample_size(100);
    targets =
    small_multi_layer_graph_benchmark,
}