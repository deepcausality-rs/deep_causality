// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::time::Instant;

use deep_causality::prelude::{ContextuableGraph, Identifiable, TimeScale};

use crate::model::get_main_causaloid;
use crate::utils;
use crate::workflow::{build_time_data_context, load_data};
use crate::workflow::build_model::build_model;

pub fn run()
{
    // Determines the maximum level of time resolution in the context hypergraph.
    // Important: If you change this value to a lower level i.e. seconds,
    // the initial context hypergraph generation requires exponentially more time and memory.
    //  For example, 10 years of market data at minute resolution, you would need 30 - 50 GB of memory
    //  and quite a long time to generate a comprehensive context hypergraph. Right now, a context cannot
    //  be serialized, but it's technically possible with Serde.
    let max_time_scale = TimeScale::Day;

    // Node capacity is the maximum number of nodes (Vertexes) that can be stored
    // in the context hypergraph before a dynamic resizing occurs. For large graphs,
    // the resizing operation is quite expensive, so the node capacity should be set to a level
    // that all nodes fit into the graph. Also, too large capacity can lead to memory issues because
    // of large pre-allocation of memory that remains unused. You can discard the number of edges
    // since these have zero impact on the node capacity and only marginally impact memory.
    let node_capacity: usize = 800;

    // Here we load a bunch of pre-sampled market data from parquet files.
    println!("Loading data...");
    let lap = Instant::now();
    let data = match load_data() {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    // Reading parquet files is at least 10x faster than reading CSV files.
    let elapsed = &lap.elapsed();
    utils::print_duration("Load Data", elapsed);

    // This context hypergraph is low resolution (Day), relatively small (<1k nodes),
    // and thus takes only a few milliseconds to generate. In practice,
    // you would probably update the context regularly. Bear in mind, if you do update
    // on a regular basis, the context hypergraph will grow significantly over time. Therefore, you
    // may want to prune old branches every once in a while to stay within the pre-allocated node
    // capacity to prevent expensive graph resizing operations in production.
    println!("Building Context HyperGraph...");
    let lap = Instant::now();
    let context = match build_time_data_context(&data, max_time_scale, node_capacity) {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    let elapsed = &lap.elapsed();
    utils::print_duration("Build Context HyperGraph", elapsed);

    // Print out some key metrics of the context graph.
    println!("Context HyperGraph Metrics:");
    println!("Edge Count: {}", context.edge_count());
    println!("Vertex Count: {}", context.node_count());
    println!("Number Datapoints: {}", data.total_number_of_bars());

    println!();
    println!("Building Causal Model...");
    let lap = Instant::now();
    let causaloid = get_main_causaloid(&context);
    let model = match build_model(&context, &causaloid){
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };
    let elapsed = &lap.elapsed();
    utils::print_duration("Build Causal Model", elapsed);

    println!("Causal Model:");
    println!("Model ID: {}", model.id());
    println!("Model Description: {}", model.description());
    println!();
}
