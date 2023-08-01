// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use crate::types::data_symbol::DataSymbol;


#[derive(Deserialize, Serialize, Debug, Copy, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct DateTimeBar
{
    date_time: DateTime<Utc>,
    symbol: DataSymbol,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    one_hundred: Decimal,
}


impl DateTimeBar
{
    pub fn new(symbol: DataSymbol, date_time: DateTime<Utc>, open: Decimal, high: Decimal, low: Decimal, close: Decimal, volume: Decimal) -> Self {
        Self { symbol, date_time, open, high, low, close, volume, one_hundred: Decimal::new(100, 2) }
    }
}

impl Default for DateTimeBar
{
    fn default() -> Self
    {
        Self
        {
            date_time: Utc::now(),
            symbol: DataSymbol::default(),
            open: Decimal::default(),
            high: Decimal::default(),
            low: Decimal::default(),
            close: Decimal::default(),
            volume: Decimal::default(),
            one_hundred: Decimal::new(100, 2),
        }
    }
}

impl DateTimeBar
{
    pub fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }
    pub fn open(&self) -> Decimal {
        self.open
    }
    pub fn high(&self) -> Decimal {
        self.high
    }
    pub fn low(&self) -> Decimal {
        self.low
    }
    pub fn close(&self) -> Decimal {
        self.close
    }
}

impl Display for DateTimeBar
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "DataTime: {},\n Symbol {},\n Open {},\n High {},\n Low {},\n Close {},\n Volume {},",
            self.date_time, self.symbol, self.open, self.high, self.low, self.close, self.volume
        )
    }
}