// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Identifiable, Spaceoid, Spatial};

#[test]
fn test_new() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Spaceoid::new(id, x, y, z);
    assert_eq!(d.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Spaceoid::new(id, x, y, z);
    assert_eq!(d.id(), id);
}

#[test]
fn test_x() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Spaceoid::new(id, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), x);
}

#[test]
fn test_y() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Spaceoid::new(id, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), x);
    assert_eq!(d.y(), y);
}

#[test]
fn test_z() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Spaceoid::new(id, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), x);
    assert_eq!(d.y(), y);
    assert_eq!(d.z(), z);
}

#[test]
fn test_to_string() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Spaceoid::new(id, x, y, z);
    assert_eq!(d.id(), id);
    assert_eq!(d.x(), x);
    assert_eq!(d.y(), y);
    assert_eq!(d.z(), z);

    let expected = format!("Spaceoid: id={}, x={}, y={}, z={}", id, x, y, z);
    let actual = d.to_string();
    assert_eq!(actual, expected);
}
