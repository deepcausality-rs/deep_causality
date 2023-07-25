// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::types::date_time_bar::DateTimeBar;

#[derive(Debug, Clone)]
pub struct SampledDataBars
{
    minute_bars: Vec<DateTimeBar>,
    hour_bars: Vec<DateTimeBar>,
    day_bars: Vec<DateTimeBar>,
    week_bars: Vec<DateTimeBar>,
    month_bars: Vec<DateTimeBar>,
    quarter_bars: Vec<DateTimeBar>,
    year_bars: Vec<DateTimeBar>,
}


impl Default for SampledDataBars
{
    fn default() -> Self {
        Self {
            minute_bars: Vec::new(),
            hour_bars: Vec::new(),
            day_bars: Vec::new(),
            week_bars: Vec::new(),
            month_bars: Vec::new(),
            quarter_bars: Vec::new(),
            year_bars: Vec::new(),
        }
    }
}


impl SampledDataBars
{
    pub fn set_minute_bars(&mut self, minute_bars: Vec<DateTimeBar>) {
        self.minute_bars = minute_bars;
    }
    pub fn set_hour_bars(&mut self, hour_bars: Vec<DateTimeBar>) {
        self.hour_bars = hour_bars;
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
    pub fn set_quarter_bars(&mut self, quarter_bars: Vec<DateTimeBar>) {
        self.quarter_bars = quarter_bars;
    }
    pub fn set_year_bars(&mut self, year_bars: Vec<DateTimeBar>) {
        self.year_bars = year_bars;
    }
}


impl SampledDataBars
{
    pub fn minute_bars(&self) -> &Vec<DateTimeBar> {
        &self.minute_bars
    }
    pub fn hour_bars(&self) -> &Vec<DateTimeBar> {
        &self.hour_bars
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
    pub fn quarter_bars(&self) -> &Vec<DateTimeBar> {
        &self.quarter_bars
    }
    pub fn year_bars(&self) -> &Vec<DateTimeBar> {
        &self.year_bars
    }
}