// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.


use deep_causality::prelude::{Adjustable, AdjustableTime, TimeScale};

use crate::types::context_types::adjustable::utils;

#[test]
fn test_update() {
    let mut d = AdjustableTime::new(0, TimeScale::Minute, 12);
    assert_eq!(d.time_unit(), 12);
    assert_eq!(d.time_scale(), TimeScale::Minute);

    let array_grid = utils::get_array_grid(42);

    let res = d.update(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.time_unit(), 42);
}

#[test]
fn test_update_err() {
    let mut d = AdjustableTime::new(0, TimeScale::Minute, 42);
    assert_eq!(d.time_unit(), 42);
    assert_eq!(d.time_scale(), TimeScale::Minute);

    let array_grid = utils::get_array_grid(0);

    // Update fails with UpdateError
    let res = d.update(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed update.
    assert_eq!(d.time_unit(), 42);
}


#[test]
fn test_adjust() {
    let mut d = AdjustableTime::new(0, TimeScale::Minute, 42);
    assert_eq!(d.time_unit(), 42);
    assert_eq!(d.time_scale(), TimeScale::Minute);

    let array_grid = utils::get_array_grid(22);

    let res = d.adjust(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.time_unit(), 64);
}


#[test]
fn test_adjust_err() {
    let mut d = AdjustableTime::new(0, TimeScale::Minute, 21);
    assert_eq!(d.time_unit(), 21);
    assert_eq!(d.time_scale(), TimeScale::Minute);

    let array_grid = utils::get_array_grid(-23);

    // adjustment fails with AdjustmentError
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed adjustment.
    assert_eq!(d.time_unit(), 21);
}