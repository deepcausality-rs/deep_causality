// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Display, Formatter};

use rust_decimal::Decimal;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct BarRange {
    high: Decimal,
    close: Decimal,
    close_above_open: bool,
    close_below_open: bool,
}

impl BarRange {
    pub fn new(
        high_low: Decimal,
        open_close: Decimal,
        close_above_open: bool,
        close_below_open: bool,
    ) -> Self {
        Self {
            high: high_low,
            close: open_close,
            close_above_open,
            close_below_open,
        }
    }
}

impl BarRange {
    pub fn high(&self) -> Decimal {
        self.high
    }
    pub fn close(&self) -> Decimal {
        self.close
    }
    pub fn close_above_open(&self) -> bool {
        self.close_above_open
    }
    pub fn close_below_open(&self) -> bool {
        self.close_below_open
    }
}

impl Display for BarRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
