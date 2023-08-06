// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use deep_causality::prelude::TimeScale;
use crate::io::file::read_sampled_bars::read_sampled_bars;
use crate::types::sampled_date_time_bar::SampledDataBars;

pub fn load_data()
    -> Result<SampledDataBars, Box<dyn Error>>
{
    let mut sampled_bars = SampledDataBars::default();
    let scales = vec![TimeScale::Day,TimeScale::Week,TimeScale::Month,TimeScale::Year];

    // Sequential data loading because the mutable reference to sampled_bars prevents parallel loading.
    // However, because the data set is so small, it is not really beneficial to parallelize.
    for time_scale in scales {
        let capacity = get_capacity(&time_scale);
        read_sampled_bars(&time_scale, capacity,&mut sampled_bars)
            .expect("Failed to read sampled data bars from parquet file");
    }

    Ok(sampled_bars)
}

fn get_capacity(
    time_scale: &TimeScale,
)
    -> usize
{
    match time_scale {
        TimeScale::Day => 370,
        TimeScale::Week => 60,
        TimeScale::Month => 12,
        TimeScale::Year => 1,
        _ => 500, // default
    }
}