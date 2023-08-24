// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::TimeScale;

use crate::types::config_types::parquet_config::ParquetConfig;

pub fn get_file_config() -> ParquetConfig {
    ParquetConfig::new(
        format!("deep_causality/examples/ctx/data/btc/pqt/Day.parquet"),
        "btcusd".to_string(),
        TimeScale::Day,
    )
}
