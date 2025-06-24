// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::types::context_types::adjustable::utils;
use deep_causality::prelude::{Adjustable, AdjustableSpaceTime, Identifiable, TimeScale};
use deep_causality::traits::contextuable::space_temporal::SpaceTemporal;

#[test]
fn test_update() {
    let id = 1;
    let mut d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, 1, 2, 3);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.time_unit(), &4);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    // Shift coordinates and time
    let array_grid = utils::get_4d_array_grid(2, 3, 4, 5);
    let res = d.update(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.x(), &2);
    assert_eq!(d.y(), &3);
    assert_eq!(d.z(), &4);
    assert_eq!(d.time_unit(), &5);
}

#[test]
fn test_update_err() {
    let id = 1;
    let mut d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, 1, 2, 3);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.time_unit(), &4);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    let array_grid = utils::get_4d_array_grid(0, 3, 4, 5);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_4d_array_grid(1, 0, 4, 5);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_4d_array_grid(1, 2, 0, 5);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_4d_array_grid(1, 2, 3, -5);
    let res = d.update(&array_grid);
    assert!(res.is_err());

    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.time_unit(), &4);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);
}

#[test]
fn test_adjust() {
    let id = 1;
    let mut d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, 1, 2, 3);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.time_unit(), &4);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    // Shift coordinates and time
    let array_grid = utils::get_4d_array_grid(10, 10, 10, 10);

    let res = d.adjust(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.time_scale(), &TimeScale::Minute);

    assert_eq!(d.x(), &11);
    assert_eq!(d.y(), &12);
    assert_eq!(d.z(), &13);
    assert_eq!(d.time_unit(), &14);
}

#[test]
fn test_adjust_err() {
    let id = 1;
    let mut d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, 1, 2, 3);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.time_unit(), &4);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);

    let array_grid = utils::get_4d_array_grid(-10, 3, 4, 5);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_4d_array_grid(1, -10, 4, 5);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_4d_array_grid(1, 2, -10, 5);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    let array_grid = utils::get_4d_array_grid(1, 2, 3, -10);
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), &TimeScale::Minute);
    assert_eq!(d.time_unit(), &4);
    assert_eq!(d.x(), &1);
    assert_eq!(d.y(), &2);
    assert_eq!(d.z(), &3);
}

#[test]
fn test_id() {
    let id = 1;

    let d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, 1, 2, 3);
    assert_eq!(d.id(), id);
}

#[test]
fn test_x() {
    let id = 1;
    let x = 42;

    let d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, x, 2, 3);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), &x);
}

#[test]
fn test_y() {
    let id = 1;
    let x = 42;
    let y = 23;

    let d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, x, y, 3);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), &x);
    assert_eq!(d.y(), &y);
}

#[test]
fn test_z() {
    let id = 1;
    let x = 42;
    let y = 23;
    let z = 99;

    let d = AdjustableSpaceTime::new(id, TimeScale::Minute, 4, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), &x);
    assert_eq!(d.y(), &y);
    assert_eq!(d.z(), &z);
}

#[test]
fn test_t() {
    let id = 1;
    let x = 42;
    let y = 23;
    let z = 99;
    let t = 5;

    let d = AdjustableSpaceTime::new(id, TimeScale::Minute, t, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), &x);
    assert_eq!(d.y(), &y);
    assert_eq!(d.z(), &z);
    assert_eq!(d.t(), &t);
}

#[test]
fn test_to_string() {
    let id = 1;
    let x = 42;
    let y = 23;
    let z = 99;
    let t = 5;

    let d = AdjustableSpaceTime::new(id, TimeScale::Minute, t, x, y, z);
    let exp =
        format!(
        "AdjustableSpaceTime {{ id: {}, time_scale: {:?}, time_unit: {}, x: {}, y: {}, z: {} }}",
        id, TimeScale::Minute, t, x, y, z
    );
    let act = d.to_string();
    assert_eq!(act, exp);
}
