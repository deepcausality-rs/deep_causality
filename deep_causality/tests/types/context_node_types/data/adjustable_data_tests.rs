/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_array_grid;
use deep_causality::{Adjustable, Data, Datable};

//
// You have to import Adjustable to use update and adjust.
//

#[test]
fn test_update() {
    let mut d = Data::new(0, 0);
    assert_eq!(d.get_data(), 0);

    let array_grid = test_utils_array_grid::get_1d_array_grid(42);

    let res = d.update(&array_grid);
    // dbg!(&res);
    assert!(res.is_ok());

    assert_eq!(d.get_data(), 42);
}

#[test]
fn test_update_err() {
    let mut d = Data::new(0, 42);
    assert_eq!(d.get_data(), 42);

    let array_grid = test_utils_array_grid::get_1d_array_grid(0);

    // Update fails with UpdateError
    let res = d.update(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed update.
    assert_eq!(d.get_data(), 42);
}

#[test]
fn test_adjust() {
    let mut d = Data::new(0, 21);
    assert_eq!(d.get_data(), 21);

    let array_grid = test_utils_array_grid::get_1d_array_grid(21);

    let res = d.adjust(&array_grid);
    // dbg!(&res);
    assert!(res.is_ok());

    assert_eq!(d.get_data(), 42);
}

#[test]
fn test_adjust_err() {
    let mut d = Data::new(0, 21);
    assert_eq!(d.get_data(), 21);

    let array_grid = test_utils_array_grid::get_1d_array_grid(-23);

    // adjustment fails with AdjustmentError
    let res = d.adjust(&array_grid);
    // dbg!(&res);
    assert!(res.is_err());

    // Old value still in place, as before the failed adjustment.
    assert_eq!(d.get_data(), 21);
}
