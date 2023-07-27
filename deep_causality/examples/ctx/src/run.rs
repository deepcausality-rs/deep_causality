// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::time::{Duration, Instant};
use deep_causality::prelude::TimeScale;
use crate::workflow::{build_time_data_context, load_data};

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
    let lap = Instant::now();
    let data = match load_data() {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    // Reading parquet files is at least 10x faster than reading CSV files.
    let elapsed = &lap.elapsed();
    print_duration("Load Data", elapsed);

    // This context hypergraph is low resolution (Day), relatively small (<1k nodes),
    // and thus takes only a few milliseconds to generate. In practice,
    // you would probably update the context regularly. Bear in mind, if you do update the context
    // on a regular basis, the context hypergraph will grow significantly over time. Therefore, you
    // may want to prune old branches every once in a while to stay within you pre-allocated node
    // capacity to prevent expensive graph resizing operations in production.
    let lap = Instant::now();
    let context = match build_time_data_context(&data, max_time_scale, node_capacity) {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    let elapsed = &lap.elapsed();
    print_duration("Build Context HyperGraph", elapsed);

    // Print out some key metrics of the context graph.
    println!("Edge Count: {}", context.edge_count());
    println!("Vertex Count: {}", context.node_count());
    println!("Number Datapoints: {}", data.total_number_of_bars());
}

fn print_duration(msg: &str, elapsed: &Duration)
{
    println!("{} took: {:?} ", msg, elapsed);
    println!();
}
