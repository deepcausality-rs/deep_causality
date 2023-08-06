// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use deep_causality::prelude::{Identifiable, Tempoid, Temporable, TimeScale};

#[test]
fn test_new() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 12;

    let d = Tempoid::new(id, time_scale, time_unit);
    assert_eq!(d.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 12;

    let d = Tempoid::new(id, time_scale, time_unit);
    assert_eq!(d.id(), id);
}

#[test]
fn test_time_scale() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 12;

    let d = Tempoid::new(id, time_scale, time_unit);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
}

#[test]
fn test_time_unit() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 12;

    let d = Tempoid::new(id, time_scale, time_unit);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
}

#[test]
fn test_to_string() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 12;

    let d = Tempoid::new(id, time_scale, time_unit);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);

    let exp = format!("Tempoid: id: {}, time_scale: {}, time_unit: {}", id, time_scale, time_unit);
    let act = d.to_string();
    assert_eq!(act, exp);
}