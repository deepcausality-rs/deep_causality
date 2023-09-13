// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Adjustable, AdjustableSpace};

use crate::types::context_types::adjustable::utils;

#[test]
fn test_update() {
    let mut d = AdjustableSpace::new(0, 1, 2, 3);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    let array_grid = utils::get_3d_array_grid(42, 43, 44);

    let res = d.update(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.x(), &42); // 42

    assert_eq!(d.y(), &43); // 43

    assert_eq!(d.z(), &44); // 44
}

#[test]
fn test_update_err() {
    let mut d = AdjustableSpace::new(0, 1, 2, 3);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    let array_grid = utils::get_3d_array_grid(0, 1, 2);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_3d_array_grid(1, 9, 2);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_3d_array_grid(1, 9, 0);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    // Old values still in place, as before the failed update.
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);
}

#[test]
fn test_adjust() {
    let mut d = AdjustableSpace::new(0, 10, 20, 30);
    assert_eq!(d.x(), &10);
    assert_eq!(d.y(), &20);
    assert_eq!(d.z(), &30);

    let array_grid = utils::get_3d_array_grid(42, 43, 44);

    let res = d.adjust(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.x(), &52);
    assert_eq!(d.y(), &63);
    assert_eq!(d.z(), &74);
}

#[test]
fn test_adjust_err() {
    let mut d = AdjustableSpace::new(0, 1, 2, 3);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    let array_grid = utils::get_3d_array_grid(-10, 20, 30);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_3d_array_grid(10, -10, 30);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_3d_array_grid(10, 10, -10);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    // Old values still in place, as before the failed update.
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);
}

#[test]
fn test_id() {
    let id = 1;

    let d = AdjustableSpace::new(id, 1, 2, 3);
    assert_eq!(d.id(), &id);
}

#[test]
fn test_x() {
    let id = 1;
    let x = 42;

    let d = AdjustableSpace::new(id, x, 2, 3);
    assert_eq!(d.id(), &id);
    assert_eq!(d.x(), &x);
}

#[test]
fn test_y() {
    let id = 1;
    let x = 42;
    let y = 23;

    let d = AdjustableSpace::new(id, x, y, 3);
    assert_eq!(d.id(), &id);
    assert_eq!(d.x(), &x);
    assert_eq!(d.y(), &y);
}

#[test]
fn test_z() {
    let id = 1;
    let x = 42;
    let y = 23;
    let z = 99;

    let d = AdjustableSpace::new(id, x, y, z);
    assert_eq!(d.id(), &id);
    assert_eq!(d.x(), &x);
    assert_eq!(d.y(), &y);
    assert_eq!(d.z(), &z);
}

#[test]
fn test_to_string() {
    let id = 1;
    let x = 42;
    let y = 23;
    let z = 99;

    let d = AdjustableSpace::new(id, x, y, z);
    let exp = format!(
        "AdjustableSpace {{ id={:?}, x={:?}, y={:?}, z={:?} }}",
        id, x, y, z
    );
    let act = d.to_string();
    assert_eq!(act, exp);
}
