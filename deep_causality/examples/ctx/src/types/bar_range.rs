// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};

use rust_decimal::Decimal;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct BarRange
{
    high_low: Decimal,
    open_close: Decimal,
    close_above_open: bool,
    close_below_open: bool,
}

impl BarRange
{
    pub fn new(high_low: Decimal, open_close: Decimal, close_above_open: bool, close_below_open: bool)
        -> Self
    {
        Self { high_low, open_close, close_above_open, close_below_open }
    }
}

impl BarRange
{
    pub fn high_low(&self) -> Decimal {
        self.high_low
    }
    pub fn open_close(&self) -> Decimal {
        self.open_close
    }
    pub fn close_above_open(&self) -> bool {
        self.close_above_open
    }
    pub fn close_below_open(&self) -> bool {
        self.close_below_open
    }
}

impl Display for BarRange
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
