// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use chrono::{Datelike, Timelike};
use deep_causality::prelude::{BaseNumberType, Time, TimeScale};

use crate::types::bar_range::BarRange;
use crate::types::date_time_bar::DateTimeBar;
use crate::types::dateoid::CustomData;

pub fn convert_bar_to_augmented(
    data_bar: &DateTimeBar,
    time_scale: TimeScale,
) -> (Time<BaseNumberType>, CustomData) {
    let id = data_bar.date_time().timestamp() as u64;
    let data_range = calculate_ranges(data_bar);

    let time_unit = get_time_unit(data_bar, time_scale) as BaseNumberType;
    let tempoid = Time::new(id, time_scale, time_unit);
    let dataoid = CustomData::new(id, data_range);

    (tempoid, dataoid)
}

fn calculate_ranges(data_bar: &DateTimeBar) -> BarRange {
    let high = data_bar.high();
    let close = data_bar.open();
    let close_above_open = data_bar.close() > data_bar.open();
    let close_below_open = data_bar.close() < data_bar.open();

    BarRange::new(high, close, close_above_open, close_below_open)
}

fn get_time_unit(data_bar: &DateTimeBar, time_scale: TimeScale) -> u32 {
    match time_scale {
        TimeScale::NoScale => data_bar.date_time().minute(),
        TimeScale::Second => data_bar.date_time().second(),
        TimeScale::Minute => data_bar.date_time().minute(),
        TimeScale::Hour => data_bar.date_time().hour(),
        TimeScale::Day => data_bar.date_time().day(),
        TimeScale::Week => data_bar.date_time().iso_week().week(),
        TimeScale::Month => data_bar.date_time().month(),
        TimeScale::Quarter => data_bar.date_time().year() as u32,
        TimeScale::Year => data_bar.date_time().year() as u32,
    }
}
