// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use deep_causality::prelude::{Identifiable, SpaceTempoid, SpaceTemporal, Spatial, Temporable, TimeScale};

#[test]
fn test_new() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
}

#[test]
fn test_time_scale() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
}

#[test]
fn test_time_unit() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
}

#[test]
fn test_t() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
    assert_eq!(d.t(), time_unit as u64);
}

#[test]
fn test_x() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
    assert_eq!(d.x(), x);
}

#[test]
fn test_y() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
    assert_eq!(d.x(), x);
    assert_eq!(d.y(), y);
}

#[test]
fn test_z() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
    assert_eq!(d.x(), x);
    assert_eq!(d.y(), y);
    assert_eq!(d.z(), z);
}

#[test]
fn test_to_string() {
    let id = 1;
    let time_scale = TimeScale::Minute;
    let time_unit = 12;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTempoid::new(id, time_scale, time_unit, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.time_scale(), time_scale);
    assert_eq!(d.time_unit(), time_unit);
    assert_eq!(d.x(), x);
    assert_eq!(d.y(), y);
    assert_eq!(d.z(), z);

    let expected = format!("SpaceTempoid: id={}, time_scale={}, time_unit={}, x={}, y={}, z={}", id, time_scale, time_unit, x, y, z);
    let actual = d.to_string();
    assert_eq!(actual, expected);
}