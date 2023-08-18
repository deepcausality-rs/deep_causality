// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Adjustable, AdjustableData, Identifiable};

use crate::types::context_types::adjustable::utils;

#[test]
fn test_update() {
    let mut d = AdjustableData::new(0, 0);
    assert_eq!(d.data(), 0);

    let array_grid = utils::get_array_grid(42);

    let res = d.update(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.data(), 42);
}

#[test]
fn test_update_err() {
    let mut d = AdjustableData::new(0, 42);
    assert_eq!(d.data(), 42);

    let array_grid = utils::get_array_grid(0);

    // Update fails with UpdateError
    let res = d.update(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed update.
    assert_eq!(d.data(), 42);
}

#[test]
fn test_adjust() {
    let mut d = AdjustableData::new(0, 21);
    assert_eq!(d.data(), 21);

    let array_grid = utils::get_array_grid(21);

    let res = d.adjust(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.data(), 42);
}

#[test]
fn test_adjust_err() {
    let mut d = AdjustableData::new(0, 21);
    assert_eq!(d.data(), 21);

    let array_grid = utils::get_array_grid(-23);

    // adjustment fails with AdjustmentError
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed adjustment.
    assert_eq!(d.data(), 21);
}

#[test]
fn test_id() {
    let id = 1;

    let d = AdjustableData::new(id, 21);
    assert_eq!(d.id(), id);
}

#[test]
fn test_to_string() {
    let id = 1;
    let data = 21;

    let d = AdjustableData::new(id, data);
    let exp = format!("AdjustableData: id: {} data: {}", id, data);
    let act = d.to_string();
    assert_eq!(act, exp);
}