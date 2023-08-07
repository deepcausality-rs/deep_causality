// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::time::Instant;
use crate::{config, utils};

pub fn run()
{
    println!("Loading file config...");
    let lap = Instant::now();

    let _config = config::get_file_config();

    let elapsed = &lap.elapsed();
    utils::print_duration("Load file config", elapsed);

    println!("Loading data...");
    let lap = Instant::now();

    // let data = file::load_data(&config);

    let elapsed = &lap.elapsed();
    utils::print_duration("Load data", elapsed);
}