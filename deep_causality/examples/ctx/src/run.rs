// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::time::{Duration, Instant};
use deep_causality::prelude::TimeScale;
use crate::workflow::{build_time_data_context, load_data};

pub fn run()
{
    // Dermines the lowest level of time resolution in the context hypergraph.
    // Important: If you change this value to a lower level i.e. seconds,
    // the initial context hypergraph generation requires exponentially more time and memory.
    //  For 10 years of market data at minute resolution, you would need 30 - 50 GB of memory
    //  and quite a long time to generate a comprehensive context hypergraph. Right now, a context cannot
    //  be serialized and stored, but it's technically possible with Serde.
    let max_time_scale = TimeScale::Day;

    // Here we just load a bunch of pre-sampled data from parquet files.
    let lap = Instant::now();
    let data = match load_data() {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    // Reading parquet files is at least 10x faster than reading CSV files.
    let elapsed = &lap.elapsed();
    print_duration("Load Data", elapsed);

    //
    let lap = Instant::now();
    let context = match build_time_data_context(&data, max_time_scale)
    {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    //
    let elapsed = &lap.elapsed();
    print_duration("Build Context HyperGraph", elapsed);

    let data_size = data.day_bars().len() as u64;
    let vertex_count = context.node_count();
    let edge_count = context.edge_count();

    println!("Edge Count: {}", edge_count);
    println!("Vertex Count: {}", vertex_count);
    println!("Number Datapoints: {}", data_size);
}

fn print_duration(msg: &str, elapsed: &Duration)
{
    println!("{} took: {:?} ", msg, elapsed);
    println!();
}
