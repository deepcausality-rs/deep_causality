// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::types::date_time_bar::DateTimeBar;

#[derive(Debug, Default, Clone)]
pub struct SampledDataBars
{
    day_bars: Vec<DateTimeBar>,
    week_bars: Vec<DateTimeBar>,
    month_bars: Vec<DateTimeBar>,
    year_bars: Vec<DateTimeBar>,
}


impl SampledDataBars
{
    pub fn total_number_of_bars(&self) -> usize {
        self.day_bars.len() + self.week_bars.len() + self.month_bars.len() + self.year_bars.len()
    }
    pub fn set_day_bars(&mut self, day_bars: Vec<DateTimeBar>) {
        self.day_bars = day_bars;
    }
    pub fn set_week_bars(&mut self, week_bars: Vec<DateTimeBar>) {
        self.week_bars = week_bars;
    }
    pub fn set_month_bars(&mut self, month_bars: Vec<DateTimeBar>) {
        self.month_bars = month_bars;
    }
    pub fn set_year_bars(&mut self, year_bars: Vec<DateTimeBar>) {
        self.year_bars = year_bars;
    }

    pub fn day_bars(&self) -> &Vec<DateTimeBar> {
        &self.day_bars
    }
    pub fn week_bars(&self) -> &Vec<DateTimeBar> {
        &self.week_bars
    }
    pub fn month_bars(&self) -> &Vec<DateTimeBar> {
        &self.month_bars
    }
    pub fn year_bars(&self) -> &Vec<DateTimeBar> {
        &self.year_bars
    }
}