// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use deep_causality::prelude::TimeScale;
use crate::io::file::read_sampled_bars::read_sampled_bars;
use crate::types::sampled_date_time_bar::SampledDataBars;

pub fn load_data()
    -> Result<SampledDataBars, Box<dyn Error>>
{
    let mut result = SampledDataBars::default();
    let scales = vec![TimeScale::Day,TimeScale::Week,TimeScale::Month,TimeScale::Year];

    for time_scale in scales {
        read_sampled_bars(&time_scale, &mut result)
            .expect("Failed to read sampled data bars from parquet file");
    }

    Ok(result)
}
