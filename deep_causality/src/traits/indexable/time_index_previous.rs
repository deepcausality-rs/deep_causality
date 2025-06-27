/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{TimeIndexable, TimeScale};

pub trait PreviousTimeIndex: TimeIndexable {
    fn get_previous_year_index(&self) -> Option<&usize> {
        let key = TimeScale::Year as usize;
        self.get_time_index(&key, false)
    }

    fn get_previous_month_index(&self) -> Option<&usize> {
        let key = TimeScale::Month as usize;
        self.get_time_index(&key, false)
    }

    fn get_previous_week_index(&self) -> Option<&usize> {
        let key = TimeScale::Week as usize;
        self.get_time_index(&key, false)
    }

    fn get_previous_day_index(&self) -> Option<&usize> {
        let key = TimeScale::Day as usize;
        self.get_time_index(&key, false)
    }

    fn get_previous_hour_index(&self) -> Option<&usize> {
        let key = TimeScale::Hour as usize;
        self.get_time_index(&key, false)
    }

    fn get_previous_minute_index(&self) -> Option<&usize> {
        let key = TimeScale::Minute as usize;
        self.get_time_index(&key, false)
    }

    fn set_previous_year_index(&mut self, index: usize) {
        let key = TimeScale::Year as usize;
        self.set_time_index(key, index, false)
    }

    fn set_previous_month_index(&mut self, index: usize) {
        let key = TimeScale::Month as usize;
        self.set_time_index(key, index, false)
    }

    fn set_previous_week_index(&mut self, index: usize) {
        let key = TimeScale::Week as usize;
        self.set_time_index(key, index, false)
    }

    fn set_previous_day_index(&mut self, index: usize) {
        let key = TimeScale::Day as usize;
        self.set_time_index(key, index, false)
    }

    fn set_previous_hour_index(&mut self, index: usize) {
        let key = TimeScale::Hour as usize;
        self.set_time_index(key, index, false)
    }

    fn set_previous_minute_index(&mut self, index: usize) {
        let key = TimeScale::Minute as usize;
        self.set_time_index(key, index, false)
    }
}
