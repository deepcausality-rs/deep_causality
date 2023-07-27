// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::fmt::{Display, Formatter};
use deep_causality::prelude::TimeScale;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ParquetConfig
{
    path: String,
    symbol: String,
    time_scale: TimeScale,
}

impl Default for ParquetConfig{
    fn default() -> Self {
        Self {
            path: "".to_string(),
            symbol: "".to_string(),
            time_scale: TimeScale::NoScale,
        }
    }
}

impl ParquetConfig
{

    pub fn new(path: String, symbol: String, time_scale: TimeScale) -> Self {
        Self { path, symbol, time_scale }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn time_scale(&self) -> TimeScale {
        self.time_scale
    }
}

impl Display for ParquetConfig
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParquetConfig symbol: {} path: {} ",
               self.symbol, self.path
        )
    }
}