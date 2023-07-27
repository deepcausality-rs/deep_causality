// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::types::date_time_bar::DateTimeBar;

#[derive(Debug, Clone)]
pub struct SampledDataBars
{
    day_bars: Vec<DateTimeBar>,
    week_bars: Vec<DateTimeBar>,
    month_bars: Vec<DateTimeBar>,
    year_bars: Vec<DateTimeBar>,
}


impl Default for SampledDataBars
{
    fn default() -> Self {
        Self {
            day_bars: Vec::new(),
            week_bars: Vec::new(),
            month_bars: Vec::new(),
            year_bars: Vec::new(),
        }
    }
}


impl SampledDataBars
{
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
}


impl SampledDataBars
{
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