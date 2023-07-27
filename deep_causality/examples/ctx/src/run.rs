// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::time::{Duration, Instant};
use deep_causality::prelude::TimeScale;
use crate::workflow::build_time_data_context;
use crate::workflow::load_data::load_data;

pub fn run()
{
    let max_time_scale = TimeScale::Day;

    let lap = Instant::now();
    let data = match load_data() {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    let elapsed = &lap.elapsed();
    print_duration("Load Data", elapsed);

    let lap = Instant::now();
    let context = match build_time_data_context(
        1,
        "BTC-1Y".to_uppercase(),
        &data,
        max_time_scale,
    )
    {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

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
