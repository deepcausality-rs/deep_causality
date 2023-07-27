// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::TimeScale;
use crate::types::parquet_config::ParquetConfig;


pub fn get_sampled_bar_config(
    time_scale: &TimeScale
)
    -> ParquetConfig
{
    match time_scale {
        TimeScale::Day => get_file_config(time_scale),
        TimeScale::Week => get_file_config(time_scale),
        TimeScale::Month => get_file_config(time_scale),
        TimeScale::Year => get_file_config(time_scale),
        _ => ParquetConfig::default(),
    }
}

fn get_file_config(
    time_scale: &TimeScale
)
    -> ParquetConfig
{
    ParquetConfig::new(
        format!("/data/btc/{}.parquet", time_scale.to_string()),
        format!("BTC"),
        *time_scale,
    )
}