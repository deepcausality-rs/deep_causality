// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use std::fs::File;
use std::path::Path;

use parquet::file::reader::{FileReader, SerializedFileReader};
use deep_causality::prelude::TimeScale;
use crate::config;

use crate::io::file::parquet_2_bar;
use crate::types::date_time_bar::DateTimeBar;
use crate::types::parquet_config::ParquetConfig;
use crate::types::sampled_date_time_bar::SampledDataBars;

pub fn read_sampled_bars<'a>(
    time_scale: &'a TimeScale,
    capacity: usize,
    sampled_bars: &'a mut SampledDataBars,
)
    -> Result<(), Box<dyn Error>>
{
    let config = config::get_sampled_bar_config(time_scale);
    read_sampled_bars_from_parquet(&config, capacity, sampled_bars)
        .expect("Failed to read hourly sampled bars from parquet file");

    Ok(())
}

fn read_sampled_bars_from_parquet<'a>(
    config: &'a ParquetConfig,
    capacity: usize,
    sampled_bars: &'a mut SampledDataBars,
)
    -> Result<(), Box<dyn Error>>
{
    let time_scale = config.time_scale();
    let mut content: Vec<DateTimeBar> = Vec::with_capacity(capacity);

    let path = config.path();
    let file = File::open(Path::new(path)).expect("Could not open file");
    let symbol = config.symbol();

    let reader = SerializedFileReader::new(file)
        .expect("Could not create parquet reader");

    let iter = reader
        .get_row_iter(None)
        .expect("Could not create parquet row iterator");

    for record in iter {
        content.push(parquet_2_bar::convert_field_to_date_time_bar(&record.unwrap(), symbol));
    }

    match time_scale {
        TimeScale::Day => sampled_bars.set_day_bars(content),
        TimeScale::Week => sampled_bars.set_week_bars(content),
        TimeScale::Month => sampled_bars.set_month_bars(content),
        TimeScale::Year => sampled_bars.set_year_bars(content),
        _ => {}
    }

    Ok(())
}

