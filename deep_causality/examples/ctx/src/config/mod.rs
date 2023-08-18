// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

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
        format!("deep_causality/examples/ctx/data/btc/pqt/{}.parquet", time_scale),
        "btcusd".to_string(),
        *time_scale,
    )
}